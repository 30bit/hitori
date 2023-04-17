use crate as hitori;
/// An empty all-pattern
pub struct True;

#[hitori::impl_expr]
impl Expr<usize, char> for True {
    const PATTERN: _ = ();
}
