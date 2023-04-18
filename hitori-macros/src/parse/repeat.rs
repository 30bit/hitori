use crate::utils::{expr_add_one_usize, expr_try_from_lit_int_or_lit_str_expr, path_eq_ident_str};
use proc_macro2::Literal;
use std::ops::Bound;
use syn::{parse::Parse, punctuated::Punctuated, Expr, ExprLit, Lit, MetaNameValue, Token};

enum Internal {
    Exact(Expr),
    In { lo: Bound<Expr>, hi: Bound<Expr> },
}

impl Internal {
    fn set_parse_exact(
        repeat: &mut Option<Internal>,
        name_value: MetaNameValue,
    ) -> syn::Result<()> {
        if repeat.is_none() {
            *repeat = Some(Internal::Exact(expr_try_from_lit_int_or_lit_str_expr(
                name_value.value,
            )?));
            Ok(())
        } else {
            Err(syn::Error::new_spanned(
                &name_value.path,
                "must be the only bound",
            ))
        }
    }

    fn set_parse_in_lo(
        repeat: &mut Option<Internal>,
        name_value: MetaNameValue,
        bound: fn(Expr) -> Bound<Expr>,
        err_msg: &str,
    ) -> syn::Result<()> {
        if repeat.is_none() {
            *repeat = Some(Internal::In {
                lo: bound(expr_try_from_lit_int_or_lit_str_expr(name_value.value)?),
                hi: Bound::Unbounded,
            })
        } else if let Some(Internal::In {
            lo: lo @ Bound::Unbounded,
            hi: _,
        }) = repeat
        {
            *lo = bound(expr_try_from_lit_int_or_lit_str_expr(name_value.value)?);
        } else {
            return Err(syn::Error::new_spanned(&name_value.path, err_msg));
        }
        Ok(())
    }

    fn set_parse_in_hi(
        repeat: &mut Option<Internal>,
        name_value: MetaNameValue,
        bound: fn(Expr) -> Bound<Expr>,
        err_msg: &str,
    ) -> syn::Result<()> {
        if repeat.is_none() {
            *repeat = Some(Internal::In {
                lo: Bound::Unbounded,
                hi: bound(expr_try_from_lit_int_or_lit_str_expr(name_value.value)?),
            })
        } else if let Some(Internal::In {
            lo: _,
            hi: hi @ Bound::Unbounded,
        }) = repeat
        {
            *hi = bound(expr_try_from_lit_int_or_lit_str_expr(name_value.value)?);
        } else {
            return Err(syn::Error::new_spanned(&name_value.path, err_msg));
        }
        Ok(())
    }
}

impl Parse for Internal {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let meta = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;
        let mut output = None;

        for name_value in meta {
            if path_eq_ident_str(&name_value.path, "eq") {
                Self::set_parse_exact(&mut output, name_value)?;
            } else if path_eq_ident_str(&name_value.path, "lt") {
                Self::set_parse_in_hi(
                    &mut output,
                    name_value,
                    Bound::Excluded,
                    "cannot be combined with itself or `le`",
                )?;
            } else if path_eq_ident_str(&name_value.path, "le") {
                Self::set_parse_in_hi(
                    &mut output,
                    name_value,
                    Bound::Included,
                    "cannot be combined with itself or `lt`",
                )?;
            } else if path_eq_ident_str(&name_value.path, "gt") {
                Self::set_parse_in_lo(
                    &mut output,
                    name_value,
                    Bound::Excluded,
                    "cannot be combined with itself or `ge`",
                )?;
            } else if path_eq_ident_str(&name_value.path, "ge") {
                Self::set_parse_in_lo(
                    &mut output,
                    name_value,
                    Bound::Included,
                    "cannot be combined with itself or `gt`",
                )?;
            }
        }

        Ok(output.unwrap_or_else(|| Internal::In {
            lo: Bound::Unbounded,
            hi: Bound::Unbounded,
        }))
    }
}

pub enum Repeat {
    Exact(Expr),
    InInclusive {
        lo_included: Expr,
        hi_excluded: Option<Expr>,
    },
}

impl From<Internal> for Repeat {
    fn from(repeat: Internal) -> Self {
        match repeat {
            Internal::Exact(exact) => Self::Exact(exact),
            Internal::In { lo, hi } => Self::InInclusive {
                lo_included: match lo {
                    Bound::Included(lo) => lo,
                    Bound::Excluded(lo) => expr_add_one_usize(lo),
                    Bound::Unbounded => Expr::Lit(ExprLit {
                        attrs: vec![],
                        lit: Lit::Int(Literal::usize_unsuffixed(0).into()),
                    }),
                },
                hi_excluded: match hi {
                    Bound::Included(hi) => Some(expr_add_one_usize(hi)),
                    Bound::Excluded(hi) => Some(hi),
                    Bound::Unbounded => None,
                },
            },
        }
    }
}

impl Parse for Repeat {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Internal::parse(input).map(Into::into)
    }
}
