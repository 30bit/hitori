use crate::utils::{expr_add_one_usize, path_eq_ident_str, UsizeOrExpr};
use std::ops::Bound;
use syn::{parse::Parse, punctuated::Punctuated, MetaNameValue, Token};

struct InternalIn {
    pub lo: Bound<UsizeOrExpr>,
    pub hi: Bound<UsizeOrExpr>,
}

impl InternalIn {
    fn unbounded() -> Self {
        Self {
            lo: Bound::Unbounded,
            hi: Bound::Unbounded,
        }
    }
}

fn repeat_in_lo_as_included(lo: Bound<UsizeOrExpr>) -> UsizeOrExpr {
    match lo {
        Bound::Included(lo) => lo,
        Bound::Excluded(UsizeOrExpr::Usize(lo)) => UsizeOrExpr::Usize(lo + 1),
        Bound::Excluded(UsizeOrExpr::Expr(lo)) => UsizeOrExpr::Expr(expr_add_one_usize(lo)),
        Bound::Unbounded => UsizeOrExpr::Usize(0),
    }
}

fn repeat_in_hi_as_included(hi: Bound<UsizeOrExpr>) -> Option<UsizeOrExpr> {
    match hi {
        Bound::Included(hi) => Some(hi),
        Bound::Excluded(UsizeOrExpr::Usize(hi)) => Some(UsizeOrExpr::Usize(hi + 1)),
        Bound::Excluded(UsizeOrExpr::Expr(hi)) => Some(UsizeOrExpr::Expr(expr_add_one_usize(hi))),
        Bound::Unbounded => None,
    }
}

enum Internal {
    Exact(UsizeOrExpr),
    In(InternalIn),
}

impl Internal {
    fn set_parse_exact(
        repeat: &mut Option<Internal>,
        name_value: &MetaNameValue,
    ) -> syn::Result<()> {
        if repeat.is_none() {
            *repeat = Some(Internal::Exact(UsizeOrExpr::from_lit(&name_value.lit)?));
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
        name_value: &MetaNameValue,
        bound: fn(UsizeOrExpr) -> Bound<UsizeOrExpr>,
        err_msg: &str,
    ) -> syn::Result<()> {
        if repeat.is_none() {
            *repeat = Some(Internal::In(InternalIn {
                lo: bound(UsizeOrExpr::from_lit(&name_value.lit)?),
                hi: Bound::Unbounded,
            }))
        } else if let Some(Internal::In(InternalIn {
            lo: lo @ Bound::Unbounded,
            hi: _,
        })) = repeat
        {
            *lo = bound(UsizeOrExpr::from_lit(&name_value.lit)?);
        } else {
            return Err(syn::Error::new_spanned(&name_value.path, err_msg));
        }
        Ok(())
    }

    fn set_parse_in_hi(
        repeat: &mut Option<Internal>,
        name_value: &MetaNameValue,
        bound: fn(UsizeOrExpr) -> Bound<UsizeOrExpr>,
        err_msg: &str,
    ) -> syn::Result<()> {
        if repeat.is_none() {
            *repeat = Some(Internal::In(InternalIn {
                lo: Bound::Unbounded,
                hi: bound(UsizeOrExpr::from_lit(&name_value.lit)?),
            }))
        } else if let Some(Internal::In(InternalIn {
            lo: _,
            hi: hi @ Bound::Unbounded,
        })) = repeat
        {
            *hi = bound(UsizeOrExpr::from_lit(&name_value.lit)?);
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

        for name_value in &meta {
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

        Ok(output.unwrap_or_else(|| Internal::In(InternalIn::unbounded())))
    }
}

pub enum Repeat {
    Exact(UsizeOrExpr),
    InInclusive {
        lo: UsizeOrExpr,
        hi: Option<UsizeOrExpr>,
    },
}

impl From<Internal> for Repeat {
    fn from(repeat: Internal) -> Self {
        match repeat {
            Internal::Exact(exact) => Self::Exact(exact),
            Internal::In(InternalIn { lo, hi }) => Self::InInclusive {
                lo: repeat_in_lo_as_included(lo),
                hi: repeat_in_hi_as_included(hi),
            },
        }
    }
}

impl Parse for Repeat {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Internal::parse(input).map(Into::into)
    }
}
