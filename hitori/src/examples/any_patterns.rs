use crate as hitori;
pub struct Float;

#[hitori::impl_expr(and_expr_mut)]
impl Expr<usize, char> for Float {
    const PATTERN: _ = (
        |ch| ch == 'f',
        // this is an any-pattern
        [
            (|ch| ch == '3', |ch| ch == '2'),
            (|ch| ch == '6', |ch| ch == '4'),
        ],
    );
}
