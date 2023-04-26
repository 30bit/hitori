/// Internet Protocol v4 address
pub struct IpV4;

#[hitori::impl_expr]
impl Expr<usize, char> for IpV4 {
    const PATTERN: _ = (
        #[hitori::repeat(eq = 3)]
        (
            [
                (
                    |ch| ch == '2',
                    |ch| ch == '5',
                    |ch| ('0'..='5').contains(&ch),
                ),
                (
                    |ch| ch == '2',
                    |ch| ('0'..='4').contains(&ch),
                    |ch: char| ch.is_ascii_digit(),
                ),
                (
                    |ch| ch == '0' || ch == '1',
                    |ch: char| ch.is_ascii_digit(),
                    |ch: char| ch.is_ascii_digit(),
                ),
                (
                    |ch: char| ch.is_ascii_digit(),
                    |ch: char| ch.is_ascii_digit(),
                ),
            ],
            |ch| ch == '.',
        ),
        [
            (
                |ch| ch == '2',
                |ch| ch == '5',
                |ch| ('0'..='5').contains(&ch),
            ),
            (
                |ch| ch == '2',
                |ch| ('0'..='4').contains(&ch),
                |ch: char| ch.is_ascii_digit(),
            ),
            (
                |ch| ch == '0' || ch == '1',
                |ch: char| ch.is_ascii_digit(),
                |ch: char| ch.is_ascii_digit(),
            ),
            (
                |ch: char| ch.is_ascii_digit(),
                |ch: char| ch.is_ascii_digit(),
            ),
        ],
    );
}
