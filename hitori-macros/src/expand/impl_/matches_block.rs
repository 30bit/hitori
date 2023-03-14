use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::ops::AddAssign;
use syn::{parse::Parse, punctuated::Punctuated, Expr, ExprRange, Token};

use crate::utils::{collect_hitori_attrs, find_unique_hitori_attr, take_hitori_attrs};

enum Repeat {
    Star,
    Plus,
    Question,
    Exact(Expr),
    Range(ExprRange),
}

impl Parse for Repeat {
    #[allow(unreachable_code, unused_variables)]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        return Err(input.error("repetitions are not implemented yet"));
        Ok(if input.fork().parse::<Token![*]>().is_ok() {
            Self::Star
        } else if input.fork().parse::<Token![+]>().is_ok() {
            Self::Plus
        } else if input.fork().parse::<Token![?]>().is_ok() {
            Self::Question
        } else {
            match input.parse::<Expr>() {
                Ok(Expr::Range(range)) => Self::Range(range),
                Ok(expr) => Self::Exact(expr),
                Err(expr_err) => {
                    let mut err = syn::Error::new_spanned(
                        TokenStream::new(),
                        "not a `*`, `+`, `?` or expression",
                    );
                    err.combine(expr_err);
                    return Err(err);
                }
            }
        })
    }
}

type Group<'a> = &'a mut Punctuated<Expr, Token![,]>;

enum TreeInner<'a> {
    All(Group<'a>),
    Any(Group<'a>),
    Test(&'a Expr),
}

struct Tree<'a> {
    inner: TreeInner<'a>,
    #[allow(dead_code)]
    repeat: Option<Repeat>,
    capture: Vec<Ident>, // used in place for calls
}

impl<'a> TryFrom<&'a mut Expr> for Tree<'a> {
    type Error = syn::Error;

    fn try_from(expr: &'a mut Expr) -> syn::Result<Self> {
        let attrs = take_hitori_attrs(expr);
        let repeat = find_unique_hitori_attr(&attrs, "repeat")?;
        let capture = collect_hitori_attrs(&attrs, "capture")?;
        Ok(Self {
            inner: match expr {
                Expr::Tuple(tuple) => TreeInner::All(&mut tuple.elems),
                Expr::Array(arr) => TreeInner::Any(&mut arr.elems),
                _ => TreeInner::Test(expr),
            },
            repeat,
            capture,
        })
    }
}

#[derive(Default)]
pub struct Output {
    pub tokens: TokenStream,
    pub capture: Vec<Ident>, // used for #[hitori::add_define]
}

impl AddAssign for Output {
    fn add_assign(&mut self, rhs: Self) {
        self.tokens.extend(rhs.tokens);
        self.capture.extend(rhs.capture);
    }
}

impl TryFrom<&mut Expr> for Output {
    type Error = syn::Error;

    fn try_from(expr: &mut Expr) -> syn::Result<Self> {
        Tree::try_from(expr).and_then(TryInto::try_into)
    }
}

impl<'a> TryFrom<Tree<'a>> for Output {
    type Error = syn::Error;

    fn try_from(tree: Tree<'a>) -> syn::Result<Self> {
        expand_tree(tree, NeedNext(true)).map(|(mut expanded, _)| {
            expanded.capture.sort_unstable();
            expanded.capture.dedup();
            if expanded.tokens.is_empty() {
                expanded.tokens = quote! { Ok(Some(..start)) };
            } else {
                let body = expanded.tokens;
                expanded.tokens = quote! {
                    let mut iter = iter.into_iter(); let mut next: I::Item; let mut end;
                    #body
                    Ok(Some(..start))
                };
            }
            expanded
        })
    }
}

struct NeedNext(bool);

fn expand_tree_inner_all(
    group: Group<'_>,
    mut need_next: NeedNext,
) -> syn::Result<(Output, NeedNext)> {
    let mut output = Output::default();
    for expr in group {
        let expanded = expand_tree(expr.try_into()?, need_next)?;
        output += expanded.0;
        need_next = expanded.1;
    }
    Ok((output, need_next))
}

fn expand_tree(
    Tree {
        inner, mut capture, ..
    }: Tree<'_>,
    need_next: NeedNext,
) -> syn::Result<(Output, NeedNext)> {
    let (mut output, need_next) = expand_tree_inner(inner, need_next)?;
    let has_no_tests = output.tokens.is_empty();

    if !capture.is_empty() {
        if has_no_tests {
            output.tokens.extend(quote! { end = start.clone(); });
        }
        for f in &capture[..capture.len() - 1] {
            output
                .tokens
                .extend(quote! { capture.#f(start.clone()..end.clone())?; });
        }
        let f = capture.last().unwrap();
        output
            .tokens
            .extend(quote! { capture.#f(start..end.clone())?; });
    }

    if !(has_no_tests && capture.is_empty()) {
        output.tokens.extend(quote! { start = end; });
    }

    capture.append(&mut output.capture);
    output.capture = capture;

    Ok((output, need_next))
}

fn expand_tree_inner(inner: TreeInner, need_next: NeedNext) -> syn::Result<(Output, NeedNext)> {
    Ok(match inner {
        TreeInner::All(group) => {
            let mut expanded = expand_tree_inner_all(group, need_next)?;
            let output_tokens = &expanded.0.tokens;
            if output_tokens.is_empty() {
            } else {
                // TODO: don't clone start if there are is no capture in the group
                expanded.0.tokens =
                    quote! {end = { let mut start = start.clone(); #output_tokens start };};
            }
            expanded
        }
        TreeInner::Any(group) => {
            return Err(syn::Error::new_spanned(
                group,
                "any-patterns are not implemented yet",
            ))
        }
        TreeInner::Test(expr) => {
            let mut tokens = if need_next.0 {
                quote! {
                    if let Some(x) = iter.next() { next = x; } else { return Ok(None); }
                }
            } else {
                TokenStream::default()
            };
            tokens.extend(quote! {
                end = if (#expr)(next.1) { next.0 } else { return Ok(None); };
            });
            (
                Output {
                    tokens,
                    capture: vec![],
                },
                NeedNext(true),
            )
        }
    })
}
