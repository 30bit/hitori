use std::collections::BTreeSet;

use crate::parse::repeat::Repeat;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

fn bounds_decl(repeat: &Repeat) -> TokenStream {
    match repeat {
        Repeat::Exact(lo_included)
        | Repeat::InInclusive {
            lo_included,
            hi_excluded: None,
        } => {
            quote! {
                let lo: usize = #lo_included;
            }
        }
        Repeat::InInclusive {
            lo_included,
            hi_excluded: Some(hi_excluded),
        } => {
            quote! {
                let lo: usize = #lo_included;
                let hi: usize = #hi_excluded;
                if lo >= hi {
                    return false;
                }
            }
        }
    }
}

fn lo_test(inner_matches_ident: &Ident) -> TokenStream {
    quote! {
        if lo != 0 {
            if !self.#inner_matches_ident() {
                return false;
            }
            for _ in 1..lo {
                if !self.#inner_matches_ident() {
                    return false;
                }
            }
        }
    }
}

fn some_hi_test(
    inner_matches_ident: &Ident,
    inner_unique_capture_idents: &BTreeSet<Ident>,
) -> TokenStream {
    quote! {
        if lo + 1 == hi {
            return true;
        }
        let mut cloned_iter = ::core::clone::Clone::clone(&self.__iter);
        #(
            let mut #inner_unique_capture_idents =
                ::core::clone::Clone::clone(&self.__capture.#inner_unique_capture_idents);
        )*
        for _ in lo..(hi - 1) {
            if self.#inner_matches_ident() {
                cloned_iter = ::core::clone::Clone::clone(&self.__iter);
                #(
                    #inner_unique_capture_idents =
                        ::core::clone::Clone::clone(&self.__capture.#inner_unique_capture_idents);
                )*
            } else {
                self.__iter = cloned_iter;
                #(
                    self.__capture.#inner_unique_capture_idents = #inner_unique_capture_idents;
                )*
                return true;
            }
        }
        if !self.#inner_matches_ident() {
            self.__iter = cloned_iter;
            #(
                self.__capture.#inner_unique_capture_idents = #inner_unique_capture_idents;
            )*
        }
    }
}

fn none_hi_test(
    inner_matches_ident: &Ident,
    inner_unique_capture_idents: &BTreeSet<Ident>,
) -> TokenStream {
    quote! {
        let mut cloned_iter = ::core::clone::Clone::clone(&self.__iter);
        #(
            let mut #inner_unique_capture_idents =
                ::core::clone::Clone::clone(&self.__capture.#inner_unique_capture_idents);
        )*
        while self.#inner_matches_ident() {
            cloned_iter = ::core::clone::Clone::clone(&self.__iter);
            #(
                #inner_unique_capture_idents =
                    ::core::clone::Clone::clone(&self.__capture.#inner_unique_capture_idents);
            )*
        }
        self.__iter = cloned_iter;
        #(
            self.__capture.#inner_unique_capture_idents = #inner_unique_capture_idents;
        )*
    }
}

pub fn expand_block(
    repeat: &Repeat,
    inner_matches_ident: &Ident,
    inner_unique_capture_idents: &BTreeSet<Ident>,
) -> TokenStream {
    let mut output = bounds_decl(repeat);
    output.extend(lo_test(inner_matches_ident));
    output.extend(
        if matches!(
            repeat,
            Repeat::InInclusive {
                hi_excluded: Some(_),
                ..
            }
        ) {
            some_hi_test(inner_matches_ident, inner_unique_capture_idents)
        } else {
            none_hi_test(inner_matches_ident, inner_unique_capture_idents)
        },
    );
    output.extend(quote! { true });
    output
}
