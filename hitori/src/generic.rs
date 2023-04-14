use crate::expr::{ExprMut, Matched};
use core::ops::Range;

/// The indices `iter` produces must be `Ch` ends.
/// This is unlike [`CharIndices`] that iterates over `char` starts.
///
/// [`CharIndices`]: core::str::CharIndices
#[inline]
pub fn matches<E, Idx, Ch, I>(
    mut expr: E,
    start: Idx,
    is_first: bool,
    iter: I,
) -> Option<Matched<Idx, E::Capture, I::IntoIter>>
where
    E: ExprMut<Idx, Ch>,
    I: IntoIterator<Item = (Idx, Ch)>,
    I::IntoIter: Clone,
{
    expr.matches_mut(start, is_first, iter)
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Found<Idx, C, I> {
    pub range: Range<Idx>,
    pub capture: C,
    pub iter_remainder: I,
    pub advanced_iter: bool,
}

/// The indices `iter` produces must be `Ch` ends.
/// This is unlike [`CharIndices`] that iterates over `char` starts.
///
/// [`CharIndices`]: core::str::CharIndices
pub fn find<E, Idx, Ch, I>(
    mut expr: E,
    mut start: Idx,
    is_first: bool,
    iter: I,
) -> Option<Found<Idx, E::Capture, I::IntoIter>>
where
    E: ExprMut<Idx, Ch>,
    Idx: Clone,
    I: IntoIterator<Item = (Idx, Ch)>,
    I::IntoIter: Clone,
{
    let mut iter = iter.into_iter();
    loop {
        if let Some(matched) = expr.matches_mut(start.clone(), is_first, iter.clone()) {
            return Some(Found {
                range: start..matched.end,
                capture: matched.capture,
                iter_remainder: matched.iter_remainder,
                advanced_iter: matched.advanced_iter,
            });
        } else if let Some((new_start, _)) = iter.next() {
            start = new_start;
        } else {
            return None;
        }
    }
}
