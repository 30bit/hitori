use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens as _};
use std::{collections::BTreeSet, fmt::Write as _};
use syn::{parse::Parse, Expr, ExprRange};

pub enum Repeat {
    Exact(Expr),
    Range(ExprRange),
}

impl Parse for Repeat {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Expr>().and_then(|expr| match expr {
            Expr::Range(range) => {
                if range.from.is_some() {
                    Ok(Self::Range(range))
                } else {
                    let mut expected = String::with_capacity(3);
                    expected.push('0');
                    expected
                        .write_fmt(format_args!("{}", range.limits.to_token_stream()))
                        .unwrap();
                    expected
                        .write_fmt(format_args!("{}", range.to.to_token_stream()))
                        .unwrap();
                    Err(syn::Error::new_spanned(
                        range,
                        format!("repetition range must have a lower bound (e.g. `{expected}`)",),
                    ))
                }
            }
            expr => Ok(Self::Exact(expr)),
        })
    }
}

fn lit_partial_exact(
    inner_matches_ident: &Ident,
    inner_unique_capture_idents: &BTreeSet<Ident>,
    count: usize,
) -> TokenStream {
    let mut tokens = if count != 0 {
        quote! {
            let cloned_iter = ::core::clone::Clone::clone(&self.__iter);
        }
    } else {
        TokenStream::new()
    };
    if count > 1 {
        tokens.extend(quote! {
            #(
                let #inner_unique_capture_idents =
                    ::core::clone::Clone::clone(&self.__capture.#inner_unique_capture_idents);
            )*
        });
    }
    if count != 0 {
        tokens.extend(quote! {
            if !self.#inner_matches_ident() {
                self.__iter = cloned_iter;
                return false;
            }
        });
    }
    if count > 1 {
        tokens.extend(quote! {
            for _ in 1..#count {
                if !self.#inner_matches_ident() {
                    self.__iter = cloned_iter;
                    #(
                        self.__capture.#inner_unique_capture_idents = #inner_unique_capture_idents;
                    )*
                    return false;
                }
            }
        })
    }
    tokens
}

fn lit_in_non_empty(
    inner_matches_ident: &Ident,
    start: usize,
    end: usize,
    inclusive: bool,
) -> TokenStream {
    let mut tokens = if let Some(for_end) = match inclusive {
        true if start != end => Some(end),
        false if start + 1 != end => Some(end - 1),
        _ => None,
    } {
        quote! {
            let mut cloned_iter = ::core::clone::Clone::clone(&self.__iter);
            for _ in #start..#for_end {
                if !self.#inner_matches_ident() {
                    self.__iter = cloned_iter;
                    return true;
                }
                cloned_iter = ::core::clone::Clone::clone(&self.__iter);
            }
        }
    } else {
        quote! {
            let cloned_iter = ::core::clone::Clone::clone(&self.__iter);
        }
    };
    tokens.extend(quote! {
        if !self.#inner_matches_ident() {
            self.__iter = cloned_iter;
        }
        true
    });
    tokens
}

pub fn lit_exact(
    inner_matches_ident: &Ident,
    inner_unique_capture_idents: &BTreeSet<Ident>,
    count: usize,
) -> TokenStream {
    let mut tokens = lit_partial_exact(inner_matches_ident, inner_unique_capture_idents, count);
    tokens.extend(quote! { true });
    tokens
}

pub fn lit_non_empty_range_inclusive(
    inner_matches_ident: &Ident,
    inner_unique_capture_idents: &BTreeSet<Ident>,
    start: usize,
    end: usize,
) -> TokenStream {
    assert!(start <= end, "bug");
    let mut tokens = lit_partial_exact(inner_matches_ident, inner_unique_capture_idents, start);
    tokens.extend(lit_in_non_empty(inner_matches_ident, start, end, true));
    tokens
}

pub fn lit_non_empty_range(
    inner_matches_ident: &Ident,
    inner_unique_capture_idents: &BTreeSet<Ident>,
    start: usize,
    end: usize,
) -> TokenStream {
    assert!(start < end, "bug");
    let mut tokens = lit_partial_exact(inner_matches_ident, inner_unique_capture_idents, start);
    tokens.extend(lit_in_non_empty(inner_matches_ident, start, end, false));
    tokens
}

pub fn lit_range_from(
    inner_matches_ident: &Ident,
    inner_unique_capture_idents: &BTreeSet<Ident>,
    start: usize,
) -> TokenStream {
    let mut block = lit_partial_exact(inner_matches_ident, inner_unique_capture_idents, start);
    block.extend(quote! {
        let mut cloned_iter = ::core::clone::Clone::clone(&self.__iter);
        while self.#inner_matches_ident() {
            cloned_iter = ::core::clone::Clone::clone(&self.__iter);
        }
        self.__iter = cloned_iter;
        true
    });
    block
}
