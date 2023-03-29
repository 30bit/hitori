use crate::{expr::ExprMut, generic};
use core::{
    mem,
    ops::{Range, RangeTo},
    str::CharIndices,
};

#[derive(Clone)]
struct CharEnds<'a> {
    next: char,
    indices: CharIndices<'a>,
    len: usize,
}

impl<'a> From<&'a str> for CharEnds<'a> {
    fn from(s: &'a str) -> Self {
        let mut indices = s.char_indices();
        let (next, len) = match indices.next() {
            Some((_, next)) => (next, s.len()),
            None => (char::default(), 0),
        };
        Self { next, indices, len }
    }
}

impl<'a> Iterator for CharEnds<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else if let Some((end, next)) = self.indices.next() {
            Some((end, mem::replace(&mut self.next, next)))
        } else {
            Some((mem::replace(&mut self.len, 0), self.next))
        }
    }
}

#[inline]
pub fn matches<E>(expr: E, s: &str) -> Option<(RangeTo<usize>, E::Capture)>
where
    E: ExprMut<usize, char>,
{
    generic::matches(expr, 0, CharEnds::from(s))
}

#[inline]
pub fn find<E>(expr: E, s: &str) -> Option<(Range<usize>, E::Capture)>
where
    E: ExprMut<usize, char>,
{
    generic::find(expr, 0, CharEnds::from(s))
}
