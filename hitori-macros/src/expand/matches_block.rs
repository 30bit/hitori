mod repeat;
mod repeat_new;
mod state;

use crate::utils::{
    find_le_one_hitori_attr, hitori_attr_ident_eq_str, lifetimes_into_punctuated_unit_refs,
    remove_generic_params_bounds,
};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens as _};
use repeat::Repeat;
use state::State;
use std::collections::BTreeSet;
use syn::{punctuated::Punctuated, Attribute, Expr, GenericParam, Token, Type, WhereClause};

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
