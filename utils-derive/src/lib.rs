use proc_macro::{self, TokenStream};
use quote::quote;
use syn::*;

#[proc_macro_derive(FromWords)]
pub fn derive_from_words(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl ::std::str::FromStr for #ident {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(utils::convert::words(s).collect()))
            }
        }
    };
    output.into()
}

#[proc_macro_derive(Charty)]
pub fn derive_charty(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let variants = match data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("Chars can only be used with enums"),
    };

    let variants = variants.iter()
        .map(|Variant {ident, discriminant, ..}| {
            if let Some((_, expr)) = discriminant {
                match expr {
                    Expr::Lit(ExprLit {lit, ..}) => match lit {
                        Lit::Byte(lit) => {
                            let value = lit.value();
                            quote! {
                                #value => Ok(#ident),
                            }
                        }
                        _ => panic!("Chars discriminants must be bytes"),
                    }
                    _ => panic!(),
                }
            } else {
                panic!("Chars variants must have discriminants");
            }
        });

    let try_from_char = quote! {
        impl ::std::convert::TryFrom<char> for #ident {
            type Error = ();
            fn try_from(value: char) -> Result<Self, Self::Error> {
                match value as u8 {
                    #(#variants)*
                    _ => Err(()),
                }
            }
        }
    };

    let output = quote! {
        impl ::std::convert::Into<char> for #ident {
            fn into(self) -> char {
                self as u8 as char
            }
        }

        #try_from_char

        impl ::std::marker::Copy for #ident {}
        impl ::std::clone::Clone for #ident {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl ::std::cmp::Eq for #ident {}
        impl ::std::cmp::PartialEq for #ident {
            fn eq(&self, other: &Self) -> bool {
                (*self as u8) == (*other as u8)
            }
        }
    };
    output.into()
}
