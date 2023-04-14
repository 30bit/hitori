use crate::{
    expr::{ExprMut, Matched},
    generic::{self, Found},
};
use core::{mem, str::CharIndices};

#[derive(Clone, Debug)]
pub struct CharEnds<'a> {
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
pub fn matches<E>(expr: E, s: &str) -> Option<Matched<usize, E::Capture, CharEnds>>
where
    E: ExprMut<usize, char>,
{
    generic::matches(expr, 0, true, CharEnds::from(s))
}

#[derive(Clone, Debug)]
pub struct MatchesIter<'a, E> {
    pub expr: E,
    pub start: usize,
    pub iter: CharEnds<'a>,
}

impl<'a, E> MatchesIter<'a, E> {
    pub fn new<I>(expr: E, s: &'a str) -> Self {
        Self {
            expr,
            start: 0,
            iter: s.into(),
        }
    }
}

impl<'a, E> Iterator for MatchesIter<'a, E>
where
    E: ExprMut<usize, char>,
{
    type Item = Matched<usize, E::Capture, CharEnds<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let matched = generic::matches(
            &mut self.expr,
            self.start,
            self.start == 0,
            self.iter.clone(),
        )?;
        self.start = matched.end;
        self.iter = matched.iter_remainder.clone();
        Some(matched)
    }
}

pub fn find<E>(expr: E, s: &str) -> Option<Found<usize, E::Capture, CharEnds>>
where
    E: ExprMut<usize, char>,
{
    generic::find(expr, 0, true, CharEnds::from(s))
}

#[derive(Clone, Debug)]
pub struct FindIter<'a, E> {
    pub expr: E,
    pub start: usize,
    pub iter: CharEnds<'a>,
}

impl<'a, E> FindIter<'a, E> {
    pub fn new<I>(expr: E, s: &'a str) -> Self {
        Self {
            expr,
            start: 0,
            iter: s.into(),
        }
    }
}

impl<'a, E> Iterator for FindIter<'a, E>
where
    E: ExprMut<usize, char>,
{
    type Item = Found<usize, E::Capture, CharEnds<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let found = generic::find(
            &mut self.expr,
            self.start,
            self.start == 0,
            self.iter.clone(),
        )?;
        self.start = found.range.end;
        self.iter = found.iter_remainder.clone();
        Some(found)
    }
}
