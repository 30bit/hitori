use {crate as hitori, core::str::Chars};

const PHRASE: &str = "Would you kindly ";

pub struct WouldYouKindly {
    phrase_chars: Chars<'static>,
}

impl Default for WouldYouKindly {
    fn default() -> Self {
        Self {
            phrase_chars: PHRASE.chars(),
        }
    }
}

#[hitori::impl_expr_mut]
impl ExprMut<usize, char> for WouldYouKindly {
    const PATTERN: _ = (
        #[hitori::repeat(eq = "PHRASE.len()")]
        (|ch| ch == self.phrase_chars.next().unwrap(),),
        #[hitori::capture(request)]
        (
            #[hitori::repeat(ge = 1)]
            (|ch| ch != '?',),
        ),
        |ch| ch == '?',
    );
}
