use crate as hitori;
pub struct FloatType;

#[hitori::impl_expr]
impl Expr<usize, char> for FloatType {
    const PATTERN: _ = (
        |ch| ch == 'f',
        [
            (|ch| ch == '3', |ch| ch == '2'),
            (|ch| ch == '6', |ch| ch == '4'),
        ],
    );
}
