mod expand;
mod parse;
mod utils;

use proc_macro::TokenStream;
use syn::Error;

fn parse_expand<const MUT: bool>(attr: TokenStream, item: TokenStream) -> TokenStream {
    let output = parse::Output::parse::<MUT>(attr.into(), item.into())
        .and_then(expand::expand)
        .unwrap_or_else(Error::into_compile_error);
    #[cfg(feature = "debug")]
    utils::debug(output.clone()).unwrap();
    output.into()
}

#[proc_macro_attribute]
pub fn impl_expr(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_expand::<false>(attr, item)
}

#[proc_macro_attribute]
pub fn impl_expr_mut(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_expand::<true>(attr, item)
}
