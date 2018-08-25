//! This crate provides the `#[quickcheck]` attribute. Its use is
//! documented in the `quickcheck` crate.

#![crate_name = "quickcheck_macros"]
#![crate_type = "dylib"]
#![doc(html_root_url = "http://burntsushi.net/rustdoc/quickcheck")]

#![feature(plugin_registrar, rustc_private)]

extern crate syntax;
extern crate rustc_plugin;

use syntax::ast;
use syntax::ast::{Ident, ItemKind, PatKind, StmtKind, Stmt, TyKind};
use syntax::source_map;
use syntax::ext::base::{ExtCtxt, MultiModifier, Annotatable};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;
use syntax::symbol::Symbol;

use rustc_plugin::Registry;

/// For the `#[quickcheck]` attribute. Do not use.
#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(Symbol::intern("quickcheck"),
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
                          span: source_map::Span,
                          _: &ast::MetaItem,
                          annot_item: Annotatable) -> Annotatable {
    let item = annot_item.expect_item();
    match item.node {
        ItemKind::Fn(ref decl, header, _, _) => {
            let prop_ident = cx.expr_ident(span, item.ident);
            let prop_ty = cx.ty(span, TyKind::BareFn(P(ast::BareFnTy {
                unsafety: header.unsafety,
                abi: header.abi,
                generic_params: vec![],
                decl: decl.clone().map(|mut decl| {
                    for arg in decl.inputs.iter_mut() {
                        arg.pat = arg.pat.clone().map(|mut pat| {
                            pat.node = PatKind::Wild;
                            pat
                        });
                    }
                    decl
                }),
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
             span: source_map::Span,
             item: &ast::Item,
             inner_ident: P<ast::Expr>) -> Annotatable {
    // Copy original function without attributes
    let prop = P(ast::Item {attrs: Vec::new(), ..item.clone()});
    // ::quickcheck::quickcheck
    let check_ident = Ident::from_str("quickcheck");
    let check_path = vec!(check_ident, check_ident);
    // Wrap original function in new outer function,
    // calling ::quickcheck::quickcheck()
    let fn_decl = Stmt {
        id: ast::DUMMY_NODE_ID,
        node: StmtKind::Item(prop),
        span: span,
    };
    let check_call = Stmt {
        id: ast::DUMMY_NODE_ID,
        node: StmtKind::Expr(cx.expr_call_global(span, check_path, vec![inner_ident])),
        span: span,
    };
    let body = cx.block(span, vec![fn_decl, check_call]);
    let test = cx.item_fn(span, item.ident, vec![], cx.ty(span, TyKind::Tup(vec![])), body);

    // Copy attributes from original function
    let mut attrs = item.attrs.clone();
    // Add #[test] attribute
    attrs.push(cx.attribute(
        span, cx.meta_word(span, Symbol::intern("test"))));
    // Attach the attributes to the outer function
    Annotatable::Item(P(ast::Item {attrs: attrs, ..(*test).clone()}))
}
