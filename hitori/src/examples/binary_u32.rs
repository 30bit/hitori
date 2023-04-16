use crate as hitori;
pub struct BinaryU32;

#[hitori::impl_expr]
impl Expr<usize, char> for BinaryU32 {
    const PATTERN: _ = #[hitori::repeat(ge = 1, le = 32)]
    (|ch| ch == '0' || ch == '1',);
}
