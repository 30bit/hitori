use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parse, parse_quote, punctuated::Punctuated, Expr, ExprRange, GenericArgument,
    GenericParam, LifetimeDef, Path, Token, Type, WhereClause,
};

use crate::utils::{
    collect_hitori_attrs, find_unique_hitori_attr, remove_generic_params_bounds, take_hitori_attrs,
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

pub struct Input<'a> {
    pub hitori_ident: &'a Ident,
    pub generic_params: &'a mut Punctuated<GenericParam, Token![,]>,
    pub where_clause: Option<&'a WhereClause>,
    pub trait_args: &'a [GenericArgument; 3],
    pub self_path: &'a Path,
    pub expr: &'a mut Expr,
    pub is_mut: bool,
}

pub struct Output {
    pub tokens: TokenStream,
    pub capture: Vec<Ident>, // used for #[hitori::add_define]
}

impl<'a> TryFrom<Input<'a>> for Output {
    type Error = syn::Error;

    fn try_from(mut input: Input<'a>) -> syn::Result<Self> {
        let tree = input.expr.try_into()?;
        let mut st = State::new(input.hitori_ident, &input.trait_args[0]);
        let wrapper_impl_body = st.expand_tree(tree)?;
        Ok(if wrapper_impl_body.is_empty() {
            Self {
                tokens: quote! { Ok(Some(..start)) },
                capture: vec![],
            }
        } else {
            let mut output_tokens = expand_wrapper_header(&mut input);
            let last_subexpr_fn_ident = st.last_subexpr_fn_ident;
            output_tokens.extend(quote! {
                { #wrapper_impl_body }
                let mut wrapper = __Self {
                    __target: self,
                    __capture: capture,
                    __end: start,
                    __iter: iter.into_iter(),
                    __phantom: core::marker::PhantomData,
                };
                wrapper.#last_subexpr_fn_ident().map(|matches| {
                    if matches {
                        Some(..wrapper.__end)
                    } else {
                        None
                    }
                })
            });
            st.capture_fn_idents.sort_unstable();
            st.capture_fn_idents.dedup();
            Self {
                tokens: output_tokens,
                capture: st.capture_fn_idents,
            }
        })
    }
}

fn expand_lifetime_generic_params_into_punctuated_unit_refs<'a, I>(iter: I) -> TokenStream
where
    I: IntoIterator<Item = &'a LifetimeDef>,
{
    let mut output = TokenStream::new();
    for LifetimeDef { lifetime, .. } in iter {
        output.extend(quote! { & #lifetime (), });
    }
    output
}

fn expand_wrapper_header(
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

    let mut phantom_data_params = expand_lifetime_generic_params_into_punctuated_unit_refs(
        generic_params
            .iter()
            .take_while(|param| matches!(param, GenericParam::Lifetime(_)))
            .map(|param| match param {
                GenericParam::Lifetime(l) => l,
                _ => unreachable!(),
            }),
    );

    remove_generic_params_bounds(generic_params);

    for pair in generic_params.pairs() {
        if !matches!(pair.value(), GenericParam::Const(_)) {
            pair.to_tokens(&mut phantom_data_params);
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
            __I: core::iter::Iterator<Item = (#idx_arg, #ch_arg)> + Clone,
        });
        output
    };

    let mut_ = is_mut.then_some(<Token![mut]>::default());

    let mut output = quote! {
       struct __Self<'a, __I, #all_generics_params_with_bounds> #where_clause {
           __target: &'a #mut_ #self_path,
           __capture: &'a mut #capture_arg,
           __end: #idx_arg,
           __iter: __I,
           __phantom: core::marker::PhantomData<(#phantom_data_params)>,
       };

       impl<'a, __I, #all_generics_params_with_bounds> core::ops::Deref
       for __Self<'a, __I, #generic_params>
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
            impl<'a, __I, #all_generics_params_with_bounds> core::ops::DerefMut
            for __Self<'a, __I, #generic_params>
            #where_clause
            {
                fn deref_mut(&mut self) -> &Self::Target {
                    self.__target
                }
            }
        })
    }

    output.extend(quote! {
        impl<'a, __I, #all_generics_params_with_bounds> __Self<'a, __I, #generic_params>
        #where_clause
    });

    output
}

struct State {
    returned_ty: Type,
    capture_clear_call: Expr,
    subexpr_index: usize,
    last_subexpr_fn_ident: Option<Ident>,
    capture_fn_idents: Vec<Ident>,
}

#[derive(Default)]
struct ExpandTreeInnerOutput {
    extra: TokenStream,
    body: TokenStream,
}

impl State {
    fn new(hitori_ident: &Ident, capture_arg: &GenericArgument) -> Self {
        Self {
            returned_ty: parse_quote! {
                core::result::Result<
                    bool,
                    <#capture_arg as #hitori_ident::CaptureMut>::Error
                >
            },
            capture_clear_call: parse_quote! {
                <#capture_arg as #hitori_ident::CaptureMut>::clear(self.__capture)
            },
            subexpr_index: 0,
            last_subexpr_fn_ident: None,
            capture_fn_idents: Vec::with_capacity(64),
        }
    }

    fn set_next_subexpr_fn_ident(&mut self) {
        self.last_subexpr_fn_ident = Some(format_ident!("__subexpr{}", self.subexpr_index));
        self.subexpr_index += 1;
    }

    fn expand_tree(
        &mut self,
        Tree {
            inner,
            repeat,
            mut capture,
        }: Tree,
    ) -> syn::Result<TokenStream> {
        let ExpandTreeInnerOutput {
            extra: mut output,
            body: inner_body,
        } = self.expand_tree_inner(inner)?;

        self.set_next_subexpr_fn_ident();
        output.extend(self.expand_subexpr_sig());
        output.extend(quote! {{ #inner_body }});

        if capture.is_empty() && repeat.is_none() {
            return Ok(output);
        }

        let mut body = if capture.is_empty() {
            TokenStream::new()
        } else {
            quote! { let start = self.__end.clone(); }
        };

        if repeat.is_none() {
            let inner_call = self.expand_subexpr_call();
            body.extend(quote! {
                let matches = self.#inner_call;
                match &matches {
                    Ok(true) => (),
                    _ => return matches,
                }
            })
        } else {
            unimplemented!();
        }

        if !capture.is_empty() {
            for f in &capture[..capture.len() - 1] {
                body.extend(quote! {
                    self.__capture.#f(start.clone()..self.__end.clone())?;
                });
            }
            let f = capture.last().unwrap();
            body.extend(quote! { self.__capture.#f(start..self.__end.clone())?; });
        }

        self.set_next_subexpr_fn_ident();
        output.extend(self.expand_subexpr_sig());

        output.extend(quote! {{
            #body
            core::result::Result::Ok(true)
        }});

        self.capture_fn_idents.append(&mut capture);

        Ok(output)
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
            output.extra.extend(self.expand_tree(expr.try_into()?)?);
            let call = self.expand_subexpr_call();
            output.body.extend(quote! {
                let matches = self.#call;
                match &matches {
                    Ok(true) => (),
                    _ => return matches,
                }
            });
        }
        output.body.extend(quote! {
            core::result::Result::Ok(true)
        });
        Ok(output)
    }

    fn expand_tree_inner_any(&mut self, group: Group) -> syn::Result<ExpandTreeInnerOutput> {
        let mut output = ExpandTreeInnerOutput::default();
        if group.len() > 1 {
            output.body.extend(quote! {
                let cloned_iter = self.__iter.clone();
            });
        }

        macro_rules! expand_branch {
            ($expr:expr, $reset:expr) => {{
                output.extra.extend(self.expand_tree($expr.try_into()?)?);
                let call = self.expand_subexpr_call();
                let reset = $reset;
                output.body.extend(quote! {
                    let matches = self.#call;
                    match &matches {
                        Ok(true) => return matches,
                        _ => #reset,
                    }
                });
            }};
        }

        let group_len = group.len();
        if group_len > 2 {
            let capture_clear_call = &self.capture_clear_call;
            let reset = quote! {{
                self.__iter = cloned_iter.clone();
                #capture_clear_call;
            }};
            for expr in group.iter_mut().take(group_len - 2) {
                expand_branch!(expr, &reset);
            }
        }
        if group_len > 1 {
            let capture_clear_call = &self.capture_clear_call;
            let reset = quote! {{
                self.__iter = cloned_iter;
                #capture_clear_call;
            }};
            expand_branch!(&mut group[group_len - 2], reset);
        }
        if group_len == 0 {
            output
                .body
                .extend(quote! { core::result::Result::Ok(false) });
        } else {
            expand_branch!(&mut group[group_len - 1], quote! { return matches });
        }
        Ok(output)
    }

    fn expand_tree_inner_test(&self, expr: &Expr) -> ExpandTreeInnerOutput {
        let mut body = quote! {
            let first = if let Some(first) = self.__iter.next() {
                first
            } else {
                return core::result::Result::Ok(false);
            };
        };
        body.extend(quote! {
            core::result::Result::Ok(if (#expr)(first.1) {
                self.__end = first.0;
                true
            } else {
                false
            })
        });
        ExpandTreeInnerOutput {
            body,
            extra: TokenStream::new(),
        }
    }

    fn expand_subexpr_sig(&self) -> TokenStream {
        let test_ident = &self.last_subexpr_fn_ident;
        let returned_ty = &self.returned_ty;
        quote! {
            fn #test_ident(&mut self) -> #returned_ty
        }
    }

    fn expand_subexpr_call(&self) -> TokenStream {
        let test_ident = &self.last_subexpr_fn_ident;
        quote! {
            #test_ident()
        }
    }
}
