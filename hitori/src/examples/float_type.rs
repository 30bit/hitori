use crate as hitori;
/// `f32` or `f64`
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
