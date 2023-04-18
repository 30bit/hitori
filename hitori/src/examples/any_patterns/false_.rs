use crate as hitori;
/// An empty any-pattern
pub struct False;

#[hitori::impl_expr]
impl Expr<usize, char> for False {
    const PATTERN: _ = [];
}
