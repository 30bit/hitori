pub mod define;
pub mod impl_;

use crate::utils::{eq_by_fmt, find_unique_hitori_attr, take_type_path, type_path_ref};
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens as _;
use std::array;
use syn::{
    parse2,
    punctuated::{self, Punctuated},
    token::Bang,
    Expr, GenericArgument, GenericParam, ImplItem, ImplItemConst, ItemImpl, Path, PathArguments,
    PathSegment, Token, WhereClause,
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
                    if args.args.len() == 3 {
                        return Ok((ident, args.args));
                    } else {
                        syn::Error::new_spanned(args, "expected 3 arguments")
                    }
                }
                PathArguments::Parenthesized(args) => {
                    syn::Error::new_spanned(args, "expected angle brackets around arguments")
                }
                PathArguments::None => syn::Error::new_spanned(ident, "expected 3 arguments"),
            }
        },
    )
}

fn const_expr(items: Vec<ImplItem>) -> syn::Result<Expr> {
    let mut const_iter = items.into_iter().map(|item| {
        Err(syn::Error::new_spanned(
            match item {
                syn::ImplItem::Const(const_) => {
                    return Err(if const_.ident != "PATTERN" {
                        syn::Error::new_spanned(const_.ident, "not `PATTERN`")
                    } else if !eq_by_fmt(&const_.ty, <Token![_]>::default()) {
                        syn::Error::new_spanned(const_.ty, "not an underscore")
                    } else {
                        return Ok(const_);
                    });
                }
                syn::ImplItem::Method(method) => method.into_token_stream(),
                syn::ImplItem::Type(ty) => ty.into_token_stream(),
                syn::ImplItem::Macro(macro_) => macro_.into_token_stream(),
                syn::ImplItem::Verbatim(verbatim) => verbatim,
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
    pub impl_config: impl_::Config,
    pub define_config: Option<define::Config>,
    pub generic_params: Punctuated<GenericParam, Token![,]>,
    pub where_clause: Option<WhereClause>,
    pub self_path: Path,
    pub trait_ident: Ident,
    pub trait_args: [GenericArgument; 3],
    pub const_expr: Expr,
}

impl Output {
    fn new(impl_config: impl_::Config, item: ItemImpl) -> syn::Result<Self> {
        let define_config = find_unique_hitori_attr(&item.attrs, "and_define")?;
        let self_path = type_path_ref(&item.self_ty)
            .ok_or_else(|| syn::Error::new_spanned(&item.self_ty, "not a path type"))?;
        let (trait_ident, trait_args) = trait_ident_and_args(
            item.trait_
                .ok_or_else(|| syn::Error::new_spanned(self_path, "not a trait impl"))?,
        )?;

        if impl_config != trait_ident {
            return Err(syn::Error::new_spanned(
                trait_ident,
                match impl_config {
                    impl_::Config::Expr { .. } => "not `Expr`",
                    impl_::Config::ExprMut => "not `ExprMut`",
                },
            ));
        }

        const_expr(item.items).map(|const_expr| Output {
            impl_config,
            define_config,
            generic_params: item.generics.params,
            where_clause: item.generics.where_clause,
            self_path: take_type_path(item.self_ty).unwrap().path,
            trait_ident,
            trait_args: {
                let mut trait_args_iter = trait_args.into_iter();
                array::from_fn(|_| trait_args_iter.next().unwrap())
            },
            const_expr,
        })
    }

    pub fn parse<const MUT: bool>(attr: TokenStream, item: TokenStream) -> syn::Result<Self> {
        Self::new(impl_::Config::parse::<MUT>(attr)?, parse2(item)?)
    }
}
