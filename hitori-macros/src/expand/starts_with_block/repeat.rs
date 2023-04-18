use super::cache;
use crate::parse::repeat::Repeat;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::collections::BTreeSet;

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

fn lo_test(inner_matches_ident: &Ident, inner_capture_idents: &BTreeSet<Ident>) -> TokenStream {
    let capture = cache::Capture::new(inner_capture_idents);
    let capture_cache = capture.cache();
    let capture_restore = capture.restore();
    quote! {
        #capture_cache
        for _ in 0..lo {
            if !self.#inner_matches_ident() {
                #capture_restore
                return false;
            }
        }
    }
}

fn vars_cache_update_restore(inner_capture_idents: &BTreeSet<Ident>) -> [TokenStream; 3] {
    let vars = cache::Vars::unique_in(inner_capture_idents);
    [vars.cache(), vars.update(), vars.restore()]
}

fn some_hi_test(
    inner_matches_ident: &Ident,
    [vars_cache, vars_update, vars_restore]: &[TokenStream; 3],
) -> TokenStream {
    quote! {
        if lo + 1 == hi {
            return true;
        }
        #vars_cache
        for _ in lo + 2..hi {
            if self.#inner_matches_ident() {
                #vars_update
            } else {
                #vars_restore
                return true;
            }
        }
        if !self.#inner_matches_ident() {
            #vars_restore
        }
    }
}

fn none_hi_test(
    inner_matches_ident: &Ident,
    [vars_cache, vars_update, vars_restore]: &[TokenStream; 3],
) -> TokenStream {
    quote! {
        #vars_cache
        while self.#inner_matches_ident() {
            #vars_update
        }
        #vars_restore
    }
}

pub fn expand_block(
    repeat: &Repeat,
    inner_matches_ident: &Ident,
    inner_capture_idents: &BTreeSet<Ident>,
) -> TokenStream {
    let mut output = bounds_decl(repeat);
    output.extend(lo_test(inner_matches_ident, inner_capture_idents));
    if let Repeat::InInclusive { hi_excluded, .. } = repeat {
        let vars_streams = vars_cache_update_restore(inner_capture_idents);
        output.extend(if hi_excluded.is_some() {
            some_hi_test(inner_matches_ident, &vars_streams)
        } else {
            none_hi_test(inner_matches_ident, &vars_streams)
        });
    }
    output.extend(quote! { true });
    output
}
