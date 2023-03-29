use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{GenericArgument, Visibility};

pub struct Field {
    ident: Ident,
    max_set_count: usize,
}

pub fn capture<'a>(
    vis: &Visibility,
    ident: &Ident,
    idx_ident: &Ident,
    idx_default_arg: Option<GenericArgument>,
    field_idents: impl Iterator<Item = &'a Ident> + Clone,
) -> TokenStream {
    let field_idents_clone = field_idents.clone();
    quote! {
        #[derive(
            core::clone::Clone,
            core::cmp::Eq,
            core::cmp::PartialEq,
            core::fmt::Debug,
        )]
        #vis struct #ident<#idx_ident = #idx_default_arg> {
            #(
                #field_idents: core::option::Option<core::ops::Range<#idx_ident>>,
            )*
        }

        impl<#idx_ident> core::default::Default for #ident<#idx_ident> {
            fn default() -> Self {
                Self {
                    #(
                        #field_idents_clone: None,
                    )*
                }
            }
        }
    }
}

pub fn capture_vecs<'a>(
    hitori_ident: &Ident,
    fields: impl Iterator<Item = &'a Field> + Clone,
) -> TokenStream {
    let field_idents = fields.clone().map(|field| &field.ident);
    let field_idents_clone = field_idents.clone();
    let field_max_set_counts =
        fields.map(|field| proc_macro2::Literal::usize_unsuffixed(field.max_set_count));
    quote! {
        struct CaptureVecs {
            #(
                #field_idents: #hitori_ident::__arrayvec::ArrayVec<Idx, #field_max_set_counts>,
            )*
        }

        impl<Idx> core::default::Default for CaptureVecs<Idx> {
            fn default() -> Self {
                Self {
                    #(
                        #field_idents_clone: core::default::Default::default(),
                    )*
                }
            }
        }
    }
}
