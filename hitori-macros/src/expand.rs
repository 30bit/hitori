mod matches_block;

use crate::{parse, utils::hitori_ident};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::collections::BTreeSet;
use syn::{punctuated::Punctuated, GenericParam, Token, Type, Visibility, WhereClause};

fn impl_decl(
    hitori_ident: &Ident,
    self_ty: &Type,
    trait_ident: &Ident,
    idx_ty: &Type,
    ch_ty: &Type,
    generic_params: &Punctuated<GenericParam, Token![,]>,
    where_clause: Option<&WhereClause>,
) -> TokenStream {
    quote! {
        impl<#generic_params> #hitori_ident::#trait_ident<#idx_ty, #ch_ty> for #self_ty
        #where_clause
    }
}

fn matches_sig(
    hitori_ident: &Ident,
    is_mut: bool,
    matches_ident: &Ident,
    iter_ident: &Ident,
    idx_ty: &Type,
    ch_ty: &Type,
    inline: bool,
) -> TokenStream {
    let inline = inline.then(|| quote! { #[inline] });
    let mut_ = is_mut.then(|| <Token![mut]>::default());
    quote! {
        #inline
        fn #matches_ident<#iter_ident>(
            &#mut_ self,
            mut start: #idx_ty,
            is_first: bool,
            iter: #iter_ident,
        ) -> ::core::option::Option<#hitori_ident::Matched<
            #idx_ty,
            <Self as #hitori_ident::ExprMut<#idx_ty, #ch_ty>>::Capture,
            #iter_ident::IntoIter,
        >>
        where
            #iter_ident: ::core::iter::IntoIterator<Item = (#idx_ty, #ch_ty)>,
            #iter_ident::IntoIter: ::core::clone::Clone,
    }
}

fn type_capture(capture_ident: &Ident, idx_ty: &Type) -> TokenStream {
    quote! { type Capture = #capture_ident<#idx_ty>; }
}

fn capture<'a>(
    vis: &Visibility,
    ident: &Ident,
    idx_ident: &Ident,
    default_idx_ty: Option<&Type>,
    field_idents: &BTreeSet<Ident>,
) -> TokenStream {
    let (members, default_block, doc) = if field_idents.is_empty() {
        (
            quote! {( ::core::marker::PhantomData<#idx_ident> );},
            quote! {( ::core::marker::PhantomData )},
            Some(quote! { #[doc = "This is an empty placeholder-struct"] }),
        )
    } else {
        (
            quote! {{
                #(
                    #vis #field_idents: ::core::option::Option<::core::ops::Range<#idx_ident>>,
                )*
            }},
            quote! {{
                #(
                    #field_idents: ::core::option::Option::None,
                )*
            }},
            None,
        )
    };
    let idx_bound = default_idx_ty
        .map(|ty| quote! { #idx_ident = #ty })
        .unwrap_or_else(|| idx_ident.to_token_stream());
    quote! {
        #doc
        #[derive(
            ::core::clone::Clone,
            ::core::cmp::Eq,
            ::core::cmp::PartialEq,
            ::core::fmt::Debug,
        )]
        #vis struct #ident<#idx_bound> #members
        impl<#idx_ident> ::core::default::Default for #ident<#idx_ident> {
            fn default() -> Self {
                Self #default_block
            }
        }
    }
}

fn derived_impl_expr_mut_matches_block(
    hitori_ident: &Ident,
    idx_ty: &Type,
    ch_ty: &Type,
) -> TokenStream {
    quote! {
        <Self as #hitori_ident::Expr<#idx_ty, #ch_ty>>::matches(self, start, is_first, iter)
    }
}

pub fn expand(parsed: parse::Output) -> syn::Result<TokenStream> {
    let hitori_ident = hitori_ident();
    let impl_decl = |trait_ident| {
        impl_decl(
            &hitori_ident,
            &parsed.self_ty,
            trait_ident,
            &parsed.idx_ty,
            &parsed.ch_ty,
            &parsed.generic_params,
            parsed.where_clause.as_ref(),
        )
    };
    let matches_sig = |is_mut, inline| {
        let matches_ident = if is_mut {
            format_ident!("matches_mut")
        } else {
            format_ident!("matches")
        };
        matches_sig(
            &hitori_ident,
            is_mut,
            &matches_ident,
            &parsed.iter_ident,
            &parsed.idx_ty,
            &parsed.ch_ty,
            inline,
        )
    };

    let type_capture = type_capture(&parsed.capture_ident, &parsed.idx_ty);
    let (mut output, impl_decl, type_capture, matches_sig) = if parsed.is_mut {
        (
            TokenStream::new(),
            impl_decl(&parsed.trait_ident),
            Some(type_capture),
            matches_sig(true, false),
        )
    } else {
        let impl_expr_decl = impl_decl(&parsed.trait_ident);
        let impl_expr_mut_decl = impl_decl(&format_ident!("ExprMut"));
        let impl_expr_mut_matches_sig = matches_sig(true, true);
        let impl_expr_mut_matches_block =
            derived_impl_expr_mut_matches_block(&hitori_ident, &parsed.idx_ty, &parsed.ch_ty);
        (
            quote! {
                #impl_expr_mut_decl {
                    #type_capture
                    #impl_expr_mut_matches_sig { #impl_expr_mut_matches_block }
                }
            },
            impl_expr_decl,
            None,
            matches_sig(false, false),
        )
    };

    let matches_block::Output {
        tokens: matches_block,
        inner_capture_idents,
    } = matches_block::Input {
        hitori_ident: &hitori_ident,
        is_mut: parsed.is_mut,
        capture_ident: &parsed.capture_ident,
        self_ty: &parsed.self_ty,
        iter_ident: &parsed.iter_ident,
        idx_ty: &parsed.idx_ty,
        ch_ty: &parsed.ch_ty,
        expr: &parsed.expr,
        wrapper_ident: &parsed.wrapper_ident,
        generic_params: parsed.generic_params,
        where_clause: parsed.where_clause.as_ref(),
    }
    .expand()?;

    output.extend(quote! {
        #impl_decl {
            #type_capture
            #matches_sig { #matches_block }
        }
    });

    output.extend(capture(
        &parsed.capture_vis,
        &parsed.capture_ident,
        &parsed.capture_idx_ident,
        (!parsed.is_idx_generic).then(|| &parsed.idx_ty),
        &inner_capture_idents,
    ));

    Ok(output)
}
