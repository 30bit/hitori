// trait MyExprCapture is created

// explicitly impl<T: MyExprCapture> MyExprCapture for &T {} etc

// impl<C: MyExprCapture> Expr<C> for MyExpr {}

// &mut &mut MyExpr, &&&MyExpr, Box<Box<MyExpr>> all auto impl Expr

// C can be &mut C, &&C, etc

use crate::{
    parse::{
        self,
        args::{Args, ConfigKind},
    },
    utils::{eq_by_fmt, has_generic_arg_any_generic_params},
};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::borrow::Cow;
use syn::{punctuated::Punctuated, GenericArgument, GenericParam, Path, Token, Visibility};

struct InternalInput<'a> {
    hitori_ident: &'a Ident,
    vis: &'a Visibility,
    prefix: Cow<'a, Ident>,
    prefix_mut: Ident,
    fn_idents: &'a [Ident],
    idx_ident: Ident,
    idx_default_bound: Option<TokenStream>,
}

fn capture_mut(
    InternalInput {
        hitori_ident,
        vis,
        prefix_mut,
        fn_idents,
        idx_ident,
        idx_default_bound,
        ..
    }: &InternalInput,
) -> TokenStream {
    quote! {
        #vis trait #prefix_mut<#idx_ident #idx_default_bound>: #hitori_ident::CaptureMut {
            #(
                fn #fn_idents(
                    &mut self,
                    range: core::ops::Range<#idx_ident>
                ) -> core::result::Result<(), <Self as #hitori_ident::CaptureMut>::Error>;
            )*
        }

        impl<C: #prefix_mut<#idx_ident>, #idx_ident> #prefix_mut<#idx_ident> for &mut C {
            #(
                fn #fn_idents(
                    &mut self,
                    range: core::ops::Range<#idx_ident>
                ) -> core::result::Result<(), <Self as #hitori_ident::CaptureMut>::Error> {
                    C::#fn_idents(self, range)
                }
            )*
        }

        impl<#idx_ident> #prefix_mut<#idx_ident> for () {
            #(
                #[inline]
                fn #fn_idents(
                    &mut self,
                    range: core::ops::Range<#idx_ident>
                ) -> core::result::Result<(), <Self as #hitori_ident::CaptureMut>::Error> {
                    Ok(())
                }
            )*
        }
    }
}

fn capture(
    InternalInput {
        hitori_ident,
        vis,
        prefix,
        prefix_mut,
        fn_idents,
        idx_ident,
        idx_default_bound,
    }: &InternalInput,
) -> TokenStream {
    quote! {
        #vis trait #prefix<#idx_ident #idx_default_bound>: #prefix_mut<#idx_ident> + #hitori_ident::Capture {
            #(
                fn #fn_idents(
                    &self,
                    range: core::ops::Range<#idx_ident>
                ) -> core::result::Result<(), <Self as #hitori_ident::CaptureMut>::Error>;
            )*
        }

        impl<C: #prefix<#idx_ident>, #idx_ident> #prefix_mut<#idx_ident> for &C {
            #(
                fn #fn_idents(
                    &mut self,
                    range: core::ops::Range<#idx_ident>
                ) -> core::result::Result<(), <Self as #hitori_ident::CaptureMut>::Error> {
                    <C as #prefix<#idx_ident>>::#fn_idents(self, range)
                }
            )*
        }

        impl<C: #prefix<#idx_ident>, #idx_ident> #prefix<#idx_ident> for &C {
            #(
                fn #fn_idents(
                    &self,
                    range: core::ops::Range<#idx_ident>
                ) ->core::result::Result<(), <Self as #hitori_ident::CaptureMut>::Error> {
                    <C as #prefix<#idx_ident>>::#fn_idents(self, range)
                }
            )*
        }

        impl<#idx_ident> #prefix<#idx_ident> for () {
            #(
                #[inline]
                fn #fn_idents(
                    &self,
                    range: core::ops::Range<#idx_ident>
                ) -> core::result::Result<(), <Self as #hitori_ident::CaptureMut>::Error> {
                    Ok(())
                }
            )*
        }
    }
}

#[cfg(feature = "box")]
fn impl_for_box(
    InternalInput {
        hitori_ident,
        prefix,
        prefix_mut,
        fn_idents,
        idx_ident,
        ..
    }: &InternalInput,
    and_capture: bool,
) -> TokenStream {
    let alloc_ident = format_ident!("__hitori_alloc_for_box_impl_for_{prefix_mut}");
    let mut output = quote! {
        mod #alloc_ident {
            extern crate alloc;

            pub use alloc::boxed::Box;
        }

        impl<C: #prefix_mut<#idx_ident>, #idx_ident> #prefix_mut<#idx_ident> for #alloc_ident::Box<C> {
            #(
                fn #fn_idents(
                    &mut self,
                    range: core::ops::Range<#idx_ident>
                ) -> core::result::Result<(), <Self as #hitori_ident::CaptureMut>::Error> {
                    C::#fn_idents(self, range)
                }
            )*
        }
    };

    if and_capture {
        output.extend(quote! {
            impl<C: #prefix<#idx_ident>, #idx_ident> #prefix<#idx_ident> for #alloc_ident::Box<C> {
                #(
                    fn #fn_idents(
                        &self,
                        range: core::ops::Range<#idx_ident>
                    ) -> core::result::Result<(), <Self as #hitori_ident::CaptureMut>::Error> {
                        <C as #prefix<#idx_ident>>::#fn_idents(self, range)
                    }
                )*
            }
        });
    }

    output
}

fn capture_ranges(
    InternalInput {
        hitori_ident,
        vis,
        prefix,
        prefix_mut,
        fn_idents,
        idx_ident,
        idx_default_bound,
    }: &InternalInput,
) -> TokenStream {
    let prefix_ranges = format_ident!("{prefix}Ranges");
    quote! {
        #[derive(Clone, Eq, PartialEq, Default)]
        #vis struct #prefix_ranges<#idx_ident #idx_default_bound> {
            #(
                #vis #fn_idents: core::option::Option<core::ops::Range<#idx_ident>>,
            )*
        }

        impl<#idx_ident> #hitori_ident::CaptureMut for #prefix_ranges<#idx_ident> {
            type Error = core::convert::Infallible;

            fn clear(&mut self) {
                #(
                    self.#fn_idents = None;
                )*
            }
        }

        impl<#idx_ident> #prefix_mut<#idx_ident> for #prefix_ranges<#idx_ident> {
            #(
                #[inline]
                fn #fn_idents(
                    &mut self,
                    range: core::ops::Range<#idx_ident>
                ) -> core::result::Result<(), <Self as #hitori_ident::CaptureMut>::Error> {
                    self.#fn_idents = Some(range);
                    core::result::Result::Ok(())
                }
            )*
        }
    }
}

pub struct Input<'a> {
    pub hitori_ident: &'a Ident,
    pub config: &'a Args,
    pub generic_params: &'a Punctuated<GenericParam, Token![,]>,
    pub self_path: &'a Path,
    pub trait_idx_arg: &'a GenericArgument,
    pub fn_idents: &'a [Ident],
}

impl<'a> Input<'a> {
    pub fn new(
        hitori_ident: &'a Ident,
        parsed: &'a parse::Output,
        fn_idents: &'a [Ident],
    ) -> Option<Self> {
        parsed.define_config.as_ref().map(|config| Self {
            hitori_ident,
            config,
            generic_params: &parsed.generic_params,
            self_path: &parsed.self_path,
            trait_idx_arg: &parsed.trait_args[1],
            fn_idents,
        })
    }
}

impl<'a> TryFrom<Input<'a>> for TokenStream {
    type Error = syn::Error;

    fn try_from(input: Input<'a>) -> syn::Result<Self> {
        let idx_arg = input.trait_idx_arg;
        let idx_default_bound =
            (!has_generic_arg_any_generic_params(input.generic_params, input.trait_idx_arg))
                .then(|| quote! { = #idx_arg });
        let mut idx_ident = format_ident!("Idx");
        if idx_default_bound.is_some() && eq_by_fmt(idx_arg, &idx_ident) {
            idx_ident = format_ident!("I");
        }

        let self_path_last_ident = &input.self_path.segments.last().unwrap().ident;
        let prefix = input
            .config
            .capture_ident
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(format_ident!("{self_path_last_ident}Capture")));

        let internal_input = InternalInput {
            hitori_ident: input.hitori_ident,
            vis: &input.config.vis,
            prefix_mut: format_ident!("{prefix}Mut"),
            prefix,
            fn_idents: input.fn_idents,
            idx_ident,
            idx_default_bound,
        };

        let mut output = capture_mut(&internal_input);

        if matches!(input.config.kind, ConfigKind::Capture) {
            output.extend(capture(&internal_input));
        }

        if let Some(span) = input.config.ranges {
            if input.fn_idents.is_empty() {
                return Err(syn::Error::new(
                    span,
                    "cannot be defined if nothing is captured",
                ));
            }
            output.extend(capture_ranges(&internal_input));
        }

        #[cfg(feature = "box")]
        output.extend(impl_for_box(
            &internal_input,
            matches!(input.config.kind, ConfigKind::Capture),
        ));

        Ok(output)
    }
}
