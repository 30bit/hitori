use crate::utils::path_eq_ident_str;
use proc_macro2::Ident;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Lit, Meta, MetaNameValue, Token, Visibility,
};

pub struct Args {
    pub vis: Option<Visibility>,
    pub capture_ident: Option<Ident>,
}

impl TryFrom<Punctuated<Meta, Token![,]>> for Args {
    type Error = syn::Error;

    fn try_from(args: Punctuated<Meta, Token![,]>) -> syn::Result<Self> {
        let mut capture = None;
        let mut vis = None;

        for arg in &args {
            match arg {
                Meta::NameValue(MetaNameValue {
                    path,
                    lit: Lit::Str(s),
                    ..
                }) => {
                    if path_eq_ident_str(path, "with_capture") {
                        if capture.is_none() {
                            capture = Some(s.parse()?);
                        } else {
                            return Err(syn::Error::new_spanned(path, "duplicate `with_capture`"));
                        }
                    } else if path_eq_ident_str(path, "with_vis") {
                        if vis.is_none() {
                            vis = Some(s.parse()?);
                        } else {
                            return Err(syn::Error::new_spanned(path, "duplicate `with_vis`"));
                        }
                    }
                }
                _ => return Err(syn::Error::new_spanned(arg, "unexpected argument")),
            }
        }

        Ok(Self { vis, capture_ident: capture })
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input
            .parse_terminated::<_, Token![,]>(Meta::parse)
            .and_then(TryInto::try_into)
    }
}
