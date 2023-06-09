pub mod position;
pub mod repeat;

mod args;

use crate::utils::{
    eq_by_fmt, generic_arg_try_into_type, has_type_any_generic_params, ident_not_in_generic_params,
    type_as_type_path,
};
use args::Args;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, ToTokens as _};
use syn::{
    parse2,
    punctuated::{self, Punctuated},
    Expr, GenericParam, ImplItem, ImplItemConst, ItemImpl, Path, PathArguments, PathSegment, Token,
    Type, TypePath, Visibility, WhereClause,
};

fn trait_ident_and_args(mut path: Path) -> syn::Result<(Ident, [Type; 2])> {
    Err(
        if path.segments.len() != 1 || path.leading_colon.is_some() {
            syn::Error::new_spanned(path, "expected identifier")
        } else if let Some(punctuated::Pair::End(PathSegment { ident, arguments })) =
            path.segments.pop()
        {
            match arguments {
                PathArguments::AngleBracketed(args) => {
                    if args.args.len() == 2 {
                        let mut args = args.args.into_iter();
                        let idx_arg = generic_arg_try_into_type(args.next().unwrap())?;
                        let ch_arg = generic_arg_try_into_type(args.next().unwrap())?;
                        return Ok((ident, [idx_arg, ch_arg]));
                    }
                    syn::Error::new_spanned(args, "expected 2 arguments")
                }
                PathArguments::Parenthesized(args) => {
                    syn::Error::new_spanned(args, "expected angle brackets around arguments")
                }
                PathArguments::None => syn::Error::new_spanned(ident, "expected 2 arguments"),
            }
        } else {
            unreachable!()
        },
    )
}

fn const_expr(items: Vec<ImplItem>) -> syn::Result<Expr> {
    fn error(result: syn::Result<ImplItemConst>) -> syn::Error {
        match result {
            Ok(const_) => syn::Error::new_spanned(const_, "multiple const items"),
            Err(err) => err,
        }
    }

    fn combine_errors(
        mut init: syn::Error,
        iter: impl Iterator<Item = syn::Result<ImplItemConst>>,
    ) -> syn::Error {
        for result in iter {
            init.combine(error(result));
        }
        init
    }

    let mut const_iter = items.into_iter().map(|item| {
        Err(syn::Error::new_spanned(
            match item {
                ImplItem::Const(const_) => {
                    return Err(if const_.ident != "PATTERN" {
                        syn::Error::new_spanned(const_.ident, "not `PATTERN`")
                    } else if !eq_by_fmt(&const_.ty, <Token![_]>::default()) {
                        syn::Error::new_spanned(const_.ty, "not an underscore")
                    } else {
                        return Ok(const_);
                    });
                }
                item => item.into_token_stream(),
            },
            "not a const item",
        ))
    });

    Err(match const_iter.next() {
        Some(Ok(ImplItemConst { expr, .. })) => match const_iter.next() {
            Some(next) => combine_errors(error(next), const_iter),
            None => return Ok(expr),
        },
        Some(Err(err)) => combine_errors(err, const_iter),
        None => syn::Error::new_spanned(TokenStream::new(), "empty impl"),
    })
}

pub struct Output {
    pub is_mut: bool,
    pub capture_vis: Visibility,
    pub capture_ident: Ident,
    pub capture_idx_ident: Ident,
    pub self_ty: Box<Type>,
    pub trait_ident: Ident,
    pub iter_ident: Ident,
    pub idx_ty: Type,
    pub is_idx_generic: bool,
    pub ch_ty: Type,
    pub expr: Expr,
    pub wrapper_ident: Ident,
    pub generic_params: Punctuated<GenericParam, Token![,]>,
    pub where_clause: Option<WhereClause>,
}

impl Output {
    fn new(is_mut: bool, args: Args, item: ItemImpl) -> syn::Result<Self> {
        let iter_ident = ident_not_in_generic_params(&item.generics.params, "I".into());
        let wrapper_ident = ident_not_in_generic_params(&item.generics.params, "Self_".into());

        let (trait_ident, [idx_ty, ch_ty]) = trait_ident_and_args(
            item.trait_
                .ok_or_else(|| syn::Error::new_spanned(&item.self_ty, "not a trait impl"))?
                .1,
        )?;

        if is_mut {
            if trait_ident != "ExprMut" {
                return Err(syn::Error::new_spanned(trait_ident, "not `ExprMut`"));
            }
        } else if trait_ident != "Expr" {
            return Err(syn::Error::new_spanned(trait_ident, "not `Expr`"));
        }

        let is_idx_generic = has_type_any_generic_params(&item.generics.params, &idx_ty);

        let vis = args
            .capture_vis
            .unwrap_or_else(|| Visibility::Public(<Token![pub]>::default()));

        let capture_ident = if let Some(ident) = args.capture_ident {
            ident
        } else {
            match type_as_type_path(&item.self_ty) {
                Some(TypePath {
                    path: Path { segments, .. },
                    ..
                }) if !segments.is_empty() => {
                    let self_ident = &segments.last().unwrap().ident;
                    format_ident!("{self_ident}Capture")
                }
                _ => format_ident!("Capture"),
            }
        };

        let capture_idx_ident = if is_idx_generic
            && type_as_type_path(&idx_ty)
                .and_then(|type_path| type_path.path.get_ident())
                .map(|idx_ident| idx_ident == "Idx")
                .unwrap_or_default()
        {
            format_ident!("Idx_")
        } else {
            format_ident!("Idx")
        };

        const_expr(item.items).map(|expr| Output {
            is_mut,
            capture_vis: vis,
            capture_ident,
            capture_idx_ident,
            self_ty: item.self_ty,
            trait_ident,
            iter_ident,
            idx_ty,
            is_idx_generic,
            ch_ty,
            expr,
            wrapper_ident,
            generic_params: item.generics.params,
            where_clause: item.generics.where_clause,
        })
    }
}

pub fn parse(is_mut: bool, attr: TokenStream, item: TokenStream) -> syn::Result<Output> {
    Output::new(is_mut, parse2(attr)?, parse2(item)?)
}
