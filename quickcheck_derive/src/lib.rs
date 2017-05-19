#![crate_type = "proc-macro"]
#![recursion_limit = "128"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod attrs;
mod structural;

use attrs::*;
use proc_macro::TokenStream;
use structural::*;
use syn::{Body, parse_derive_input};

#[proc_macro_derive(Arbitrary, attributes(arbitrary))]
pub fn derive(input: TokenStream) -> TokenStream {
    let item = parse_derive_input(&input.to_string())
        .expect("Couldn't parse item");
    let (arbitrary, shrink) = match item.body {
        Body::Struct(ref variant) => derive_struct(&item, &variant),
        Body::Enum(ref variants) => derive_enum(&item, &variants),
    };
    let valid = process_attrs(&item);

    let name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let ast = quote! {
        impl ::quickcheck::Arbitrary for #impl_generics #name #ty_generics #where_clause {
            #[allow(unused_mut, unused_variables)]
            fn arbitrary<G: ::quickcheck::Gen>(_g: &mut G) -> Self {
                // TODO Find a way to use "self" instead of "this".
                let valid = |this: &Self| { #valid };
                let mut gen = move || { #arbitrary };

                loop {
                    let out = gen();
                    if valid(&out) {
                        return out;
                    }
                }
            }

            fn shrink(&self) -> Box<Iterator<Item=Self>> {
                #shrink
            }
        }
    };
    ast.to_string()
       .parse()
       .expect("Couldn't parse string to tokens")
}
