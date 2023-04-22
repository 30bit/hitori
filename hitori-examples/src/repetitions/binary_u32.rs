/// Literal [`u32`] binary notation (e.g. `0b110011010`)
pub struct BinaryU32;

#[hitori::impl_expr]
impl Expr<usize, char> for BinaryU32 {
    const PATTERN: _ = (
        |ch| ch == '0',
        |ch| ch == 'b',
        #[hitori::repeat(ge = 1, le = 32)]
        (|ch| ch == '0' || ch == '1',),
    );
}
