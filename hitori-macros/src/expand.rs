mod capture;
mod matches_block;

use crate::{parse, utils::hitori_ident};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, GenericParam, Token, Type, WhereClause};

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
) -> TokenStream {
    let mut_ = is_mut.then_some(<Token![mut]>::default());
    quote! {
        fn #matches_ident<__I>(
            &#mut_ self,
            mut start: #idx_ty,
            iter: #iter_ident,
        ) -> core::option::Option<
            core::ops::RangeTo<#idx_ty>>,
            <Self as #hitori_ident::ExprMut<#idx_ty, #ch_ty>>::Capture
        >
        where
            #iter_ident: core::iter::IntoIterator<Item = (#idx_ty, #ch_ty)>,
            #iter_ident::IntoIter: core::clone::Clone,
    }
}

fn type_capture(capture_ident: &Ident, idx_ty: &Type) -> TokenStream {
    quote! {
        type Capture = #capture_ident<#idx_ty>;
    }
}

fn derived_impl_expr_mut_matches_block(
    hitori_ident: &Ident,
    idx_ty: &Type,
    ch_ty: &Type,
) -> TokenStream {
    quote! {
        <Self as #hitori_ident::Expr<#idx_ty, #ch_ty>>::matches(self, start, iter)
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
    let matches_sig = |is_mut| {
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
        )
    };

    let type_capture = type_capture(&parsed.capture_options_ident, &parsed.idx_ty);
    let (mut output, impl_decl, type_capture, matches_sig) = if parsed.is_mut {
        (
            TokenStream::new(),
            impl_decl(&parsed.trait_ident),
            Some(type_capture),
            matches_sig(true),
        )
    } else {
        let impl_expr_decl = impl_decl(&parsed.trait_ident);
        let impl_expr_mut_decl = impl_decl(&format_ident!("ExprMut"));
        let impl_expr_mut_matches_sig = matches_sig(true);
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
            matches_sig(false),
        )
    };

    let (matches_block, capture_fields) = matches_block::expand(
        &hitori_ident,
        parsed.is_mut,
        &parsed.capture_vecs_ident,
        &parsed.self_ty,
        &parsed.iter_ident,
        &parsed.idx_ty,
        &parsed.ch_ty,
        parsed.const_expr,
        &parsed.wrapper_ident,
        parsed.generic_params,
        parsed.where_clause.as_ref(),
    )?;
    output.extend(quote! {
        #impl_decl {
            #type_capture
            #matches_sig { #matches_block }
        }
    });

    output.extend(capture::options(
        &parsed.capture_vis,
        &parsed.capture_options_ident,
        &parsed.capture_idx_ident,
        (!parsed.is_idx_generic).then_some(&parsed.idx_ty),
        capture_fields.iter().map(|field| &field.ident),
    ));

    Ok(output)
}
