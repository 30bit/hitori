use crate::utils::path_eq_ident_str;
use proc_macro2::Ident;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, ExprLit, Lit, Meta, MetaNameValue, Token, Visibility,
};

pub struct Args {
    pub capture_vis: Option<Visibility>,
    pub capture_ident: Option<Ident>,
}

impl TryFrom<Punctuated<Meta, Token![,]>> for Args {
    type Error = syn::Error;

    fn try_from(args: Punctuated<Meta, Token![,]>) -> syn::Result<Self> {
        let mut capture_ident = None;
        let mut capture_vis = None;

        for arg in &args {
            match arg {
                Meta::NameValue(MetaNameValue {
                    path,
                    value:
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }),
                    ..
                }) => {
                    if path_eq_ident_str(path, "with_capture") {
                        if capture_ident.is_none() {
                            capture_ident = Some(s.parse()?);
                        } else {
                            return Err(syn::Error::new_spanned(path, "duplicate `with_capture`"));
                        }
                    } else if path_eq_ident_str(path, "with_capture_vis") {
                        if capture_vis.is_none() {
                            capture_vis = Some(s.parse()?);
                        } else {
                            return Err(syn::Error::new_spanned(
                                path,
                                "duplicate `with_capture_vis`",
                            ));
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        arg,
                        "expected `with_capture` or `with_capture_vis` and literal string value",
                    ))
                }
            }
        }

        Ok(Self {
            capture_vis,
            capture_ident,
        })
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input
            .parse_terminated(Meta::parse, Token![,])
            .and_then(TryInto::try_into)
    }
}
