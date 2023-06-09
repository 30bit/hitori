use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::{convert, fmt::Write as _, mem};
use syn::{
    punctuated::Punctuated, AssocType, Attribute, BinOp, Expr, ExprBinary, ExprLit,
    GenericArgument, GenericParam, LifetimeParam, Lit, ParenthesizedGenericArguments, Path,
    PathArguments, ReturnType, Token, Type, TypeImplTrait, TypeParam, TypeParamBound, TypeParen,
    TypePath, TypePtr, TypeReference, TypeTraitObject,
};

pub fn hitori_ident() -> Ident {
    #[cfg(feature = "find-hitori")]
    match proc_macro_crate::crate_name("hitori").expect("expected `hitori` package in `Cargo.toml`")
    {
        proc_macro_crate::FoundCrate::Itself => format_ident!("hitori"),
        proc_macro_crate::FoundCrate::Name(name) => format_ident!("{name}"),
    }
    #[cfg(not(feature = "find-hitori"))]
    format_ident!("hitori")
}

pub fn hitori_attr_ident_eq_str(attr: &Attribute, s: &str) -> bool {
    let segments = &attr.path().segments;
    assert!(segments.len() == 2, "bug");
    assert_eq!(segments[0].ident, "hitori", "bug");
    segments[1].ident == s
}

fn is_hitori_attr_path(attr_path: &Path) -> bool {
    attr_path.leading_colon.is_none()
        && !attr_path.segments.empty_or_trailing()
        && attr_path.segments[0].arguments.is_empty()
        && attr_path.segments[0].ident == "hitori"
}

fn find_hitori_attr_index(attrs: &[Attribute]) -> Option<usize> {
    attrs
        .iter()
        .position(|attr| is_hitori_attr_path(attr.path()) && attr.path().segments.len() == 2)
}

struct FindHitoriAttrsIndices<'a>(&'a [Attribute]);

impl<'a> Iterator for FindHitoriAttrsIndices<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }
        if let Some(index) = find_hitori_attr_index(self.0) {
            self.0 = &self.0[(index + 1)..];
            Some(index)
        } else {
            self.0 = &[];
            None
        }
    }
}

pub fn find_le_one_hitori_attr(attrs: &[Attribute]) -> Result<Option<&Attribute>, [&Attribute; 2]> {
    let mut indices = FindHitoriAttrsIndices(attrs);
    if let Some(mut first_index) = indices.next() {
        if let Some(mut second_index) = indices.next() {
            for next_index in indices {
                first_index = mem::replace(&mut second_index, next_index);
            }
            Err([&attrs[first_index], &attrs[second_index]])
        } else {
            Ok(Some(&attrs[first_index]))
        }
    } else {
        Ok(None)
    }
}

pub fn eq_by_fmt<Lhs: ToTokens, Rhs: ToTokens>(lhs: Lhs, rhs: Rhs) -> bool {
    let mut buf = String::with_capacity(128);
    write!(buf, "{}", lhs.into_token_stream()).unwrap();
    let lhs_end = buf.len();
    write!(buf, "{}", rhs.into_token_stream()).unwrap();
    buf[..lhs_end] == buf[lhs_end..]
}

pub fn path_eq_ident_str(path: &Path, ident_str: &str) -> bool {
    path.get_ident()
        .map(|ident| ident == ident_str)
        .unwrap_or_default()
}

pub fn lifetimes_into_punctuated_unit_refs<'a>(
    iter: impl IntoIterator<Item = &'a LifetimeParam>,
) -> TokenStream {
    let mut output = TokenStream::new();
    for LifetimeParam { lifetime, .. } in iter {
        output.extend(quote! { & #lifetime (), });
    }
    output
}

pub fn generic_arg_try_into_type(arg: GenericArgument) -> syn::Result<Type> {
    match &arg {
        GenericArgument::Type(_) => match arg {
            GenericArgument::Type(ty) => Ok(ty),
            _ => unreachable!(),
        },
        _ => Err(syn::Error::new_spanned(arg, "expected type")),
    }
}

pub fn ident_not_in_generic_params(
    params: &Punctuated<GenericParam, Token![,]>,
    init: String,
) -> Ident {
    unique_ident(
        &params.iter().filter_map(|param| match param {
            GenericParam::Type(TypeParam { ident, .. }) => Some(ident),
            _ => None,
        }),
        init,
    )
}

pub fn unique_ident<'a>(
    idents: &(impl Iterator<Item = &'a Ident> + Clone),
    mut init: String,
) -> Ident {
    while idents.clone().any(|ident| ident == &init) {
        init.push('_');
    }

    format_ident!("{init}")
}

pub fn type_as_type_path(ty: &Type) -> Option<&TypePath> {
    macro_rules! next {
        ($ty:expr) => {
            match $ty {
                Type::Paren(TypeParen { elem, .. })
                | Type::Reference(TypeReference { elem, .. })
                | Type::Ptr(TypePtr { elem, .. }) => elem,
                Type::Path(path) => return Some(path),
                _ => return None,
            }
        };
    }
    let mut ty = next!(ty);
    loop {
        ty = next!(ty.as_ref());
    }
}

pub fn expr_add_one_usize(expr: Expr) -> Expr {
    Expr::Binary(ExprBinary {
        attrs: vec![],
        left: Box::new(expr),
        op: BinOp::Add(<Token![+]>::default()),
        right: Box::new(Expr::Lit(ExprLit {
            attrs: vec![],
            lit: Lit::Int(Literal::usize_suffixed(1).into()),
        })),
    })
}

pub fn expr_try_from_lit_int_or_lit_str_expr(expr: Expr) -> syn::Result<Expr> {
    if let Expr::Lit(lit) = &expr {
        match &lit.lit {
            Lit::Int(_) => return Ok(expr),
            Lit::Str(s) => return s.parse(),
            _ => (),
        }
    }
    Err(syn::Error::new_spanned(
        expr,
        "expected either a literal `usize` or an expression \
            within literal string",
    ))
}

fn is_any_generic_param_eq_ident(
    params: &Punctuated<GenericParam, Token![,]>,
    ident: &Ident,
) -> bool {
    params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(TypeParam { ident, .. }) => Some(ident),
            _ => None,
        })
        .any(|param_ident| ident == param_ident)
}

fn is_any_generic_param_eq_path_prefix(
    params: &Punctuated<GenericParam, Token![,]>,
    path: &Path,
) -> bool {
    if params.is_empty() {
        false
    } else {
        is_any_generic_param_eq_ident(params, &path.segments[0].ident)
    }
}

fn is_any_generic_param_in_path_args(
    params: &Punctuated<GenericParam, Token![,]>,
    path: &Path,
) -> bool {
    path.segments
        .iter()
        .map(|segment| match &segment.arguments {
            PathArguments::AngleBracketed(args) => args
                .args
                .iter()
                .any(|arg| has_generic_arg_any_generic_params(params, arg)),
            PathArguments::Parenthesized(ParenthesizedGenericArguments {
                inputs,
                output: ReturnType::Type(_, output),
                ..
            }) => {
                has_type_any_generic_params(params, output)
                    && inputs
                        .iter()
                        .any(|input| has_type_any_generic_params(params, input))
            }
            _ => false,
        })
        .any(convert::identity)
}

pub fn has_path_any_generic_params(
    params: &Punctuated<GenericParam, Token![,]>,
    path: &Path,
) -> bool {
    is_any_generic_param_eq_path_prefix(params, path)
        || is_any_generic_param_in_path_args(params, path)
}

pub fn has_type_path_any_generic_params(
    params: &Punctuated<GenericParam, Token![,]>,
    ty: &TypePath,
) -> bool {
    if has_path_any_generic_params(params, &ty.path) {
        true
    } else if let Some(qself) = &ty.qself {
        has_type_any_generic_params(params, &qself.ty)
    } else {
        false
    }
}

pub fn has_type_param_bound_any_generic_params(
    params: &Punctuated<GenericParam, Token![,]>,
    bound: &TypeParamBound,
) -> bool {
    match bound {
        TypeParamBound::Trait(bound) => is_any_generic_param_in_path_args(params, &bound.path),
        _ => false,
    }
}

pub fn has_type_any_generic_params(
    params: &Punctuated<GenericParam, Token![,]>,
    ty: &Type,
) -> bool {
    if let Some(path) = type_as_type_path(ty) {
        has_type_path_any_generic_params(params, path)
    } else if let Type::ImplTrait(TypeImplTrait { bounds, .. })
    | Type::TraitObject(TypeTraitObject { bounds, .. }) = ty
    {
        bounds
            .iter()
            .any(|bound| has_type_param_bound_any_generic_params(params, bound))
    } else {
        false
    }
}

pub fn has_generic_arg_any_generic_params(
    params: &Punctuated<GenericParam, Token![,]>,
    arg: &GenericArgument,
) -> bool {
    match arg {
        GenericArgument::Type(ty) | GenericArgument::AssocType(AssocType { ty, .. }) => {
            has_type_any_generic_params(params, ty)
        }
        GenericArgument::Constraint(constraint) => constraint
            .bounds
            .iter()
            .any(|bound| has_type_param_bound_any_generic_params(params, bound)),
        _ => false,
    }
}

pub fn remove_generic_params_bounds(params: &mut Punctuated<GenericParam, Token![,]>) {
    for param in params {
        if let GenericParam::Type(ty) = param {
            ty.bounds = Punctuated::new();
        } else if let GenericParam::Lifetime(l) = param {
            l.bounds = Punctuated::new();
        }
    }
}

#[cfg(feature = "debug")]
pub fn debug(tokens: proc_macro2::TokenStream) -> Result<(), Box<dyn std::error::Error>> {
    use rust_format::{Formatter as _, RustFmt};
    use std::{env, fs, path::PathBuf};
    let dir = if let Ok(out_dir) = env::var("CARGO_TARGET_DIR") {
        out_dir.into()
    } else {
        let dir = PathBuf::from("target/hitori");
        fs::create_dir_all(&dir)?;
        dir
    };
    fs::write(
        dir.join("macros_debug.rs"),
        RustFmt::default().format_tokens(tokens)?,
    )
    .map_err(Into::into)
}
