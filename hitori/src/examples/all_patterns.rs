use crate as hitori;
pub struct OhNo;

#[hitori::impl_expr(and_expr_mut)]
impl Expr<usize, char> for OhNo {
    const PATTERN: _ =
        // this is an all-pattern
        (
            // this is an all-pattern
            (|ch| ch == 'o', |ch| ch == 'h'),
            char::is_whitespace,
            // this is an all-pattern
            (|ch| ch == 'n', |ch| ch == 'o'),
        );
}
