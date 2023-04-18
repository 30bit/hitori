//! Items specific to [`ExprMut<usize, char>`]

use crate::{
    expr::{ExprMut, Match},
    generic,
};
#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, string::String};
use core::{iter::FusedIterator, mem, str::CharIndices};

/// Like [`CharIndices`], but tuples contain exclusive [`char`] ends
/// instead of [`char`] starts
#[derive(Clone)]
pub struct CharEnds<'a> {
    next: char,
    indices: CharIndices<'a>,
    len: usize,
}

impl<'a> CharEnds<'a> {
    pub fn new(s: &'a str) -> Self {
        let mut indices = s.char_indices();
        let (next, len) = match indices.next() {
            Some((_, next)) => (next, s.len()),
            None => (char::default(), 0),
        };
        Self { next, indices, len }
    }
}

impl<'a> From<&'a str> for CharEnds<'a> {
    #[inline]
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl<'a> Iterator for CharEnds<'a> {
    type Item = (usize, char);

    #[inline(always)]
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

impl<'a> FusedIterator for CharEnds<'a> {}

/// Shorthand for [`CharEnds::new`]
#[inline]
pub fn char_ends(s: &str) -> CharEnds {
    CharEnds::new(s)
}

/// Checks if a [`str`] starts with [`ExprMut`]-matched characters
#[inline]
pub fn starts_with<E>(expr: E, s: &str) -> Option<Match<usize, E::Capture, CharEnds>>
where
    E: ExprMut<usize, char>,
{
    generic::starts_with(expr, 0, true, CharEnds::from(s))
}

/// An iterator of successive non-overlapping [`Match`]es
/// that start where previous [`Match`] ends
#[derive(Clone)]
pub struct Repeat<'a, E> {
    expr: E,
    start: usize,
    iter: CharEnds<'a>,
}

impl<'a, E> Repeat<'a, E> {
    pub fn new(expr: E, s: &'a str) -> Self {
        Self {
            expr,
            start: 0,
            iter: s.into(),
        }
    }
}

impl<'a, E> Iterator for Repeat<'a, E>
where
    E: ExprMut<usize, char>,
{
    type Item = Match<usize, E::Capture, CharEnds<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let matched = generic::starts_with(
            &mut self.expr,
            self.start,
            self.start == 0,
            self.iter.clone(),
        )?;
        self.start = matched.range.end;
        self.iter = matched.iter_remainder.clone();
        Some(matched)
    }
}

/// Shorthand for [`Repeat::new`]
#[inline]
pub fn repeat<E>(expr: E, s: &str) -> Repeat<E> {
    Repeat::new(expr, s)
}

/// Finds the first substring that is matched by an [`ExprMut`]
#[inline]
pub fn find<E>(expr: E, s: &str) -> Option<Match<usize, E::Capture, CharEnds>>
where
    E: ExprMut<usize, char>,
{
    generic::find(expr, 0, true, CharEnds::from(s))
}

/// Iterator of successive non-overlapping [`find`]s
#[derive(Clone)]
pub struct FindIter<'a, E> {
    expr: E,
    start: usize,
    iter: CharEnds<'a>,
}

impl<'a, E> FindIter<'a, E> {
    pub fn new(expr: E, s: &'a str) -> Self {
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
    type Item = Match<usize, E::Capture, CharEnds<'a>>;

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

/// Shorthand for [`FindIter::new`]
#[inline]
pub fn find_iter<E>(expr: E, s: &str) -> FindIter<E> {
    FindIter::new(expr, s)
}

#[cfg(feature = "alloc")]
fn find_iter_replace<'a, I, C, F>(find_iter: I, s: &'a str, mut rep: F) -> Cow<'a, str>
where
    I: IntoIterator<Item = Match<usize, C, CharEnds<'a>>>,
    F: FnMut(&mut String, I::Item),
{
    let mut replaced = String::new();
    let mut start = 0;
    for found in find_iter {
        replaced.push_str(&s[start..mem::replace(&mut start, found.range.start)]);
        rep(&mut replaced, found);
    }
    if replaced.is_empty() {
        s.into()
    } else {
        replaced.push_str(&s[start..]);
        replaced.into()
    }
}

///  Replaces every matched substring using `rep` closure
///
/// First argument of `rep` is the current [`String`] accumulator.
/// Second is the current [`Match`].
///
/// Writing to the accumulator could be done using [`write!`].
#[cfg(feature = "alloc")]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
#[inline]
pub fn replace<'a, E, F>(expr: E, s: &'a str, rep: F) -> Cow<'a, str>
where
    E: ExprMut<usize, char>,
    F: FnMut(&mut String, Match<usize, E::Capture, CharEnds<'a>>),
{
    find_iter_replace(FindIter::new(expr, s), s, rep)
}

/// Replaces first `limit` matched substrings using `rep` closure.
///
/// *See [`replace`] for `rep` argument description*
#[cfg(feature = "alloc")]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
pub fn replacen<'a, E, F>(expr: E, limit: usize, s: &'a str, rep: F) -> Cow<'a, str>
where
    E: ExprMut<usize, char>,
    F: FnMut(&mut String, Match<usize, E::Capture, CharEnds<'a>>),
{
    find_iter_replace(FindIter::new(expr, s).take(limit), s, rep)
}
