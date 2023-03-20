use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::ops::AddAssign;
use syn::{
    parse::Parse, parse_quote, punctuated::Punctuated, Expr, ExprRange, FnArg, GenericArgument,
    GenericParam, Path, Token, Type, WhereClause,
};

use crate::utils::{
    collect_hitori_attrs, expand_lifetime_generic_params_into_unit_refs, find_unique_hitori_attr,
    remove_generic_params_bounds, take_hitori_attrs,
};

enum Repeat {
    Star,
    Plus,
    Question,
    Exact(Expr),
    Range(ExprRange),
}

impl Parse for Repeat {
    #[allow(unreachable_code, unused_variables)]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        return Err(input.error("repetitions are not implemented yet"));
        Ok(if input.fork().parse::<Token![*]>().is_ok() {
            Self::Star
        } else if input.fork().parse::<Token![+]>().is_ok() {
            Self::Plus
        } else if input.fork().parse::<Token![?]>().is_ok() {
            Self::Question
        } else {
            match input.parse::<Expr>() {
                Ok(Expr::Range(range)) => Self::Range(range),
                Ok(expr) => Self::Exact(expr),
                Err(expr_err) => {
                    let mut err = syn::Error::new_spanned(
                        TokenStream::new(),
                        "not a `*`, `+`, `?` or expression",
                    );
                    err.combine(expr_err);
                    return Err(err);
                }
            }
        })
    }
}

type Group<'a> = &'a mut Punctuated<Expr, Token![,]>;

enum TreeInner<'a> {
    All(Group<'a>),
    Any(Group<'a>),
    Test(&'a Expr),
}

struct Tree<'a> {
    inner: TreeInner<'a>,
    #[allow(dead_code)]
    repeat: Option<Repeat>,
    capture: Vec<Ident>, // used in place for calls
}

impl<'a> TryFrom<&'a mut Expr> for Tree<'a> {
    type Error = syn::Error;

    fn try_from(expr: &'a mut Expr) -> syn::Result<Self> {
        let attrs = take_hitori_attrs(expr);
        let repeat = find_unique_hitori_attr(&attrs, "repeat")?;
        let capture = collect_hitori_attrs(&attrs, "capture")?;
        Ok(Self {
            inner: match expr {
                Expr::Tuple(tuple) => TreeInner::All(&mut tuple.elems),
                Expr::Array(arr) => TreeInner::Any(&mut arr.elems),
                _ => TreeInner::Test(expr),
            },
            repeat,
            capture,
        })
    }
}

#[derive(Default)]
pub struct Output {
    pub tokens: TokenStream,
    pub capture: Vec<Ident>, // used for #[hitori::add_define]
}

impl AddAssign for Output {
    fn add_assign(&mut self, rhs: Self) {
        self.tokens.extend(rhs.tokens);
        self.capture.extend(rhs.capture);
    }
}

pub struct Input<'a> {
    pub generic_params: &'a mut Punctuated<GenericParam, Token![,]>,
    pub where_clause: Option<&'a WhereClause>,
    pub trait_args: &'a [GenericArgument; 3],
    pub self_path: &'a Path,
    pub expr: &'a mut Expr,
    pub is_mut: bool,
}

fn expand_wrapper(
    Input {
        self_path,
        generic_params,
        where_clause,
        is_mut,
        trait_args: [capture_arg, idx_arg, ch_arg],
        ..
    }: &mut Input,
) -> TokenStream {
    let all_generics_params_with_bounds = quote! { #generic_params };

    let mut phantom_data_params = expand_lifetime_generic_params_into_unit_refs(
        generic_params
            .iter()
            .take_while(|param| matches!(param, GenericParam::Lifetime(_)))
            .map(|param| match param {
                GenericParam::Lifetime(l) => l,
                _ => unreachable!(),
            }),
    );

    remove_generic_params_bounds(generic_params);

    for param in generic_params.iter() {
        if !matches!(param, GenericParam::Const(_)) {
            param.to_tokens(&mut phantom_data_params);
        }
    }

    let where_clause = {
        let mut output = where_clause.as_ref().map_or_else(
            || quote! { where },
            |existing| {
                if existing.predicates.empty_or_trailing() {
                    quote! { #where_clause }
                } else {
                    quote! { #where_clause, }
                }
            },
        );
        output.extend(quote! {
            __I: IntoIterator<Item = (#idx_arg, #ch_arg)>,
            __I::IntoIter: Clone,
        });
        output
    };

    let mut_ = is_mut.then_some(<Token![mut]>::default());

    let mut output = quote! {
       struct __Self<#all_generics_params_with_bounds, __I> #where_clause {
           __target: &#(#mut_)? #self_path,
           __capture: &mut #capture_arg,
           __end: #idx_arg,
           __iter: __I,
           __phantom: core::marker::PhantomData<(#phantom_data_params)>,
       };

       impl<#all_generics_params_with_bounds> core::ops::Deref for __Self<#generic_params>
       #where_clause
       {
           type Target = #self_path;

           fn deref(&self) -> &Self::Target {
               self.__target
           }
       }
    };

    if *is_mut {
        output.extend(quote! {
            impl<#all_generics_params_with_bounds> core::ops::DerefMut for __Self<#generic_params>
            #where_clause
            {
                fn deref_mut(&mut self) -> &Self::Target {
                    self.__target
                }
            }
        })
    }

    output.extend(quote! {
        impl<#all_generics_params_with_bounds> __Self<#generic_params> #where_clause
    });

    output
}

struct State {
    first_fn_arg: FnArg,
    returned_last_ty: Type,
    test_index: usize,
    returns_last: bool,
    last_test_fn_ident: Option<Ident>,
    capture_fn_idents: Vec<Ident>,
}

#[derive(Default)]
struct ExpandTreeInnerOutput {
    extra: TokenStream,
    body: TokenStream,
}

impl State {
    fn new(idx_arg: &GenericArgument, ch_arg: &GenericArgument) -> Self {
        Self {
            first_fn_arg: parse_quote! { first: (#idx_arg, #ch_arg) },
            returned_last_ty: parse_quote! { core::option::Option<#ch_arg> },
            test_index: 0,
            returns_last: false,
            last_test_fn_ident: None,
            capture_fn_idents: Vec::with_capacity(64),
        }
    }

    fn set_next_test_ident(&mut self) {
        self.last_test_fn_ident = Some(format_ident!("__test{}", self.test_index));
        self.test_index += 1;
    }

    fn expand_tree(
        &mut self,
        Tree {
            inner, mut capture, ..
        }: Tree,
    ) -> syn::Result<TokenStream> {
        todo!()
    }

    fn expand_tree_inner(&mut self, inner: TreeInner) -> syn::Result<ExpandTreeInnerOutput> {
        match inner {
            TreeInner::All(group) => self.expand_tree_inner_all(group),
            TreeInner::Any(group) => self.expand_tree_inner_any(group),
            TreeInner::Test(expr) => Ok(self.expand_tree_inner_test(expr)),
        }
    }

    fn expand_tree_inner_all(&mut self, group: Group) -> syn::Result<ExpandTreeInnerOutput> {
        let mut output = ExpandTreeInnerOutput::default();
        for expr in group {
            let has_first = self.returns_last;
            output.extra.extend(self.expand_tree(expr.try_into()?)?);
            let call = self.expand_test_call(has_first);
            output.body.extend(if self.returns_last {
                quote! { let first = if let Some(first) = #call { first } else { return None; } }
            } else {
                quote! { if #call.is_none() { return None } }
            });
        }
        Ok(output)
    }

    fn expand_tree_inner_any(&mut self, group: Group) -> syn::Result<ExpandTreeInnerOutput> {
        Err(syn::Error::new_spanned(
            group,
            "any-patterns are not implemented yet",
        ))
    }

    fn expand_tree_inner_test(&self, expr: &Expr) -> ExpandTreeInnerOutput {
        let mut body = if self.returns_last {
            TokenStream::default()
        } else {
            quote! { let first = if let Some(x) = self.__iter.next() { x } else { return Ok(None); }; }
        };
        body.extend(quote! { if (#expr)(first.1) { self.end = first.0 } else { return Ok(None); }; });
        ExpandTreeInnerOutput {
            body,
            extra: TokenStream::new(),
        }
    }

    fn expand_test_sig(&self, has_first: bool) -> TokenStream {
        let test_ident = &self.last_test_fn_ident;
        let first = has_first.then_some(&self.first_fn_arg);
        let mut output = quote! {
            fn #test_ident(&mut self, #(#first)?) ->
        };
        if self.returns_last {
            self.returned_last_ty.to_tokens(&mut output);
        } else {
            output.extend(quote! { core::option::Option<()> });
        }
        output
    }

    fn expand_test_call(&self, has_first: bool) -> TokenStream {
        let test_ident = &self.last_test_fn_ident;
        let first = has_first.then_some(&self.first_fn_arg);
        quote! {
            #test_ident(self, #(#first)?)
        }
    }
}
