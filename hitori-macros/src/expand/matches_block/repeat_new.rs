use std::collections::BTreeSet;

use crate::parse::repeat::Repeat;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

fn bounds_decl(repeat: &Repeat) -> TokenStream {
    match repeat {
        Repeat::Exact(lo_included) | Repeat::InInclusive { lo_included, hi_excluded: None } => {
            quote! {
                let lo: usize = #lo_included;
            }
        }
        Repeat::InInclusive { lo_included, hi_excluded: Some(hi_excluded) } => {
            quote! {
                let lo: usize = #lo_included;
                let hi: usize = #hi_excluded;
                if lo > hi {
                    return false;
                }
            }
        }
    }
}

fn lo_test(
    inner_matches_ident: &Ident,
    inner_unique_capture_idents: &BTreeSet<Ident>,
) -> TokenStream {
    quote! {
        if lo != 0 {
            let cloned_iter = ::core::clone::Clone::clone(&self.__iter);
            if lo > 1 {

            } else {
                if !self.#inner_matches_ident() {
                    self.__iter = cloned_iter;
                    return false;
                }
            }
        }
    }
}
