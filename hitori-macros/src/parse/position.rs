use proc_macro2::Ident;
use syn::{parse::Parse, punctuated::Punctuated, Token};

pub enum Position {
    First,
    Last,
    FirstAndLast,
}

impl Parse for Position {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let idents = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        let mut first = false;
        let mut last = false;

        for ident in &idents {
            if ident == "first" {
                if first {
                    return Err(syn::Error::new_spanned(ident, "duplicate"));
                } else {
                    first = true;
                }
            } else if ident == "last" {
                if last {
                    return Err(syn::Error::new_spanned(ident, "duplicate"));
                } else {
                    last = true;
                }
            }
        }

        match (first, last) {
            (true, true) => Ok(Self::FirstAndLast),
            (true, false) => Ok(Self::First),
            (false, true) => Ok(Self::Last),
            (false, false) => Err(syn::Error::new_spanned(
                idents,
                "expected `first`, or `last`, or both",
            )),
        }
    }
}
