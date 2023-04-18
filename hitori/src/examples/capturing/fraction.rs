use crate as hitori;
/// Single-digit numerator and denominator
pub struct Fraction;

#[hitori::impl_expr]
impl Expr<usize, char> for Fraction {
    const PATTERN: _ = (
        // Capture into `FractCapture.numerator`
        #[hitori::capture(numerator)]
        (|ch: char| ch.is_ascii_digit(),),
        |ch| ch == '/',
        // Capture into `FractCapture.denominator`
        #[hitori::capture(denominator)]
        (|ch| ch > '0' && ch <= '9',),
    );
}
