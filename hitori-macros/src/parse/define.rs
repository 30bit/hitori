use crate::utils::path_eq_ident_str;
use proc_macro2::{Ident, Span};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Lit, Meta, MetaNameValue, Token, VisPublic, Visibility,
};

pub enum ConfigKind {
    Capture,
    CaptureMut,
}

pub struct Config {
    pub kind: ConfigKind,
    pub ranges: Option<Span>, // need a span to put it into expansion errors
    pub vis: Visibility,
    pub prefix: Option<Ident>,
}

impl Config {
    fn new(
        capture: bool,
        capture_mut: bool,
        capture_ranges: Option<Span>,
        with_vis: Option<Visibility>,
        with_prefix: Option<Ident>,
    ) -> std::result::Result<Self, &'static str> {
        if !capture_mut {
            return Err(if capture {
                "expected `capture_mut` when `capture` is present"
            } else if capture_ranges.is_some() {
                "expected `capture_mut` when `capture_ranges` is present"
            } else if with_vis.is_some() {
                "expected `capture_mut` when `with_vis` is present"
            } else if with_prefix.is_some() {
                "expected `capture_mut` when `with_prefix` is present"
            } else {
                "must not be empty"
            });
        }

        Ok(Self {
            kind: if capture {
                ConfigKind::Capture
            } else {
                ConfigKind::CaptureMut
            },
            ranges: capture_ranges,
            vis: with_vis.unwrap_or(Visibility::Public(VisPublic {
                pub_token: <Token![pub]>::default(),
            })),
            prefix: with_prefix,
        })
    }
}

macro_rules! set_if_not_dup_and_continue {
    (dup $tokens:ident if ($if:expr) else $mutable:ident = $value:expr) => {
        if $if {
            return Err(syn::Error::new_spanned(
                $tokens,
                format!("duplicate `{}`", stringify!($mutable)),
            ));
        } else {
            $mutable = $value;
            continue;
        }
    };
    (dup $tokens:ident else $mutable:ident = true) => {
        set_if_not_dup_and_continue!(
            dup $tokens if ($mutable) else $mutable = true
        )
    };
    (dup $tokens:ident else $mutable:ident = Some($value:expr)) => {
        set_if_not_dup_and_continue!(
            dup $tokens if ($mutable.is_some()) else $mutable = Some($value)
        )
    }
}

impl TryFrom<Punctuated<Meta, Token![,]>> for Config {
    type Error = syn::Error;

    fn try_from(args: Punctuated<Meta, Token![,]>) -> syn::Result<Self> {
        let mut capture = false;
        let mut capture_mut = false;
        let mut capture_ranges = None;
        let mut with_prefix = None;
        let mut with_vis = None;

        for arg in &args {
            if let Meta::Path(path) = arg {
                if path_eq_ident_str(path, "capture") {
                    set_if_not_dup_and_continue!(dup path else capture = true);
                } else if path_eq_ident_str(path, "capture_mut") {
                    set_if_not_dup_and_continue!(dup path else capture_mut = true);
                } else if path_eq_ident_str(path, "capture_ranges") {
                    set_if_not_dup_and_continue!(
                        dup path else capture_ranges = Some(path.segments[0].ident.span())
                    );
                }
            } else if let Meta::NameValue(MetaNameValue {
                path,
                lit: Lit::Str(s),
                ..
            }) = arg
            {
                if path_eq_ident_str(path, "with_prefix") {
                    set_if_not_dup_and_continue!(dup path else with_prefix = Some(s.parse()?));
                } else if path_eq_ident_str(path, "with_vis") {
                    set_if_not_dup_and_continue!(dup path else with_vis = Some(s.parse()?));
                }
            }
            return Err(syn::Error::new_spanned(arg, "unexpected argument"));
        }

        Self::new(capture, capture_mut, capture_ranges, with_vis, with_prefix)
            .map_err(|msg| syn::Error::new_spanned(args, msg))
    }
}

impl Parse for Config {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input
            .parse_terminated::<_, Token![,]>(Meta::parse)
            .and_then(TryInto::try_into)
    }
}
