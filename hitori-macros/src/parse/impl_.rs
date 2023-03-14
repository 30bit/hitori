use proc_macro2::{Ident, TokenStream};
use syn::{parse::Parser as _, punctuated::Punctuated, Token};

pub enum Config {
    Expr { and_expr_mut: bool },
    ExprMut,
}

impl Config {
    pub fn parse<const MUT: bool>(tokens: TokenStream) -> syn::Result<Self> {
        if !MUT {
            let args = Punctuated::<Ident, Token![,]>::parse_terminated.parse2(tokens)?;
            let unexpected_arg =
                |index| Err(syn::Error::new_spanned(&args[index], "unexpected argument"));
            if args.is_empty() {
                Ok(Self::Expr {
                    and_expr_mut: false,
                })
            } else if args[0] == "and_expr_mut" {
                if args.len() != 1 {
                    if args[1] == "and_expr_mut" {
                        Err(syn::Error::new_spanned(
                            &args[1],
                            "duplicate `and_expr_mut`",
                        ))
                    } else {
                        unexpected_arg(1)
                    }
                } else {
                    Ok(Self::Expr { and_expr_mut: true })
                }
            } else {
                unexpected_arg(0)
            }
        } else if tokens.is_empty() {
            Ok(Self::ExprMut)
        } else {
            Err(syn::Error::new_spanned(tokens, "expected no arguments"))
        }
    }

    pub fn and_expr_mut(&self) -> bool {
        match self {
            Config::Expr {
                and_expr_mut: and_mut,
            } => *and_mut,
            Config::ExprMut => false,
        }
    }
}

impl PartialEq<Ident> for Config {
    fn eq(&self, other: &Ident) -> bool {
        other
            == match self {
                Config::Expr { .. } => "Expr",
                Config::ExprMut => "ExprMut",
            }
    }
}
