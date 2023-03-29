use crate as hitori;
pub struct Scream;

#[hitori::impl_expr(and_expr_mut)]
impl Expr<usize, char> for Scream {
    const PATTERN: _ = (
        // this repeats zero or more times
        #[hitori::repeat(+)]
        (
            // this repeats at least 3 and at most 30 times
            #[hitori::repeat(3..31)]
            (|ch| ch == 'A'),
            // this repeats at most 20 times
            #[hitori::repeat(..=20)]
            (|ch| ch == 'a'),
        ),
        // this repeats zero or one time
        #[hitori::repeat(?)]
        (|ch| ch == '!'),
    );
}
