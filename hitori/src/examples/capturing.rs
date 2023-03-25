use crate as hitori;
pub struct Score;

#[hitori::impl_expr(and_expr_mut)]
// this defines `ScoreCaptureMut`, `ScoreCapture` and `ScoreCaptureRanges`
#[hitori::and_define(capture_mut, capture, capture_ranges)]
impl<C: ScoreCaptureMut> Expr<C, usize, char> for Score {
    const PATTERN: _ = (
        // this calls `ScoreCaptureMut::left` on matched range
        #[hitori::capture(left)]
        (|ch: char| ch.is_ascii_digit()),
        |ch| ch == ':',
        // this calls `ScoreCaptureMut::right` on matched range
        #[hitori::capture(right)]
        (|ch: char| ch.is_ascii_digit()),
    );
}
