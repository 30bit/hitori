use crate as hitori;
pub struct Rectangle;

#[hitori::impl_expr]
impl Expr<usize, char> for Rectangle {
    const PATTERN: _ = [
        (
            |ch| ch == '◾',
            char::is_whitespace,
            // Capture into both `RectangleCapture.width` and `RectangleCapture.height`
            #[hitori::capture(width, height)]
            (|ch: char| ch.is_ascii_digit(),),
        ),
        (
            |ch| ch == '▬',
            char::is_whitespace,
            #[hitori::capture(width)]
            (|ch: char| ch.is_ascii_digit(),),
            char::is_whitespace,
            #[hitori::capture(height)]
            (|ch: char| ch.is_ascii_digit(),),
        ),
    ];
}
