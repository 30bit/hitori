mod define;
mod impl_;

use crate::{parse, utils::hitori_ident};
use proc_macro2::TokenStream;

pub fn expand(mut parsed: parse::Output) -> syn::Result<TokenStream> {
    let hitori_ident = hitori_ident();
    impl_::Input::new(&hitori_ident, &mut parsed)
        .try_into()
        .and_then(
            |impl_::Output {
                 mut tokens,
                 capture_fn_idents,
             }| {
                if let Some(define_input) =
                    define::Input::new(&hitori_ident, &parsed, &capture_fn_idents)
                {
                    tokens.extend::<TokenStream>(define_input.try_into()?);
                }
                Ok(tokens)
            },
        )
}
