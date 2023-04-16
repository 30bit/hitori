use crate as hitori;
pub struct Identifier;

#[hitori::impl_expr]
impl Expr<usize, char> for Identifier {
    const PATTERN: _ = (
        |ch: char| ch == '_' || ch.is_alphabetic(),
        #[hitori::repeat(ge = 0)]
        (|ch: char| ch == '_' || ch.is_alphanumeric(),),
    );
}
