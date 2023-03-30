use super::capture;
use crate::utils::{
    find_unique_hitori_attr, lifetimes_into_punctuated_unit_refs, remove_generic_params_bounds,
    take_hitori_attrs, FindHitoriAttrs,
};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens as _};
use std::collections::BTreeSet;
use syn::{
    parse::Parse, punctuated::Punctuated, Attribute, Expr, ExprRange, GenericParam, Token, Type,
    WhereClause,
};

fn partial_impl_wrapper(
    is_mut: bool,
    capture_vecs_ident: &Ident,
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
            #iter_ident: core::iter::Iterator<Item = (#idx_ty, #ch_ty)> + core::clone::Clone,
        });
        output
    };

    let mut_ = is_mut.then_some(<Token![mut]>::default());

    let mut output = quote! {
       struct #wrapper_ident<#wrapper_params> #where_clause {
           __target: &'a #mut_ #self_ty,
           __capture: #capture_vecs_ident<#idx_ty>,
           __end: #idx_ty,
           __iter: #iter_ident,
           __phantom: core::marker::PhantomData<(#phantom_data_params)>,
       };

       impl<#wrapper_params> core::ops::Deref
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
            impl<#wrapper_params> core::ops::DerefMut
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

enum RepeatInner {
    Question,
    Star,
    Plus,
    Exact(Expr),
    Range(ExprRange),
}

impl Parse for RepeatInner {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(if input.fork().parse::<Token![?]>().is_ok() {
            Self::Question
        } else if input.fork().parse::<Token![*]>().is_ok() {
            Self::Star
        } else if input.fork().parse::<Token![+]>().is_ok() {
            Self::Plus
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

struct Repeat {
    inner: RepeatInner,
    capture_field_idents: Vec<Ident>,
}

type Group = Punctuated<Expr, Token![,]>;

enum TreeInner {
    All(Group),
    Any(Group),
    Test(Expr),
}

struct Tree {
    inner: TreeInner,
    repeat: Option<Repeat>,
    capture_field_idents: Vec<Ident>,
}

fn collect_capture_field_idents(attrs: &[Attribute]) -> syn::Result<Vec<Ident>> {
    FindHitoriAttrs::new(attrs, "capture")
        .map(|(_index, result)| result)
        .collect()
}

impl TryFrom<Expr> for Tree {
    type Error = syn::Error;

    fn try_from(mut expr: Expr) -> Result<Self, Self::Error> {
        let attrs = take_hitori_attrs(&mut expr);
        let found_repeat_inner = find_unique_hitori_attr(&attrs, "repeat")?;
        let attrs_in_repeat = if let Some((index, _args)) = &found_repeat_inner {
            &attrs[index + 1..]
        } else {
            &attrs
        };
        let repeat = found_repeat_inner
            .map(|(index, inner)| {
                collect_capture_field_idents(&attrs[..index]).map(|capture_field_idents| Repeat {
                    inner,
                    capture_field_idents,
                })
            })
            .transpose()?;
        let capture_field_idents = collect_capture_field_idents(&attrs_in_repeat)?;
        Ok(Self {
            inner: match expr {
                Expr::Tuple(tuple) => TreeInner::All(tuple.elems),
                Expr::Array(arr) => TreeInner::Any(arr.elems),
                _ => TreeInner::Test(expr),
            },
            repeat,
            capture_field_idents,
        })
    }
}

#[derive(Default)]
struct State {
    subexpr_index: usize,
    last_subexpr_matches_ident: Option<Ident>,
    capture_fields: BTreeSet<capture::Field>,
}

impl State {
    fn tree(&mut self, tree: Tree) -> syn::Result<TokenStream> {
        todo!()
    }
}

pub struct Input<'a> {
    pub hitori_ident: &'a Ident,
    pub is_mut: bool,
    pub capture_options_ident: &'a Ident,
    pub capture_vecs_ident: &'a Ident,
    pub self_ty: &'a Type,
    pub iter_ident: &'a Ident,
    pub idx_ty: &'a Type,
    pub ch_ty: &'a Type,
    pub expr: Expr,
    pub wrapper_ident: &'a Ident,
    pub generic_params: Punctuated<GenericParam, Token![,]>,
    pub where_clause: Option<&'a WhereClause>,
}

impl<'a> Input<'a> {
    pub fn expand(self) -> syn::Result<(TokenStream, BTreeSet<capture::Field>)> {
        let mut st = State::default();
        let impl_wrapper_block = st.tree(self.expr.try_into()?)?;
        let tokens = if impl_wrapper_block.is_empty() {
            quote! {
                (core::option::Option::Some(..start), core::default::Default::default())
            }
        } else {
            let partial_impl_wrapper = partial_impl_wrapper(
                self.is_mut,
                self.capture_vecs_ident,
                self.self_ty,
                self.iter_ident,
                self.idx_ty,
                self.ch_ty,
                self.wrapper_ident,
                self.generic_params,
                self.where_clause,
            );
            let capture_vecs = capture::vecs(
                self.hitori_ident,
                self.capture_vecs_ident,
                self.capture_options_ident,
                st.capture_fields.iter(),
            );
            let last_subexpr_matches_ident = st.last_subexpr_matches_ident;
            let wrapper_ident = self.wrapper_ident;
            quote! {
                #partial_impl_wrapper {
                    #impl_wrapper_block
                }
                #capture_vecs
                let wrapper = #wrapper_ident {
                    __target: self,
                    __capture: core::default::Default::default(),
                    __end: start,
                    __iter: core::iter::IntoIterator::into_iter(iter),
                    __phantom: core::marker::PhantomData,
                };
                if wrapper.#last_subexpr_matches_ident() {
                    core::option::Option::Some(
                        (..wrapper.__end, wrapper.__capture.into_options())
                    )
                } else {
                    core::option::Option::None
                }
            }
        };
        Ok((tokens, st.capture_fields))
    }
}
