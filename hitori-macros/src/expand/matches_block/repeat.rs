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

fn cache_update_restore(inner_capture_idents: &BTreeSet<Ident>) -> [TokenStream; 3] {
    let other_vars = cache::OtherVars::unique_in(inner_capture_idents);
    let capture_vars = cache::CaptureVars::new(inner_capture_idents);
    let mut cache = other_vars.cache();
    cache.extend(capture_vars.cache());
    let mut update = other_vars.update();
    update.extend(capture_vars.update());
    let mut restore = other_vars.restore();
    restore.extend(capture_vars.restore());
    [cache, update, restore]
}

fn some_hi_test(
    inner_matches_ident: &Ident,
    inner_capture_idents: &BTreeSet<Ident>,
) -> TokenStream {
    let [cache, cache_update, cache_restore] = cache_update_restore(inner_capture_idents);
    quote! {
        if lo + 1 == hi {
            return true;
        }
        #cache
        for _ in lo..(hi - 1) {
            if self.#inner_matches_ident() {
                #cache_update
            } else {
                #cache_restore
                return true;
            }
        }
        if !self.#inner_matches_ident() {
            #cache_restore
        }
    }
}

fn none_hi_test(
    inner_matches_ident: &Ident,
    inner_capture_idents: &BTreeSet<Ident>,
) -> TokenStream {
    let [cache, cache_update, cache_restore] = cache_update_restore(inner_capture_idents);
    quote! {
        #cache
        while self.#inner_matches_ident() {
            #cache_update
        }
        #cache_restore
    }
}

pub fn expand_block(
    repeat: &Repeat,
    inner_matches_ident: &Ident,
    inner_capture_idents: &BTreeSet<Ident>,
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
            some_hi_test(inner_matches_ident, inner_capture_idents)
        } else {
            none_hi_test(inner_matches_ident, inner_capture_idents)
        },
    );
    output.extend(quote! { true });
    output
}
