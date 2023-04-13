use super::{repeat, Group, HitoriAttribute, Tree};
use crate::parse::{position::Position, repeat::Repeat};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::BTreeSet;
use syn::{punctuated::Punctuated, Expr, Token};

#[derive(Default)]
pub struct State {
    pub impl_wrapper_block: TokenStream,
    pub last_subexpr_matches_ident: Option<Ident>,
    next_subexpr_index: usize,
}

impl State {
    fn set_next_subexpr(&mut self, prefix: &str) {
        self.last_subexpr_matches_ident = Some(format_ident!(
            "__{prefix}_subexpr{}_matches",
            self.next_subexpr_index
        ));
        self.next_subexpr_index += 1;
    }

    fn push_empty_subexpr_matches(&mut self, prefix: &str, matches: bool) {
        self.set_next_subexpr(prefix);
        let ident = &self.last_subexpr_matches_ident;
        let mut tokens = quote! {
            #[inline(always)]
            fn #ident(&mut self) -> bool
        };
        tokens.extend(if matches {
            quote! {{ true }}
        } else {
            quote! {{ false }}
        });

        self.impl_wrapper_block.extend(tokens);
    }

    fn push_subexpr_matches(&mut self, prefix: &str, block: &TokenStream) {
        self.set_next_subexpr(prefix);
        let ident = &self.last_subexpr_matches_ident;
        self.impl_wrapper_block.extend(quote! {
            fn #ident(&mut self) -> bool { #block }
        });
    }

    fn push_group_all(
        &mut self,
        all: &Punctuated<Expr, Token![,]>,
    ) -> syn::Result<BTreeSet<Ident>> {
        let mut unique_capture_idents = BTreeSet::new();
        if all.is_empty() {
            self.push_empty_subexpr_matches("all", true);
            return Ok(unique_capture_idents);
        }
        let mut block = TokenStream::new();

        let mut branch = |expr: &Expr, is_last: bool| -> syn::Result<()> {
            let mut branch_unique_capture_idents = self.push_tree(expr.try_into()?)?;
            let branch_matches_ident = &self.last_subexpr_matches_ident;
            if unique_capture_idents.is_empty() {
                if !is_last {
                    block.extend(quote! {
                        #(
                            let #branch_unique_capture_idents =
                                ::core::clone::Clone::clone(&self.__capture.#branch_unique_capture_idents);
                        )*
                    })
                }
                block.extend(quote! {
                    if !self.#branch_matches_ident() { return false; }
                });
                unique_capture_idents = branch_unique_capture_idents;
            } else {
                let undo = quote! {
                    if !self.#branch_matches_ident() {
                        #(
                            self.__capture.#unique_capture_idents = #unique_capture_idents;
                        )*
                        return false;
                    }
                };
                if is_last {
                    unique_capture_idents.append(&mut branch_unique_capture_idents);
                } else {
                    for ident in branch_unique_capture_idents {
                        if unique_capture_idents.insert(ident.clone()) {
                            block.extend(quote! {
                                let #ident = ::core::clone::Clone::clone(&self.__capture.#ident);
                            });
                        }
                    }
                }
                block.extend(undo);
            }
            Ok(())
        };

        for expr in all.iter().take(all.len() - 1) {
            branch(expr, false)?;
        }
        branch(all.last().unwrap(), true)?;
        block.extend(quote! { true });
        self.push_subexpr_matches("all", &block);
        Ok(unique_capture_idents)
    }

    fn push_group_any(
        &mut self,
        any: &Punctuated<Expr, Token![,]>,
    ) -> syn::Result<BTreeSet<Ident>> {
        let mut unique_capture_idents = BTreeSet::new();
        if any.is_empty() {
            self.push_empty_subexpr_matches("any", false);
            return Ok(unique_capture_idents);
        }
        let mut block = (any.len() > 1)
            .then(|| {
                quote! {
                    let cloned_iter = ::core::clone::Clone::clone(&self.__iter);
                }
            })
            .unwrap_or_default();

        let mut branch = |expr: &Expr, reset_iter: &TokenStream| -> syn::Result<()> {
            unique_capture_idents.append(&mut self.push_tree(expr.try_into()?)?);
            let branch_matches_ident = &self.last_subexpr_matches_ident;
            block.extend(quote! {
                if self.#branch_matches_ident() {
                    return true;
                } else {
                    #reset_iter
                }
            });
            Ok(())
        };

        if any.len() > 2 {
            let reset_iter = quote! {
                self.__iter = ::core::clone::Clone::clone(&cloned_iter);
            };
            for expr in any.iter().take(any.len() - 2) {
                branch(expr, &reset_iter)?;
            }
        }

        if any.len() > 1 {
            branch(
                &any[any.len() - 2],
                &quote! {
                    self.__iter = cloned_iter;
                },
            )?;
        }

        branch(any.last().unwrap(), &quote! { false })?;

        self.push_subexpr_matches("any", &block);
        Ok(unique_capture_idents)
    }

    fn push_group(&mut self, group: Group) -> syn::Result<BTreeSet<Ident>> {
        match group {
            Group::Paren(paren) => self.push_tree(paren.try_into()?),
            Group::All(all) => self.push_group_all(all),
            Group::Any(any) => self.push_group_any(any),
        }
    }

    fn push_repeated_group(
        &mut self,
        group: Group,
        repeat: Repeat,
    ) -> syn::Result<BTreeSet<Ident>> {
        let unique_capture_idents = self.push_group(group)?;
        self.push_subexpr_matches(
            "repeat",
            &repeat::expand_block(
                &repeat,
                &self.last_subexpr_matches_ident.as_ref().unwrap(),
                &unique_capture_idents,
            ),
        );
        Ok(unique_capture_idents)
    }

    fn push_captured_group(
        &mut self,
        group: Group,
        capture_idents: Punctuated<Ident, Token![,]>,
    ) -> syn::Result<BTreeSet<Ident>> {
        let mut unique_capture_idents = self.push_group(group)?;
        if capture_idents.is_empty() {
            return Ok(unique_capture_idents);
        }

        let inner_matches_ident = &self.last_subexpr_matches_ident;
        let capture_idents_xcpt_last_iter = capture_idents.iter().take(capture_idents.len() - 1);
        let last_capture_ident = capture_idents.last().unwrap();

        self.push_subexpr_matches("capture", &quote! {
            let start = ::core::clone::Clone::clone(&self.__end);
            if !self.#inner_matches_ident() {
                return false;
            }
            #(
                self.__capture.#capture_idents_xcpt_last_iter =
                    Some(::core::clone::Clone::clone(&start)..::core::clone::Clone::clone(&self.__end));
            )*
            self.__capture.#last_capture_ident =
                Some(start..::core::clone::Clone::clone(&self.__end));
            true
        });

        unique_capture_idents.extend(capture_idents);
        Ok(unique_capture_idents)
    }

    fn push_positioned_group(
        &mut self,
        group: Group,
        position: Position,
    ) -> syn::Result<BTreeSet<Ident>> {
        let unique_capture_idents = self.push_group(group)?;
        if matches!(position, Position::First | Position::FirstAndLast) {
            // self.is_first && self.inner_subexpr_matches()
        }
        if matches!(position, Position::Last | Position::FirstAndLast) {
            // Don't need to fuse
            // - if it was ? then iter is intact
            // - otherwise there was a test and it passed (iter advanced)
            // self.inner_subexpr_matches() && self.__iter.next().is_none()
        }
        Ok(unique_capture_idents)
    }

    fn push_test(&mut self, test: &Expr) {
        self.push_subexpr_matches(
            "test",
            &quote! {
                let next = if let ::core::option::Option::Some(next) =
                    ::core::iter::Iterator::next(&mut self.__iter)
                {
                    next
                } else {
                    return false;
                };
                if (#test)(next.1) {
                    self.__end = next.0;
                    true
                } else {
                    false
                }
            },
        )
    }

    pub(super) fn push_tree(&mut self, tree: Tree) -> syn::Result<BTreeSet<Ident>> {
        match tree {
            Tree::Group(group, maybe_attr) => match maybe_attr {
                Some(attr) => match attr {
                    HitoriAttribute::Repeat(repeat) => self.push_repeated_group(group, repeat),
                    HitoriAttribute::Capture(capture_idents) => {
                        self.push_captured_group(group, capture_idents)
                    }
                    HitoriAttribute::Position(position) => {
                        self.push_positioned_group(group, position)
                    }
                },
                None => self.push_group(group),
            },
            Tree::Test(test) => {
                self.push_test(test);
                Ok(BTreeSet::new())
            }
        }
    }
}
