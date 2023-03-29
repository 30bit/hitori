use proc_macro2::{Ident, TokenStream};
use std::collections::BTreeSet;
use syn::{punctuated::Punctuated, Expr, GenericParam, Token, Type, WhereClause};

use super::capture;

pub fn expand(
    is_mut: bool,
    hitori_ident: &Ident,
    self_ty: &Type,
    idx_ty: &Type,
    ch_ty: &Type,
    expr: Expr,
    generic_params: Punctuated<GenericParam, Token![,]>,
    where_clause: Option<&WhereClause>,
) -> syn::Result<(TokenStream, BTreeSet<capture::Field>)> {
    // TODO: include capture vecs in TokenStream
    todo!()
}
