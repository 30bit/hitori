/// Rust identifier such as `my_var32`
pub struct Identifier;

#[hitori::impl_expr]
impl Expr<usize, char> for Identifier {
    const PATTERN: _ = (
        |ch: char| ch == '_' || ch.is_alphabetic(),
        #[hitori::repeat(ge = 0)]
        (|ch: char| ch == '_' || ch.is_alphanumeric(),),
    );
}
