/// Uniform Resource Identifier
pub struct Uri;

#[hitori::impl_expr]
impl Expr<usize, char> for Uri {
    const PATTERN: _ = (
        #[hitori::capture(schema)]
        (
            #[hitori::repeat(ge = 1)]
            (|ch: char| ch == '_' || ch.is_ascii_alphanumeric(),),
        ),
        |ch| ch == ':',
        |ch| ch == '/',
        |ch| ch == '/',
        #[hitori::capture(path)]
        (
            |ch: char| ch != '/' && ch != '?' && ch != '#' && !ch.is_ascii_whitespace(),
            #[hitori::repeat(ge = 1)]
            (|ch: char| ch != '?' && ch != '#' && !ch.is_ascii_whitespace(),),
        ),
        #[hitori::repeat(le = 1)]
        (
            |ch| ch == '?',
            #[hitori::capture(query)]
            (
                #[hitori::repeat(ge = 0)]
                (|ch: char| ch != '#' && !ch.is_ascii_whitespace(),),
            ),
        ),
        #[hitori::repeat(le = 1)]
        (
            |ch| ch == '#',
            #[hitori::capture(fragment)]
            (
                #[hitori::repeat(ge = 0)]
                (|ch: char| !ch.is_ascii_whitespace(),),
            ),
        ),
    );
}
