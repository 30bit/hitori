use crate as hitori;
pub struct True;

#[hitori::impl_expr]
impl Expr<usize, char> for True {
    const PATTERN: _ = ();
}
