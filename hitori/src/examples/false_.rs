use crate as hitori;
pub struct False;

#[hitori::impl_expr]
impl Expr<usize, char> for False {
    const PATTERN: _ = [];
}
