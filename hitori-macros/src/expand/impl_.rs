mod matches_block;

use crate::parse::{self, impl_::Config};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::mem;
use syn::{punctuated::Punctuated, Expr, GenericArgument, GenericParam, Path, Token, WhereClause};

fn matches_sig(
    hitori_ident: &Ident,
    [capture_arg, idx_arg, ch_arg]: &[GenericArgument; 3],
    is_mut: bool,
) -> TokenStream {
    let mut_ = is_mut.then_some(<Token![mut]>::default());
    quote! {
        fn matches<I>(
            &#mut_ self,
            capture: &mut #capture_arg,
            mut start: #idx_arg,
            iter: I
        ) -> core::result::Result<
            core::option::Option<core::ops::RangeTo<#idx_arg>>,
            <#capture_arg as #hitori_ident::CaptureMut>::Error
        >
        where
            I: IntoIterator<Item = (#idx_arg, #ch_arg)>,
            I::IntoIter: Clone
    }
}

pub struct Input<'a> {
    pub hitori_ident: &'a Ident,
    pub config: &'a Config,
    pub generic_params: &'a Punctuated<GenericParam, Token![,]>,
    pub where_clause: Option<&'a WhereClause>,
    pub self_path: &'a Path,
    pub trait_ident: &'a Ident,
    pub trait_args: &'a [GenericArgument; 3],
    pub const_expr: &'a mut Expr,
}

impl<'a> Input<'a> {
    pub fn new(hitori_ident: &'a Ident, parsed: &'a mut parse::Output) -> Self {
        Self {
            hitori_ident,
            config: &parsed.impl_config,
            generic_params: &parsed.generic_params,
            where_clause: parsed.where_clause.as_ref(),
            self_path: &parsed.self_path,
            trait_ident: &parsed.trait_ident,
            trait_args: &parsed.trait_args,
            const_expr: &mut parsed.const_expr,
        }
    }

    fn expand_one(
        Self {
            hitori_ident,
            config,
            generic_params,
            where_clause,
            self_path,
            trait_ident,
            trait_args,
            ..
        }: &Self,
        matches_block: &TokenStream,
    ) -> TokenStream {
        let matches_sig = matches_sig(hitori_ident, trait_args, matches!(config, Config::ExprMut));
        quote! {
            impl<#generic_params> #hitori_ident::#trait_ident<#(#trait_args),*> for #self_path
            #where_clause
            {
                #matches_sig { #matches_block }
            }
        }
    }
}

pub struct Output {
    pub tokens: TokenStream,
    pub capture_fn_idents: Vec<Ident>,
}

impl<'a> TryFrom<Input<'a>> for Output {
    type Error = syn::Error;

    fn try_from(mut input: Input<'a>) -> Result<Self, Self::Error> {
        let matches_block::Output {
            tokens: matches_block,
            capture: capture_fn_idents,
        } = input.const_expr.try_into()?;
        let mut tokens = Input::expand_one(&input, &matches_block);
        if input.config.and_expr_mut() {
            let trait_ident = format_ident!("ExprMut");
            let impl_config = Config::ExprMut;
            input.trait_ident = unsafe { mem::transmute(&trait_ident) };
            input.config = unsafe { mem::transmute(&impl_config) };
            tokens.extend(Input::expand_one(&input, &matches_block));
        }
        Ok(Self {
            tokens,
            capture_fn_idents,
        })
    }
}
