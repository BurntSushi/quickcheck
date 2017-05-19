use quote::Tokens;
use syn::{DeriveInput, Field, Ident, Variant, VariantData};

pub fn derive_struct(item: &DeriveInput, variant: &VariantData) -> (Tokens, Tokens) {
    let name = &item.ident;
    
    let arbitrary = arbitrary_variant(variant, name);
    let shrink = match *variant {
        VariantData::Struct(ref fields) => {
            let field_names = fields.iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect::<Vec<_>>();
            let field_names = &field_names;
            let alphas = alpha_names(fields.len());
            let alphas = &alphas;

            let tuple_pattern = match alphas.len() {
                0 => quote!(()),
                1 => {
                    let alpha = &alphas[0];
                    quote!(#alpha)
                },
                _ => quote!((#(#alphas),*)),
            };

            quote! {
                Box::new(
                    (#(self.#field_names),*).shrink().map(|#tuple_pattern| #name {
                        #(#field_names: #alphas),*
                    })
                )
            }
        },
        VariantData::Tuple(ref fields) => {
            let field_names = (0..fields.len()).map(Ident::new).map(|i| quote!(self.#i));
            let alpha_names = &alpha_names(fields.len());

            quote! {
                // TODO This isn't a *great* way to do this until we get
                // generics over tuples, to be able to implement shrinking
                // for tuples of all sizes.
                Box::new((#(#field_names),*).shrink().map(|(#(#alpha_names),*)| #name(#(#alpha_names),*)))
            }
        },
        VariantData::Unit => quote!(quickcheck::empty_shrinker()),
    };

    (arbitrary, shrink)
}

pub fn derive_enum(item: &DeriveInput, variants: &[Variant]) -> (Tokens, Tokens) {
    let name = &item.ident;
    let variant_count = variants.len();
    if variants.len() == 0 {
        panic!("Can't derive Arbitrary on an uninhabited type!");
    }

    let arbitrary_variants = variants.iter().enumerate().map(|(i, v)| {
        let arb = arbitrary_variant(&v.data, &v.ident);
        quote!(#i => #name::#arb)
    });
    let shrink_variants = variants.iter().map(|v| enum_shrink_variant(name, v));

    let arbitrary = quote! {
        match _g.gen_range(0, #variant_count) {
            #(#arbitrary_variants,)*
            _ => unreachable!(),
        }
    };
    let shrink = quote! {
        match *self {
            #(#shrink_variants,)*
        }
    };

    (arbitrary, shrink)
}
fn arbitrary_variant(variant: &VariantData, name: &Ident) -> Tokens {
    match *variant {
        VariantData::Struct(ref fields) => {
            let fields = fields.iter().map(derive_field);
            
            quote! {
                #name {
                    #(#fields),*
                }
            }
        },
        VariantData::Tuple(ref fields) => {
            let arbitraries = fields.iter().map(derive_field);
 
            quote! {
                #name (#(#arbitraries),*)
            }
        },
        VariantData::Unit => quote!(#name),
    }
}

fn derive_field(field: &Field) -> Tokens {
    let gen = quote! { ::quickcheck::Arbitrary::arbitrary(_g) };
    if let Some(ref name) = field.ident {
        quote! { #name: #gen }
    } else {
        quote! { #gen }
    }
}

fn alpha_name(n: usize) -> Ident {
    Ident::new(format!("quickcheck_derived_param_{}", n))
}
fn alpha_names(i: usize) -> Vec<Ident> {
    (0..i).map(alpha_name).collect()
}

fn enum_shrink_variant(name: &Ident, v: &Variant) -> Tokens {
    let ident = &v.ident;
    match v.data {
        VariantData::Struct(ref fields) => {
            let field_names = fields.iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect::<Vec<_>>();
            let field_names = &field_names;
            let alphas = alpha_names(fields.len());
            let alphas = &alphas;

            let tuple_pattern = match alphas.len() {
                0 => quote!(()),
                1 => {
                    let alpha = &alphas[0];
                    quote!(#alpha)
                },
                _ => quote!((#(#alphas),*)),
            };

            quote! {
                #name::#ident {
                    #(#field_names: ref #alphas),*
                } => {
                    let iter = (#(#alphas.clone()),*).shrink().map(|#tuple_pattern| #name::#ident {
                        #(#field_names: #alphas),*
                    });
                    Box::new(iter)
                }
            }
        },
        VariantData::Tuple(ref fields) => {
            let l = fields.len();
            let alphas = alpha_names(l);
            let alphas = &alphas;
            let tuple_pattern = match alphas.len() {
                0 => quote!(()),
                1 => {
                    let alpha = &alphas[0];
                    quote!(#alpha)
                },
                _ => quote!((#(#alphas),*)),
            };
            quote! {
                #name::#ident(#(ref #alphas),*) => {
                    let iter = (#(#alphas.clone()),*)
                        .shrink()
                        .map(|#tuple_pattern| #name::#ident(#(#alphas),*));
                    Box::new(iter)
                }
            }
        },
        VariantData::Unit => quote!(#name::#ident => ::quickcheck::empty_shrinker()),
    }
}
