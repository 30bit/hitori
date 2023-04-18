use crate as hitori;
/// Numeric-only password with up to 8 characters
pub struct BadPassword;

#[hitori::impl_expr]
impl Expr<usize, char> for BadPassword {
    const PATTERN: _ = #[hitori::repeat(gt = 0, le = 8)]
    (|ch: char| ch.is_ascii_digit(),); // removing a comma in this line won't compile
}
