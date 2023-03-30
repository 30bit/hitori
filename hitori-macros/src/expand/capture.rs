use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Type, Visibility};

pub struct Field {
    pub ident: Ident,
    pub max_set_count: usize,
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident
    }
}

impl Eq for Field {}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ident.partial_cmp(&other.ident)
    }
}

impl Ord for Field {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ident.cmp(&other.ident)
    }
}

pub fn options<'a>(
    vis: &Visibility,
    ident: &Ident,
    idx_ident: &Ident,
    default_idx_ty: Option<&Type>,
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
        #vis struct #ident<#idx_ident = #default_idx_ty> {
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

pub fn vecs<'a>(
    hitori_ident: &Ident,
    ident: &Ident,
    options_ident: &Ident,
    fields: impl Iterator<Item = &'a Field> + Clone,
) -> TokenStream {
    let field_idents = fields.clone().map(|field| &field.ident);
    let field_idents1 = field_idents.clone();
    let field_idents2 = field_idents.clone();
    let field_idents3 = field_idents.clone();
    let field_max_set_counts =
        fields.map(|field| proc_macro2::Literal::usize_unsuffixed(field.max_set_count));
    quote! {
        struct #ident<Idx> {
            #(
                #field_idents: #hitori_ident::__arrayvec::ArrayVec<Idx, #field_max_set_counts>,
            )*
        }

        impl<Idx> core::default::Default for #ident<Idx> {
            fn default() -> Self {
                Self {
                    #(
                        #field_idents1: core::default::Default::default(),
                    )*
                }
            }
        }

        impl<Idx> #ident<Idx> {
            fn into_options(mut self) -> #options_ident<Idx> {
                #options_ident {
                    #(
                        #field_idents2: self.#field_idents3.pop(),
                    )*
                }
            }
        }
    }
}
