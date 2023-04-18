use crate as hitori;
/// Checks that all characters are contained in `self.0`
pub struct AllIn<'a, Ch>(pub &'a [Ch]);

#[hitori::impl_expr]
impl<'a, Idx: Clone, Ch: PartialEq> Expr<Idx, Ch> for AllIn<'a, Ch> {
    const PATTERN: _ = #[hitori::repeat(ge = 0)]
    (|ch| self.0.contains(&ch),);
}
