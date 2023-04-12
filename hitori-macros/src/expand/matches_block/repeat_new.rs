use crate::{parse::repeat::Repeat, utils::UsizeOrExpr};
use proc_macro2::TokenStream;
use quote::quote;

pub fn bounds_decl(repeat: &Repeat) -> TokenStream {
    match repeat {
        Repeat::Exact(lo) | Repeat::InInclusive { lo, hi: None } => {
            let mut output = quote! {
                let lo = #lo;
            };
            if matches!(lo, UsizeOrExpr::Expr(_)) {
                output.extend(quote! {
                    fn check_is_usize(_: usize) {}
                    check_is_usize(lo);
                })
            }
            output
        }
        Repeat::InInclusive { lo, hi: Some(hi) } => {
            let mut output = quote! {
                let lo = #lo;
                let hi = #hi;
            };
            if matches!(lo, UsizeOrExpr::Expr(_)) | matches!(hi, UsizeOrExpr::Expr(_)) {
                output.extend(quote! {
                    fn check_is_usize(_: usize) {}
                });
            }
            if matches!(lo, UsizeOrExpr::Expr(_)) {
                output.extend(quote! {
                    check_is_usize(lo);
                });
            }
            if matches!(hi, UsizeOrExpr::Expr(_)) {
                output.extend(quote! {
                    check_is_usize(hi);
                });
            }
            output
        }
    }
}
