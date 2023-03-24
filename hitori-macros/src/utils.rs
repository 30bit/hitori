use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use std::{borrow::BorrowMut, convert, fmt::Write as _, mem};
use syn::{
    parse::Parse, punctuated::Punctuated, Attribute, Binding, Expr, ExprArray, ExprAssign,
    ExprAssignOp, ExprAsync, ExprAwait, ExprBinary, ExprBlock, ExprBox, ExprBreak, ExprCall,
    ExprCast, ExprClosure, ExprContinue, ExprField, ExprForLoop, ExprGroup, ExprIf, ExprIndex,
    ExprLet, ExprLit, ExprLoop, ExprMacro, ExprMatch, ExprMethodCall, ExprParen, ExprPath,
    ExprRange, ExprReference, ExprRepeat, ExprReturn, ExprStruct, ExprTry, ExprTryBlock, ExprTuple,
    ExprType, ExprUnary, ExprUnsafe, ExprWhile, ExprYield, GenericArgument, GenericParam,
    ParenthesizedGenericArguments, Path, PathArguments, ReturnType, Token, Type, TypeImplTrait,
    TypeParam, TypeParamBound, TypeParen, TypePath, TypePtr, TypeReference, TypeTraitObject,
};

pub fn hitori_ident() -> Ident {
    #[cfg(feature = "proc-macro-crate")]
    match proc_macro_crate::crate_name("hitori").expect("expected `hitori` package in `Cargo.toml`")
    {
        proc_macro_crate::FoundCrate::Itself => format_ident!("hitori"),
        proc_macro_crate::FoundCrate::Name(name) => format_ident!("{name}"),
    }
    #[cfg(not(feature = "proc-macro-crate"))]
    format_ident!("hitori")
}

fn is_hitori_attr_path(attr_path: &Path) -> bool {
    attr_path.leading_colon.is_none()
        && !attr_path.segments.empty_or_trailing()
        && attr_path.segments[0].arguments.is_empty()
        && attr_path.segments[0].ident == "hitori"
}

fn find_hitori_attr_index(attrs: &[Attribute], suffix: &str) -> Option<usize> {
    attrs.iter().position(|attr| {
        is_hitori_attr_path(&attr.path)
            && attr.path.segments.len() == 2
            && attr.path.segments[1].ident == suffix
    })
}

fn find_unique_hitori_attr_index(attrs: &[Attribute], suffix: &str) -> syn::Result<Option<usize>> {
    let Some(index) = find_hitori_attr_index(attrs, suffix) else {
        return Ok(None);
    };
    match find_hitori_attr_index(&attrs[(index + 1)..], suffix) {
        Some(dup_index) => Err(syn::Error::new_spanned(
            &attrs[dup_index],
            format!("duplicate `{suffix}`"),
        )),
        None => Ok(Some(index)),
    }
}

pub fn find_unique_hitori_attr<T: Parse>(
    attrs: &[Attribute],
    suffix: &str,
) -> syn::Result<Option<T>> {
    find_unique_hitori_attr_index(attrs, suffix)
        .and_then(|found| found.map(|index| attrs[index].parse_args()).transpose())
}

struct FindHitoriAttrsIndices<'a> {
    attrs: &'a [Attribute],
    suffix: &'a str,
}

impl<'a> Iterator for FindHitoriAttrsIndices<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.attrs.is_empty() {
            return None;
        }
        match find_hitori_attr_index(self.attrs, self.suffix) {
            Some(index) => {
                self.attrs = &self.attrs[(index + 1)..];
                Some(index)
            }
            None => {
                self.attrs = &[];
                None
            }
        }
    }
}

pub fn collect_hitori_attrs<T: Parse>(attrs: &[Attribute], suffix: &str) -> syn::Result<Vec<T>> {
    FindHitoriAttrsIndices { attrs, suffix }
        .map(|index| attrs[index].parse_args())
        .collect()
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

pub fn type_path_ref(ty: &Type) -> Option<&TypePath> {
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

pub fn take_type_path<T: BorrowMut<Type>>(mut ty: T) -> Option<TypePath> {
    macro_rules! next {
        ($ty:expr) => {
            match $ty.borrow_mut() {
                Type::Paren(TypeParen { elem, .. })
                | Type::Reference(TypeReference { elem, .. })
                | Type::Ptr(TypePtr { elem, .. }) => elem,
                Type::Path(TypePath {
                    qself,
                    path:
                        Path {
                            leading_colon,
                            segments,
                        },
                }) => {
                    return Some(TypePath {
                        qself: qself.take(),
                        path: Path {
                            leading_colon: leading_colon.take(),
                            segments: mem::take(segments),
                        },
                    })
                }
                _ => return None,
            }
        };
    }
    let mut ty = next!(ty);
    loop {
        ty = next!(ty);
    }
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
        TypeParamBound::Lifetime(_) => false,
    }
}

pub fn has_type_any_generic_params(
    params: &Punctuated<GenericParam, Token![,]>,
    ty: &Type,
) -> bool {
    if let Some(path) = type_path_ref(ty) {
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
        GenericArgument::Type(ty) | GenericArgument::Binding(Binding { ty, .. }) => {
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
            ty.bounds = Punctuated::new()
        } else if let GenericParam::Lifetime(l) = param {
            l.bounds = Punctuated::new()
        }
    }
}

pub fn take_hitori_attrs(expr: &mut Expr) -> Vec<Attribute> {
    match expr {
        Expr::Array(ExprArray { attrs, .. })
        | Expr::Tuple(ExprTuple { attrs, .. })
        | Expr::Assign(ExprAssign { attrs, .. })
        | Expr::AssignOp(ExprAssignOp { attrs, .. })
        | Expr::Async(ExprAsync { attrs, .. })
        | Expr::Await(ExprAwait { attrs, .. })
        | Expr::Binary(ExprBinary { attrs, .. })
        | Expr::Block(ExprBlock { attrs, .. })
        | Expr::Box(ExprBox { attrs, .. })
        | Expr::Break(ExprBreak { attrs, .. })
        | Expr::Call(ExprCall { attrs, .. })
        | Expr::Cast(ExprCast { attrs, .. })
        | Expr::Closure(ExprClosure { attrs, .. })
        | Expr::Continue(ExprContinue { attrs, .. })
        | Expr::Field(ExprField { attrs, .. })
        | Expr::ForLoop(ExprForLoop { attrs, .. })
        | Expr::Group(ExprGroup { attrs, .. })
        | Expr::If(ExprIf { attrs, .. })
        | Expr::Index(ExprIndex { attrs, .. })
        | Expr::Let(ExprLet { attrs, .. })
        | Expr::Lit(ExprLit { attrs, .. })
        | Expr::Loop(ExprLoop { attrs, .. })
        | Expr::Macro(ExprMacro { attrs, .. })
        | Expr::Match(ExprMatch { attrs, .. })
        | Expr::MethodCall(ExprMethodCall { attrs, .. })
        | Expr::Paren(ExprParen { attrs, .. })
        | Expr::Path(ExprPath { attrs, .. })
        | Expr::Range(ExprRange { attrs, .. })
        | Expr::Reference(ExprReference { attrs, .. })
        | Expr::Repeat(ExprRepeat { attrs, .. })
        | Expr::Return(ExprReturn { attrs, .. })
        | Expr::Struct(ExprStruct { attrs, .. })
        | Expr::Try(ExprTry { attrs, .. })
        | Expr::TryBlock(ExprTryBlock { attrs, .. })
        | Expr::Type(ExprType { attrs, .. })
        | Expr::Unary(ExprUnary { attrs, .. })
        | Expr::Unsafe(ExprUnsafe { attrs, .. })
        | Expr::While(ExprWhile { attrs, .. })
        | Expr::Yield(ExprYield { attrs, .. }) => {
            let mut hitori_attrs = Vec::with_capacity(attrs.capacity());
            for index in (0..attrs.len()).rev() {
                if is_hitori_attr_path(&attrs[index].path) {
                    hitori_attrs.push(attrs.remove(index));
                }
            }
            hitori_attrs
        }
        _ => vec![],
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
