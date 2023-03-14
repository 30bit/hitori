use crate::{generic, CaptureMut, ExprMut};
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

pub fn matches<E, C>(
    expr: E,
    capture: C,
    s: &str,
) -> Result<Option<RangeTo<usize>>, <C as CaptureMut>::Error>
where
    E: ExprMut<C, usize, char>,
    C: CaptureMut,
{
    generic::matches(expr, capture, 0, CharEnds::from(s))
}

pub fn matches_no_capture<E>(s: &str, expr: E) -> Option<RangeTo<usize>>
where
    E: ExprMut<(), usize, char>,
{
    generic::matches_no_capture(expr, 0, CharEnds::from(s))
}

pub fn find<E, C>(
    expr: E,
    capture: C,
    s: &str,
) -> Result<Option<Range<usize>>, <C as CaptureMut>::Error>
where
    E: ExprMut<C, usize, char>,
    C: CaptureMut,
{
    generic::find(expr, capture, 0, CharEnds::from(s))
}

pub fn find_no_capture<E>(expr: E, s: &str) -> Option<Range<usize>>
where
    E: ExprMut<(), usize, char>,
{
    generic::find_no_capture(expr, 0, CharEnds::from(s))
}
