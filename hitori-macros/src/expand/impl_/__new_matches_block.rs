use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::ops::AddAssign;
use syn::{
    parse::Parse, punctuated::Punctuated, Expr, ExprRange, GenericParam, Path, Token, WhereClause,
};

use crate::utils::{
    collect_hitori_attrs, find_unique_hitori_attr, remove_generic_params_bounds,
    expand_lifetime_generic_params_into_unit_refs, take_hitori_attrs,
};

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

fn expand_wrapper(
    self_path: &Path,
    generic_params: &mut Punctuated<GenericParam, Token![,]>,
    where_clause: Option<&WhereClause>,
) -> TokenStream {
    let all_generics_params_with_bounds = quote! { <#generic_params> };

    let mut phantom_data_params = expand_lifetime_generic_params_into_unit_refs(
        generic_params
            .iter()
            .take_while(|param| matches!(param, GenericParam::Lifetime(_)))
            .map(|param| match param {
                GenericParam::Lifetime(l) => l,
                _ => unreachable!(),
            }),
    );

    remove_generic_params_bounds(generic_params);

    for param in generic_params.iter() {
        if !matches!(param, GenericParam::Const(_)) {
            phantom_data_params.extend(quote! { #param, })
        }
    }

    quote! {
       struct SelfWrapper #all_generics_params_with_bounds #where_clause {
           target: &mut #self_path,
           phantom: core::marker::PhantomData<(#phantom_data_params)>,
       };

       impl #all_generics_params_with_bounds core::ops::Deref 
       for SelfWrapper<#generic_params> 
       #where_clause 
       {
           type Target = #self_path;

           fn deref(&self) -> &Self::Target {
               self.target
           }
       }

       impl #all_generics_params_with_bounds core::ops::DerefMut 
       for SelfWrapper<#generic_params> 
       #where_clause 
       {
           fn deref_mut(&mut self) -> &Self::Target {
               sut self.target
           }
       }

       impl #all_generics_params_with_bounds SelfWrapper<#generic_params> #where_clause
    }
}
