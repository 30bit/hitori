use crate as hitori;
pub struct Aaaaa;

#[hitori::impl_expr]
impl Expr<usize, char> for Aaaaa {
    const PATTERN: _ = #[hitori::repeat(eq = 5)]
    (|ch| ch == 'a',); // removing a comma in this line won't compile
}
