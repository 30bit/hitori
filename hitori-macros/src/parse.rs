pub mod args;

use crate::utils::{eq_by_fmt, type_path_ref};
use args::Args;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, ToTokens as _};
use std::array;
use syn::{
    parse2,
    punctuated::{self, Punctuated},
    token::Bang,
    Expr, GenericArgument, GenericParam, ImplItem, ImplItemConst, ItemImpl, Path, PathArguments,
    PathSegment, Token, Type, TypePath, VisPublic, Visibility, WhereClause,
};

fn trait_ident_and_args(
    (_, mut path, _): (Option<Bang>, Path, Token![for]),
) -> syn::Result<(Ident, Punctuated<GenericArgument, Token![,]>)> {
    Err(
        if path.segments.len() != 1 || path.leading_colon.is_some() {
            syn::Error::new_spanned(path, "expected ident")
        } else {
            let Some(punctuated::Pair::End(PathSegment { ident, arguments })) = path.segments.pop() else {
            unreachable!()
        };
            match arguments {
                PathArguments::AngleBracketed(args) => {
                    if args.args.len() == 2 {
                        return Ok((ident, args.args));
                    } else {
                        syn::Error::new_spanned(args, "expected 2 arguments")
                    }
                }
                PathArguments::Parenthesized(args) => {
                    syn::Error::new_spanned(args, "expected angle brackets around arguments")
                }
                PathArguments::None => syn::Error::new_spanned(ident, "expected 2 arguments"),
            }
        },
    )
}

fn const_expr(items: Vec<ImplItem>) -> syn::Result<Expr> {
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
                ImplItem::Method(method) => method.into_token_stream(),
                ImplItem::Type(ty) => ty.into_token_stream(),
                ImplItem::Macro(macro_) => macro_.into_token_stream(),
                ImplItem::Verbatim(verbatim) => verbatim,
                _ => TokenStream::new(),
            },
            "not a const item",
        ))
    });

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
            init.combine(error(result))
        }
        init
    }

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
    pub vis: Visibility,
    pub capture_ident: Ident,
    pub generic_params: Punctuated<GenericParam, Token![,]>,
    pub where_clause: Option<WhereClause>,
    pub self_ty: Box<Type>,
    pub trait_ident: Ident,
    pub trait_args: [GenericArgument; 2],
    pub const_expr: Expr,
}

impl Output {
    fn new(is_mut: bool, args: Args, item: ItemImpl) -> syn::Result<Self> {
        let (trait_ident, trait_args) = trait_ident_and_args(
            item.trait_
                .ok_or_else(|| syn::Error::new_spanned(&item.self_ty, "not a trait impl"))?,
        )?;

        if is_mut {
            if trait_ident != "ExprMut" {
                return Err(syn::Error::new_spanned(trait_ident, "not `ExprMut`"));
            }
        } else if trait_ident != "Expr" {
            return Err(syn::Error::new_spanned(trait_ident, "not `Expr`"));
        }

        let capture_ident = if let Some(ident) = args.capture_ident {
            ident
        } else {
            match type_path_ref(&item.self_ty) {
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

        let vis = args.vis.unwrap_or_else(|| {
            Visibility::Public(VisPublic {
                pub_token: Default::default(),
            })
        });

        const_expr(item.items).map(|const_expr| Output {
            is_mut,
            vis,
            capture_ident,
            generic_params: item.generics.params,
            where_clause: item.generics.where_clause,
            self_ty: item.self_ty,
            trait_ident,
            trait_args: {
                let mut trait_args_iter = trait_args.into_iter();
                array::from_fn(|_| trait_args_iter.next().unwrap())
            },
            const_expr,
        })
    }
}

pub fn parse(is_mut: bool, attr: TokenStream, item: TokenStream) -> syn::Result<Output> {
    Output::new(is_mut, parse2(attr)?, parse2(item)?)
}