use quote::Tokens;
use syn::{Attribute, DeriveInput, Ident, Lit, MetaItem, NestedMetaItem};

pub fn process_attrs(item: &DeriveInput) -> Tokens {
    let mut toks = quote!(true);
    for constraint in item.attrs.iter().flat_map(constraints) {
        toks.append("&&");
        toks.append("(");
        toks.append(constraint);
        toks.append(")");
    }
    toks
}

fn constraints(attr: &Attribute) -> Vec<&str> {
    match attr.value {
        MetaItem::List(ref name, ref nested) => if name == &Ident::new("arbitrary") {
            nested.iter().filter_map(|n| match *n {
                NestedMetaItem::MetaItem(ref m) => match *m {
                    MetaItem::NameValue(ref name, ref val) => if name == &Ident::new("constraint") {
                        match *val {
                            Lit::Str(ref s, _) => Some(s as &str),
                            _ => panic!("Invalid 'arbitrary' attribute"),
                        }
                    } else {
                        panic!("Invalid 'arbitrary' attribute");
                    },
                    _ => panic!("Invalid 'arbitrary' attribute"),
                },
                _ => panic!("Invalid 'arbitrary' attribute"),
            }).collect()
        } else {
            Vec::new()
        },
        _ => Vec::new(),
    }
}
