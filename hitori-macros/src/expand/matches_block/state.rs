use super::{cache, repeat, Group, HitoriAttribute, Tree};
use crate::parse::{position::Position, repeat::Repeat};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::BTreeSet;
use syn::{punctuated::Punctuated, Expr, Token};

#[derive(Default)]
pub struct State {
    pub impl_wrapper_block: TokenStream,
    pub prev_subexpr_matches_ident: Option<Ident>,
    next_subexpr_index: usize,
}

impl State {
    fn set_next_subexpr(&mut self, prefix: &str) {
        self.prev_subexpr_matches_ident = Some(format_ident!(
            "__{prefix}_subexpr{}_matches",
            self.next_subexpr_index
        ));
        self.next_subexpr_index += 1;
    }

    fn unwrap_prev_subexpr_matches_ident(&self) -> &Ident {
        self.prev_subexpr_matches_ident.as_ref().unwrap()
    }

    fn push_subexpr_matches(&mut self, prefix: &str, block: &TokenStream) {
        self.set_next_subexpr(prefix);
        let ident = self.unwrap_prev_subexpr_matches_ident();
        self.impl_wrapper_block.extend(quote! {
            fn #ident(&mut self) -> bool { #block }
        });
    }

    fn push_group_all(
        &mut self,
        all: &Punctuated<Expr, Token![,]>,
    ) -> syn::Result<BTreeSet<Ident>> {
        let mut inner_capture_idents = BTreeSet::new();
        let mut block = TokenStream::new();

        for expr in all {
            inner_capture_idents.append(&mut self.push_tree(expr.try_into()?)?);
            let branch_matches_ident = self.unwrap_prev_subexpr_matches_ident();
            block.extend(quote! {
                if !self.#branch_matches_ident() {
                    return false;
                }
            });
        }
        block.extend(quote! { true });

        self.push_subexpr_matches("all", &block);
        Ok(inner_capture_idents)
    }

    // TODO: instead of restoring after each branch mismatch,
    // restore if branch matches using `CaptureVars` containing 
    // `inner_capture_idents.difference(branch_capture_idents)`
    fn push_group_any(
        &mut self,
        any: &Punctuated<Expr, Token![,]>,
    ) -> syn::Result<BTreeSet<Ident>> {
        let mut branch_capture_idents_and_subexpr_matches = any
            .iter()
            .map(|expr| {
                expr.try_into()
                    .and_then(|tree| self.push_tree(tree))
                    .map(|capture_idents| {
                        (
                            capture_idents,
                            self.unwrap_prev_subexpr_matches_ident().clone(),
                        )
                    })
            })
            .collect::<syn::Result<Vec<_>>>()?;
        let cache_other_vars = cache::OtherVars::unique_in(
            branch_capture_idents_and_subexpr_matches
                .iter()
                .flat_map(|(capture_idents, _)| capture_idents),
        );

        let mut inner_capture_idents = BTreeSet::new();
        let mut block = TokenStream::new();
        if any.len() > 1 {
            block.extend(cache_other_vars.cache());
        }

        let mut capture_restoring_branch =
            |branch_capture_idents: &mut _,
             branch_subexpr_matches: &_,
             cache_other_vars_restore: &_| {
                let cache_capture_vars = cache::CaptureVars::new(&*branch_capture_idents);
                let capture_vars_cache = cache_capture_vars.cache();
                let capture_vars_restore = cache_capture_vars.restore();
                block.extend(quote! {
                    #capture_vars_cache
                    if self.#branch_subexpr_matches() {
                        return true;
                    }
                    #cache_other_vars_restore
                    #capture_vars_restore
                });
                inner_capture_idents.append(branch_capture_idents);
            };

        if any.len() > 2 {
            let cache_other_vars_restore = cache_other_vars.restore_clone();
            for (branch_capture_idents, branch_subexpr_matches) in
                &mut branch_capture_idents_and_subexpr_matches[..any.len() - 2]
            {
                capture_restoring_branch(
                    branch_capture_idents,
                    branch_subexpr_matches,
                    &cache_other_vars_restore,
                );
            }
        }
        if any.len() > 1 {
            let (branch_capture_idents, branch_subexpr_matches) =
                &mut branch_capture_idents_and_subexpr_matches[any.len() - 2];
            capture_restoring_branch(
                branch_capture_idents,
                branch_subexpr_matches,
                &cache_other_vars.restore(),
            );
        }
        if any.len() != 0 {
            let (branch_capture_idents, branch_subexpr_matches) =
                &mut branch_capture_idents_and_subexpr_matches[any.len() - 1];
            block.extend(quote! {
                if self.#branch_subexpr_matches() {
                    return true;
                }
            });
            inner_capture_idents.append(branch_capture_idents);
        }

        block.extend(quote! { false });
        self.push_subexpr_matches("any", &block);
        Ok(inner_capture_idents)
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
        let inner_capture_idents = self.push_group(group)?;
        self.push_subexpr_matches(
            "repeat",
            &repeat::expand_block(
                &repeat,
                self.unwrap_prev_subexpr_matches_ident(),
                &inner_capture_idents,
            ),
        );
        Ok(inner_capture_idents)
    }

    fn push_captured_group(
        &mut self,
        group: Group,
        capture_idents: Punctuated<Ident, Token![,]>,
    ) -> syn::Result<BTreeSet<Ident>> {
        let mut inner_capture_idents = self.push_group(group)?;
        if capture_idents.is_empty() {
            return Ok(inner_capture_idents);
        }

        let inner_matches_ident = self.unwrap_prev_subexpr_matches_ident();
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

        inner_capture_idents.extend(capture_idents);
        Ok(inner_capture_idents)
    }

    fn push_positioned_group(
        &mut self,
        group: Group,
        position: Position,
    ) -> syn::Result<BTreeSet<Ident>> {
        let inner_capture_idents = self.push_group(group)?;
        if matches!(position, Position::First | Position::FirstAndLast) {
            // self.is_first && self.inner_subexpr_matches()
        }
        if matches!(position, Position::Last | Position::FirstAndLast) {
            // Don't need to fuse
            // - if it was ? then iter is intact
            // - otherwise there was a test and it passed (iter advanced)
            // self.inner_subexpr_matches() && self.__iter.next().is_none()
        }
        Ok(inner_capture_idents)
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
                    self.__is_first = false;
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
