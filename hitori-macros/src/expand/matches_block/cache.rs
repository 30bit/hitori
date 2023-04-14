use crate::utils::unique_ident;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub struct CaptureVars<C>(C);

impl<'a, C: Iterator<Item = &'a Ident> + Clone> CaptureVars<C> {
    pub fn new<I: IntoIterator<IntoIter = C>>(capture_idents: I) -> Self {
        Self(capture_idents.into_iter())
    }

    pub fn cache(&self) -> TokenStream {
        let idents = self.0.clone();
        quote! {
            #(
                let mut #idents =
                    ::core::clone::Clone::clone(&self.__capture.#idents);
            )*
        }
    }

    pub fn update(&self) -> TokenStream {
        let idents = self.0.clone();
        quote! {
            #(
                #idents =
                    ::core::clone::Clone::clone(&self.__capture.#idents);
            )*
        }
    }

    pub fn restore(&self) -> TokenStream {
        let idents = self.0.clone();
        quote! {
            #(
                self.__capture.#idents = #idents;
            )*
        }
    }
}


pub struct OtherVars {
    iter: Ident,
    is_first: Ident,
    end: Ident,
}

impl OtherVars {
    pub fn unique_in<'a, C>(capture_idents: C) -> Self
    where
        C: IntoIterator<Item = &'a Ident>,
        C::IntoIter: Clone,
    {
        let capture_idents = capture_idents.into_iter();
        Self {
            iter: unique_ident(capture_idents.clone(), "iter".into()),
            is_first: unique_ident(capture_idents.clone(), "is_first".into()),
            end: unique_ident(capture_idents, "end".into()),
        }
    }

    pub fn cache(&self) -> TokenStream {
        let iter = &self.iter;
        let is_first = &self.is_first;
        let end = &self.end;
        quote! {
            let mut #iter = ::core::clone::Clone::clone(&self.__iter);
            let mut #is_first = ::core::clone::Clone::clone(&self.__is_first);
            let mut #end = ::core::clone::Clone::clone(&self.__end);
        }
    }

    pub fn update(&self) -> TokenStream {
        let iter = &self.iter;
        let is_first = &self.is_first;
        let end = &self.end;
        quote! {
            #iter = ::core::clone::Clone::clone(&self.__iter);
            #is_first = ::core::clone::Clone::clone(&self.__is_first);
            #end = ::core::clone::Clone::clone(&self.__end);
        }
    }

    pub fn restore(&self) -> TokenStream {
        let iter = &self.iter;
        let is_first = &self.is_first;
        let end = &self.end;
        quote! {
            self.__iter = #iter;
            self.__is_first = #is_first;
            self.__end = #end;
        }
    }

    pub fn restore_clone(&self) -> TokenStream {
        let iter = &self.iter;
        let is_first = &self.is_first;
        let end = &self.end;
        quote! {
            self.__iter = ::core::clone::Clone::clone(#iter);
            self.__is_first = #is_first;
            self.__end = ::core::clone::Clone::clone(#end);
        }
    }
}