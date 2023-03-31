use crate::utils::{
    eq_by_fmt, expr_as_lit_int, find_le_one_hitori_attr, hitori_attr_ident_eq_str,
    lifetimes_into_punctuated_unit_refs, remove_generic_params_bounds, unique_ident,
};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::{collections::BTreeSet, fmt::Write as _, ops::Bound};
use syn::{
    parse::Parse, punctuated::Punctuated, Attribute, Expr, ExprRange, GenericParam, RangeLimits,
    Token, Type, WhereClause,
};

fn partial_impl_wrapper(
    is_mut: bool,
    capture_ident: &Ident,
    self_ty: &Type,
    iter_ident: &Ident,
    idx_ty: &Type,
    ch_ty: &Type,
    wrapper_ident: &Ident,
    mut generic_params: Punctuated<GenericParam, Token![,]>,
    where_clause: Option<&WhereClause>,
) -> TokenStream {
    let wrapper_params = quote! { 'a, #iter_ident, #generic_params };

    let mut phantom_data_params = lifetimes_into_punctuated_unit_refs(
        generic_params
            .iter()
            .take_while(|param| matches!(param, GenericParam::Lifetime(_)))
            .map(|param| match param {
                GenericParam::Lifetime(l) => l,
                _ => unreachable!(),
            }),
    );

    remove_generic_params_bounds(&mut generic_params);
    let no_bounds_wrapper_params = quote! { 'a, #iter_ident, #generic_params };

    for pair in generic_params.pairs() {
        if !matches!(pair.value(), GenericParam::Const(_)) {
            pair.to_tokens(&mut phantom_data_params);
        }
    }

    let where_clause = {
        let mut output = where_clause.as_ref().map_or_else(
            || quote! { where },
            |existing| {
                if existing.predicates.empty_or_trailing() {
                    quote! { #where_clause }
                } else {
                    quote! { #where_clause, }
                }
            },
        );
        output.extend(quote! {
            #iter_ident: ::core::iter::Iterator<Item = (#idx_ty, #ch_ty)> + ::core::clone::Clone,
        });
        output
    };

    let mut_ = is_mut.then_some(<Token![mut]>::default());

    let mut output = quote! {
       struct #wrapper_ident<#wrapper_params> #where_clause {
           __target: &'a #mut_ #self_ty,
           __capture: #capture_ident<#idx_ty>,
           __end: #idx_ty,
           __iter: #iter_ident,
           __phantom: ::core::marker::PhantomData<(#phantom_data_params)>,
       };

       impl<#wrapper_params> ::core::ops::Deref
       for #wrapper_ident<#no_bounds_wrapper_params>
       #where_clause
       {
           type Target = #self_ty;

           fn deref(&self) -> &Self::Target {
               self.__target
           }
       }
    };

    if is_mut {
        output.extend(quote! {
            impl<#wrapper_params> ::core::ops::DerefMut
            for #wrapper_ident<#no_bounds_wrapper_params>
            #where_clause
            {
                fn deref_mut(&mut self) -> &Self::Target {
                    self.__target
                }
            }
        })
    }

    output.extend(quote! {
        impl<#wrapper_params> #wrapper_ident<#no_bounds_wrapper_params> #where_clause
    });

    output
}

enum Repeat {
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

fn repeat_question_block(inner_matches_ident: &Ident) -> TokenStream {
    quote! {
        let cloned_iter = ::core::clone::Clone::clone(&self.__iter);
        if !self.#inner_matches_ident() {
            self.__iter = cloned_iter;
        }
        true
    }
}

fn repeat_star_block(inner_matches_ident: &Ident) -> TokenStream {
    quote! {
        let mut cloned_iter = ::core::clone::Clone::clone(&self.__iter);
        while self.#inner_matches_ident() {
            cloned_iter = ::core::clone::Clone::clone(&self.__iter);
        }
        self.__iter = cloned_iter;
        true
    }
}

fn repeat_plus_block(inner_matches_ident: &Ident) -> TokenStream {
    quote! {
        let mut cloned_iter = ::core::clone::Clone::clone(&self.__iter);
        if self.#inner_matches_ident() {
            cloned_iter = ::core::clone::Clone::clone(&self.__iter);
        } else {
            self.__iter = cloned_iter;
            return false;
        }
        while self.#inner_matches_ident() {
            cloned_iter = ::core::clone::Clone::clone(&self.__iter);
        }
        self.__iter = cloned_iter;
        true
    }
}

fn repeat_lit_exact_block(
    inner_matches_ident: &Ident,
    count: i128,
    unique_capture_idents: &BTreeSet<Ident>,
) -> Result<TokenStream, &'static str> {
    if count < 0 {
        return Err("expected non-negative repeat");
    } else if count == 0 {
        return Ok(quote! { true });
    }

    let (cache_tokens, for_tokens) = if count > 1 {
        let count = count as usize;
        (
            Some(quote! {
                #(
                    let #unique_capture_idents =
                        ::core::clone::Clone::clone(&self.__capture.#unique_capture_idents);
                )*
            }),
            Some(quote! {
                for _ in 1..#count {
                    if !self.#inner_matches_ident() {
                        #(
                            self.__capture.#unique_capture_idents = #unique_capture_idents;
                        )*
                        return false;
                    }
                }
            }),
        )
    } else {
        (None, None)
    };

    Ok(quote! {
        #cache_tokens
        if !self.#inner_matches_ident() {
            return false;
        }
        #for_tokens
        true
    })
}

/*

fn repeat_range_block(
    inner_matches_ident: &Ident,
    range: &ExprRange,
    lit_from: Option<i128>,
    lit_to: Option<i128>,
    unique_capture_idents: &BTreeSet<Ident>,
) -> TokenStream {
    let has_zero_start = eq_by_fmt(range.from.as_ref().unwrap(), quote! { 0 });
    let range_ident = unique_ident(unique_capture_idents.iter(), "range".into());
    let mut block = quote! {
        let #range_ident = #range;
    };

    if !has_zero_start {
        if matches!(&range.limits, RangeLimits::HalfOpen(_)) {
            block.extend(quote! {
                if #range_ident.start < 0  { return false; }
            });
        } else {
            block.extend(quote! {
                if #range_ident.start() < &0  { return false; }
            });
        }
    }


    quote! { todo!() }
}

 */

fn repeat_lit_range_block(
    inner_matches_ident: &Ident,
    from: i128,
    to: Bound<i128>,
    unique_capture_idents: &BTreeSet<Ident>,
) -> Result<TokenStream, &'static str> {
    /*
        let range_ident = unique_ident(&unique_capture_idents, "range");
        let mut block = quote! {
            let #range_ident = #range;
            // Either `star < 0t` or `::core::ops::RangeInclusive::start(&#range_ident) < &0`
            if #range_ident.start < 0 { return false };
        };
        if range.to.is_some() {
            // Either Range or RangeInclusive
            block.extend(quote! {
                if ::core::ops::range::Range::is_empty(&#range_ident) {
                    return false;
                }
            });
        }
        let has_lower_bound = eq_by_fmt(range.from.as_ref().unwrap(), 0);
        let mut inner_matches_owned_ident;
        let inner_matches_ident = &self.last_subexpr_matches_ident;
        if has_lower_bound {
            inner_matches_owned_ident = self.last_subexpr_matches_ident.clone();
            inner_matches_ident = &inner_matches_owned_ident;
            block.extend(&repeat_exact_block(inner_matches_ident, &quote { #range_ident.start }, ));
        } else if if range.to.is_some() {
    `       block.extend(&quote! {
                #(
                let #unique_capture_idents =
                    ::core::clone::Clone::clone(&self.__capture.#unique_capture_idents);
                )*
            });
        }

        if let Some(upper_bound) = range.to.as_ref() {
            block.extend(quote! {
                let mut cloned_iter = ::core::Clone::clone(&self.__iter);
                // Don't clone last iter
                for _ in range {
                    if self.#inner_matches_ident() {
                         cloned_iter = :core::Clone::clone(&self.__iter);
                    } else {
                        self.__iter = cloned_iter;
                        return true;
                    }
                }
                true
            });
        } else {
            block.extend(&repeat_star_block(inner_matches_ident));
            block
        }
        */
    todo!()
}

enum HitoriAttribute {
    Repeat(Repeat),
    Capture(Punctuated<Ident, Token![,]>),
}

impl HitoriAttribute {
    fn find(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
        match find_le_one_hitori_attr(attrs) {
            Ok(Some(attr)) => Ok(Some(if hitori_attr_ident_eq_str(attr, "capture") {
                Self::Capture(attr.parse_args_with(Punctuated::parse_terminated)?)
            } else {
                Self::Repeat(attr.parse_args()?)
            })),
            Ok(None) => Ok(None),
            Err([first, second]) => {
                let is_first_capture = hitori_attr_ident_eq_str(first, "capture");
                let is_second_capture = hitori_attr_ident_eq_str(second, "capture");
                Err(syn::Error::new_spanned(
                    first,
                    if is_first_capture {
                        if is_second_capture {
                            "to capture group into multiple destinations, \
                            use single `capture` attribute and \
                            add each identifier to its argument list \
                            (e.g. `#[hitori::capture(a, b, c)] _group`)"
                        } else {
                            "to capture a repetition \
                            surround it by parenthesis or square brackets \
                            (e.g. `#[hitori::capture(cap)] ( #[hitori::repeat(*)] _group )`)"
                        }
                    } else if is_second_capture {
                        "to repeat a captured group \
                        surround it by parenthesis or square brackets \
                        (e.g. `#[hitori::repeat(+)] ( #[hitori::capture(cap)] _group )`)"
                    } else {
                        "to repeat a repetition, \
                        surround it by parenthesis or square brackets \
                        (e.g. `#[hitori::repeat(+)] ( #[hitori::repeat(?)] _group )`)"
                    },
                ))
            }
        }
    }
}

enum Group<'a> {
    Paren(&'a Expr),
    All(&'a Punctuated<Expr, Token![,]>),
    Any(&'a Punctuated<Expr, Token![,]>),
}

enum Tree<'a> {
    Group(Group<'a>, Option<HitoriAttribute>),
    Test(&'a Expr),
}

impl<'a> TryFrom<&'a Expr> for Tree<'a> {
    type Error = syn::Error;

    fn try_from(expr: &'a Expr) -> syn::Result<Self> {
        Ok(match &expr {
            Expr::Tuple(tuple) => Tree::Group(
                Group::All(&tuple.elems),
                HitoriAttribute::find(&tuple.attrs)?,
            ),
            Expr::Array(arr) => {
                Tree::Group(Group::Any(&arr.elems), HitoriAttribute::find(&arr.attrs)?)
            }
            Expr::Paren(paren) => Tree::Group(
                Group::Paren(&paren.expr),
                HitoriAttribute::find(&paren.attrs)?,
            ),
            _ => Tree::Test(expr),
        })
    }
}

#[derive(Default)]
struct State {
    impl_wrapper_block: TokenStream,
    next_subexpr_index: usize,
    last_subexpr_matches_ident: Option<Ident>,
}

impl State {
    fn set_next_subexpr(&mut self) {
        self.last_subexpr_matches_ident = Some(format_ident!(
            "__subexpr{}_matches",
            self.next_subexpr_index
        ));
        self.next_subexpr_index += 1;
    }

    fn push_empty_subexpr_matches(&mut self, matches: bool) {
        self.set_next_subexpr();
        let ident = &self.last_subexpr_matches_ident;
        let mut tokens = quote! {
            #[inline(always)]
            fn #ident(&mut self) -> bool
        };
        tokens.extend(if matches {
            quote! {{ true }}
        } else {
            quote! {{ false }}
        });

        self.impl_wrapper_block.extend(tokens);
    }

    fn push_subexpr_matches(&mut self, block: &TokenStream) {
        self.set_next_subexpr();
        let ident = &self.last_subexpr_matches_ident;
        self.impl_wrapper_block.extend(quote! {
            fn #ident(&mut self) -> bool { #block }
        });
    }

    fn push_group_all(
        &mut self,
        all: &Punctuated<Expr, Token![,]>,
    ) -> syn::Result<BTreeSet<Ident>> {
        let mut unique_capture_idents = BTreeSet::new();
        if all.is_empty() {
            self.push_empty_subexpr_matches(true);
            return Ok(unique_capture_idents);
        }
        let mut block = TokenStream::new();

        let mut branch = |expr: &Expr, is_last: bool| -> syn::Result<()> {
            let mut branch_unique_capture_idents = self.push_tree(expr.try_into()?)?;
            let branch_matches_ident = &self.last_subexpr_matches_ident;
            if unique_capture_idents.is_empty() {
                if !is_last {
                    block.extend(quote! {
                        #(
                            let #branch_unique_capture_idents =
                                ::core::clone::Clone::clone(&self.__capture.#branch_unique_capture_idents);
                        )*
                    })
                }
                block.extend(quote! {
                    if !self.#branch_matches_ident() { return false; }
                });
                unique_capture_idents = branch_unique_capture_idents;
            } else {
                let undo = quote! {
                    if !self.#branch_matches_ident() {
                        #(
                            self.__capture.#unique_capture_idents = #unique_capture_idents;
                        )*
                        return false;
                    }
                };
                if is_last {
                    unique_capture_idents.append(&mut branch_unique_capture_idents);
                } else {
                    for ident in branch_unique_capture_idents {
                        if unique_capture_idents.insert(ident.clone()) {
                            block.extend(quote! {
                                let #ident = ::core::clone::Clone::clone(&self.__capture.#ident);
                            });
                        }
                    }
                }
                block.extend(undo);
            }
            Ok(())
        };

        for expr in all.iter().take(all.len() - 1) {
            branch(expr, false)?;
        }
        branch(all.last().unwrap(), true)?;
        block.extend(quote! { true });
        self.push_subexpr_matches(&block);
        Ok(unique_capture_idents)
    }

    fn push_group_any(
        &mut self,
        any: &Punctuated<Expr, Token![,]>,
    ) -> syn::Result<BTreeSet<Ident>> {
        let mut unique_capture_idents = BTreeSet::new();
        if any.is_empty() {
            self.push_empty_subexpr_matches(false);
            return Ok(unique_capture_idents);
        }
        let mut block = (any.len() > 1)
            .then(|| {
                quote! {
                    let cloned_iter = ::core::clone::Clone::clone(&self.__iter);
                }
            })
            .unwrap_or_default();

        let mut branch = |expr: &Expr, reset_iter: &TokenStream| -> syn::Result<()> {
            unique_capture_idents.append(&mut self.push_tree(expr.try_into()?)?);
            let branch_matches_ident = &self.last_subexpr_matches_ident;
            block.extend(quote! {
                if self.#branch_matches_ident() {
                    return true;
                } else {
                    #reset_iter
                }
            });
            Ok(())
        };

        if any.len() > 2 {
            let reset_iter = quote! {
                self.__iter = ::core::clone::Clone::clone(&cloned_iter);
            };
            for expr in any.iter().take(any.len() - 2) {
                branch(expr, &reset_iter)?;
            }
        }

        if any.len() > 1 {
            branch(
                &any[any.len() - 2],
                &quote! {
                    self.__iter = cloned_iter;
                },
            )?;
        }

        branch(any.last().unwrap(), &quote! { false })?;

        self.push_subexpr_matches(&block);
        Ok(unique_capture_idents)
    }

    fn push_group(&mut self, group: Group) -> syn::Result<BTreeSet<Ident>> {
        match group {
            Group::Paren(paren) => self.push_tree(paren.try_into()?),
            Group::All(all) => self.push_group_all(all),
            Group::Any(any) => self.push_group_any(any),
        }
    }

    fn push_repeated_group(
        &mut self,
        group: Group,
        repeat: Repeat,
    ) -> syn::Result<BTreeSet<Ident>> {
        let unique_capture_idents = self.push_group(group)?;
        let inner_matches_ident = self.last_subexpr_matches_ident.as_ref().unwrap();
        let block = match &repeat {
            Repeat::Exact(count) => {
                let lit_count = expr_as_lit_int(count);
                match lit_count {
                    Some(count) => {
                        repeat_lit_exact_block(inner_matches_ident, count, &unique_capture_idents)
                            .map_err(|msg| syn::Error::new_spanned(count, msg))?
                    }
                    None => {
                        return Err(syn::Error::new_spanned(
                            count,
                            "non-literal exact repetitions are not implemented yet",
                        ))
                    }
                }
            }
            Repeat::Range(range) => {
                let lit_expr = |expr: &Option<_>| -> syn::Result<_> {
                    Ok(match expr.as_deref() {
                        Some(expr) => Some(expr_as_lit_int(expr).ok_or_else(|| {
                            syn::Error::new_spanned(
                                expr,
                                "non-literal range repetitions are not implemented yet",
                            )
                        })?),
                        None => None,
                    })
                };
                let lit_from = lit_expr(&range.from)?.unwrap();
                let lit_to = lit_expr(&range.to)?;
                let inclusive = matches!(&range.limits, RangeLimits::Closed(_));
                match (inclusive, lit_from, lit_to) {
                    (true, 0, Some(1)) | (false, 0, Some(2)) => {
                        repeat_question_block(inner_matches_ident)
                    }
                    (_, 0, None) => repeat_star_block(inner_matches_ident),
                    (_, 1, None) => repeat_plus_block(inner_matches_ident),
                    _ => repeat_lit_range_block(
                        inner_matches_ident,
                        lit_from,
                        lit_to
                            .map(|to| {
                                (if inclusive {
                                    Bound::Included
                                } else {
                                    Bound::Excluded
                                })(to)
                            })
                            .unwrap_or(Bound::Unbounded),
                        &unique_capture_idents,
                    )
                    .map_err(|msg| syn::Error::new_spanned(range, msg))?,
                }
            }
        };
        self.push_subexpr_matches(&block);
        Ok(unique_capture_idents)
    }

    fn push_captured_group(
        &mut self,
        group: Group,
        capture_idents: Punctuated<Ident, Token![,]>,
    ) -> syn::Result<BTreeSet<Ident>> {
        let mut unique_capture_idents = self.push_group(group)?;
        if capture_idents.is_empty() {
            return Ok(unique_capture_idents);
        }

        let inner_matches_ident = &self.last_subexpr_matches_ident;
        let capture_idents_xcpt_last_iter = capture_idents.iter().take(capture_idents.len() - 1);
        let last_capture_ident = capture_idents.last().unwrap();

        self.push_subexpr_matches(&quote! {
            let start = ::core::clone::Clone::clone(&self.__end);
            if !self.#inner_matches_ident() {
                return false;
            }
            #(
                self.__capture.#capture_idents_xcpt_last_iter =
                    Some(::core::clone::Clone::clone(&start)..::core::clone::Clone::clone(&self.__end));
            )*
            self.__capture.#last_capture_ident =
                Some(start..::core::clone::Clone::clone(&self.__end));
            true
        });

        unique_capture_idents.extend(capture_idents);
        Ok(unique_capture_idents)
    }

    fn push_test(&mut self, test: &Expr) {
        self.push_subexpr_matches(&quote! {
            let next = if let ::core::option::Option::Some(next) =
                ::core::iter::Iterator::next(&mut self.__iter)
            {
                next
            } else {
                return false;
            };
            if (#test)(next.1) {
                self.__end = next.0;
                true
            } else {
                false
            }
        })
    }

    fn push_tree(&mut self, tree: Tree) -> syn::Result<BTreeSet<Ident>> {
        match tree {
            Tree::Group(group, maybe_attr) => match maybe_attr {
                Some(attr) => match attr {
                    HitoriAttribute::Repeat(repeat) => self.push_repeated_group(group, repeat),
                    HitoriAttribute::Capture(capture_idents) => {
                        self.push_captured_group(group, capture_idents)
                    }
                },
                None => self.push_group(group),
            },
            Tree::Test(test) => {
                self.push_test(test);
                Ok(BTreeSet::new())
            }
        }
    }
}

pub struct Output {
    pub tokens: TokenStream,
    pub unique_capture_idents: BTreeSet<Ident>,
}

pub struct Input<'a> {
    pub hitori_ident: &'a Ident,
    pub is_mut: bool,
    pub capture_ident: &'a Ident,
    pub self_ty: &'a Type,
    pub iter_ident: &'a Ident,
    pub idx_ty: &'a Type,
    pub ch_ty: &'a Type,
    pub expr: &'a Expr,
    pub wrapper_ident: &'a Ident,
    pub generic_params: Punctuated<GenericParam, Token![,]>,
    pub where_clause: Option<&'a WhereClause>,
}

impl<'a> Input<'a> {
    pub fn expand(self) -> syn::Result<Output> {
        let mut st = State::default();
        let unique_capture_idents = st.push_tree(self.expr.try_into()?)?;
        let partial_impl_wrapper = partial_impl_wrapper(
            self.is_mut,
            self.capture_ident,
            self.self_ty,
            self.iter_ident,
            self.idx_ty,
            self.ch_ty,
            self.wrapper_ident,
            self.generic_params,
            self.where_clause,
        );
        let impl_wrapper_block = st.impl_wrapper_block;
        let last_subexpr_matches_ident = st.last_subexpr_matches_ident.unwrap();
        let wrapper_ident = self.wrapper_ident;
        let tokens = quote! {
            #partial_impl_wrapper {
                #impl_wrapper_block
            }
            let mut wrapper = #wrapper_ident {
                __target: self,
                __capture: ::core::default::Default::default(),
                __end: start,
                __iter: ::core::iter::IntoIterator::into_iter(iter),
                __phantom: ::core::marker::PhantomData,
            };
            if wrapper.#last_subexpr_matches_ident() {
                ::core::option::Option::Some((
                    ..wrapper.__end,
                    wrapper.__capture
                ))
            } else {
                ::core::option::Option::None
            }
        };
        Ok(Output {
            tokens,
            unique_capture_idents,
        })
    }
}
