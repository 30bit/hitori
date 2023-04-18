use crate as hitori;
/// Email address
pub struct Email;

#[hitori::impl_expr]
impl Expr<usize, char> for Email {
    const PATTERN: _ = (
        #[hitori::capture(user)]
        (
            #[hitori::repeat(ge = 1)]
            (|ch: char| {
                ch == '.' || ch == '+' || ch == '-' || ch == '_' || ch.is_ascii_alphanumeric()
            },),
        ),
        |ch| ch == '@',
        #[hitori::capture(domain_with_extension)]
        (
            #[hitori::repeat(ge = 0)]
            (|ch: char| ch == '-' || ch == '_' || ch.is_ascii_alphanumeric(),),
            #[hitori::repeat(ge = 1)]
            (
                |ch| ch == '.',
                #[hitori::capture(domain_extension)]
                (
                    #[hitori::repeat(ge = 1)]
                    (|ch: char| ch == '-' || ch == '_' || ch.is_ascii_alphanumeric(),),
                ),
            ),
        ),
    );
}
