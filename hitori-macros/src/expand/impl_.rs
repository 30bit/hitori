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
        fn matches<__I>(
            &#mut_ self,
            capture: &mut #capture_arg,
            mut start: #idx_arg,
            iter: __I
        ) -> core::result::Result<
            core::option::Option<core::ops::RangeTo<#idx_arg>>,
            <#capture_arg as #hitori_ident::CaptureMut>::Error
        >
        where
            __I: core::iter::IntoIterator<Item = (#idx_arg, #ch_arg)>,
            __I::IntoIter: Clone,
    }
}

pub struct Input<'a> {
    pub hitori_ident: &'a Ident,
    pub config: &'a Config,
    pub generic_params: &'a mut Punctuated<GenericParam, Token![,]>,
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
            generic_params: &mut parsed.generic_params,
            where_clause: parsed.where_clause.as_ref(),
            self_path: &parsed.self_path,
            trait_ident: &parsed.trait_ident,
            trait_args: &parsed.trait_args,
            const_expr: &mut parsed.const_expr,
        }
    }
}

fn expand_header(
    Input {
        hitori_ident,
        generic_params,
        where_clause,
        self_path,
        trait_ident,
        trait_args,
        ..
    }: &Input,
) -> TokenStream {
    quote! {
        impl<#generic_params> #hitori_ident::#trait_ident<#(#trait_args),*> for #self_path
        #where_clause
    }
}

fn expand_expr_mut_from_expr(input: &Input) -> TokenStream {
    let matches_sig = matches_sig(input.hitori_ident, input.trait_args, true);
    let mut output = expand_header(&input);
    let hitori_ident = input.hitori_ident;
    output.extend(quote! {{
        #matches_sig {
            <Self as #hitori_ident::Expr<_, _, _>>::matches(self, capture, start, iter)
        }
    }});
    output
}

pub struct Output {
    pub tokens: TokenStream,
    pub capture_fn_idents: Vec<Ident>,
}

fn expand_with_matches_block(input: &mut Input) -> syn::Result<Output> {
    let mut header = expand_header(&input);
    matches_block::Input {
        hitori_ident: input.hitori_ident,
        generic_params: input.generic_params,
        where_clause: input.where_clause,
        trait_args: input.trait_args,
        self_path: input.self_path,
        expr: input.const_expr,
        is_mut: matches!(input.config, Config::ExprMut),
    }
    .try_into()
    .map(
        |matches_block::Output {
             tokens: matches_block,
             capture,
         }| {
            let matches_sig = matches_sig(
                input.hitori_ident,
                input.trait_args,
                matches!(input.config, Config::ExprMut),
            );
            header.extend(quote! { { #matches_sig { #matches_block } } });
            Output {
                tokens: header,
                capture_fn_idents: capture,
            }
        },
    )
}

impl<'a> TryFrom<Input<'a>> for Output {
    type Error = syn::Error;

    fn try_from(mut input: Input<'a>) -> Result<Self, Self::Error> {
        if input.config.and_expr_mut() {
            let trait_ident = format_ident!("ExprMut");
            let impl_config = Config::ExprMut;
            let other_trait_ident = mem::replace(&mut input.trait_ident, unsafe {
                mem::transmute(&trait_ident)
            });
            let other_config =
                mem::replace(&mut input.config, unsafe { mem::transmute(&impl_config) });
            let tokens = expand_expr_mut_from_expr(&input);
            input.trait_ident = other_trait_ident;
            input.config = other_config;
            expand_with_matches_block(&mut input).map(|mut output| {
                output.tokens.extend(tokens);
                output
            })
        } else {
            expand_with_matches_block(&mut input)
        }
    }
}
