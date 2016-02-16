//! This crate provides the `#[quickcheck]` attribute. Its use is
//! documented in the `quickcheck` crate.

#![crate_name = "quickcheck_macros"]
#![crate_type = "dylib"]
#![doc(html_root_url = "http://burntsushi.net/rustdoc/quickcheck")]

#![feature(plugin_registrar, rustc_private)]

extern crate syntax;
extern crate rustc_plugin;

use syntax::abi::Abi;
use syntax::ast;
use syntax::ast::{DeclKind, ItemKind, StmtKind, TyKind};
use syntax::codemap;
use syntax::parse::token;
use syntax::ext::base::{ExtCtxt, MultiModifier, Annotatable};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

use rustc_plugin::Registry;

/// For the `#[quickcheck]` attribute. Do not use.
#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(token::intern("quickcheck"),
                                  MultiModifier(Box::new(expand_meta_quickcheck)));
}

/// Expands the `#[quickcheck]` attribute.
///
/// Expands:
/// ```
/// #[quickcheck]
/// fn check_something(_: usize) -> bool {
///     true
/// }
/// ```
/// to:
/// ```
/// #[test]
/// fn check_something() {
///     fn check_something(_: usize) -> bool {
///         true
///     }
///     ::quickcheck::quickcheck(check_something as fn(usize) -> bool)
/// }
/// ```
fn expand_meta_quickcheck(cx: &mut ExtCtxt,
                          span: codemap::Span,
                          _: &ast::MetaItem,
                          annot_item: Annotatable) -> Annotatable {
    let item = annot_item.expect_item();
    match item.node {
        ItemKind::Fn(ref decl, unsafety, _constness, abi, _, _) => {
            let prop_ident = cx.expr_ident(span, item.ident);
            let prop_ty = cx.ty(span, TyKind::BareFn(P(ast::BareFnTy {
                unsafety: unsafety,
                abi: abi,
                lifetimes: vec![],
                decl: decl.clone(),
            })));
            let inner_ident = cx.expr_cast(span, prop_ident, prop_ty);
            return wrap_item(cx, span, &*item, inner_ident);
        },
        ItemKind::Static(..) => {
            let inner_ident = cx.expr_ident(span, item.ident);
            return wrap_item(cx, span, &*item, inner_ident);
        },
        _ => {
            cx.span_err(
                span, "#[quickcheck] only supported on statics and functions");
        }
    }
    Annotatable::Item(item)
}

fn wrap_item(cx: &mut ExtCtxt,
             span: codemap::Span,
             item: &ast::Item,
             inner_ident: P<ast::Expr>) -> Annotatable {
    // Copy original function without attributes
    let prop = P(ast::Item {attrs: Vec::new(), ..item.clone()});
    // ::quickcheck::quickcheck
    let check_ident = token::str_to_ident("quickcheck");
    let check_path = vec!(check_ident, check_ident);
    // Wrap original function in new outer function,
    // calling ::quickcheck::quickcheck()
    let fn_decl = P(codemap::respan(span, DeclKind::Item(prop.clone())));
    let inner_fn = codemap::respan(
        span, StmtKind::Decl(fn_decl, ast::DUMMY_NODE_ID));
    let check_call = cx.expr_call_global(span, check_path, vec![inner_ident]);
    let body = cx.block(span, vec![inner_fn], Some(check_call));
    let test = item_fn(cx, span, item, body);

    // Copy attributes from original function
    let mut attrs = item.attrs.clone();
    // Add #[test] attribute
    attrs.push(cx.attribute(
        span, cx.meta_word(span, token::intern_and_get_ident("test"))));
    // Attach the attributes to the outer function
    Annotatable::Item(P(ast::Item {attrs: attrs, ..(*test).clone()}))
}

fn item_fn(cx: &mut ExtCtxt, span: codemap::Span,
           towrap_item: &ast::Item, body: P<ast::Block>) -> P<ast::Item> {
    let decl = P(ast::FnDecl {
        inputs: vec![],
        output: ast::FunctionRetTy::Default(span),
        variadic: false,
    });
    let item = ItemKind::Fn(
        decl, ast::Unsafety::Normal, ast::Constness::NotConst, Abi::Rust,
        ast::Generics::default(), body);
    cx.item(span, towrap_item.ident, vec![], item)
}
