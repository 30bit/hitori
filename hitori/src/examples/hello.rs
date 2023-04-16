use crate as hitori;
pub struct Hello;

#[hitori::impl_expr]
impl Expr<usize, char> for Hello {
    const PATTERN: _ = (
        |ch| ch == 'h',
        |ch| ch == 'e',
        |ch| ch == 'l',
        |ch| ch == 'l',
        |ch| ch == 'o',
    );
}
