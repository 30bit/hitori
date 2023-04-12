use crate as hitori;
pub struct Score;

#[hitori::impl_expr]
impl Expr<usize, char> for Score {
    const PATTERN: _ = (
        // this sets `ScoreCapture::left` and `ScoreCapture::another_left`
        #[hitori::capture(left, another_left)]
        (|ch: char| ch.is_ascii_digit()),
        |ch| ch == ':',
        // this sets `ScoreCaptureMut::right`
        #[hitori::capture(right)]
        (|ch: char| ch.is_ascii_digit()),
    );
}
