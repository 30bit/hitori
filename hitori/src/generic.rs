use crate::expr::ExprMut;
use core::ops::{Range, RangeTo};

/// The indices `iter` produces must be `Ch` ends.
/// This is unlike [`CharIndices`] that iterates over `char` starts.
///
/// [`CharIndices`]: core::str::CharIndices
#[inline]
pub fn matches<E, Idx, Ch, I>(
    mut expr: E,
    start: Idx,
    iter: I,
) -> Option<(RangeTo<Idx>, E::Capture)>
where
    E: ExprMut<Idx, Ch>,
    I: IntoIterator<Item = (Idx, Ch)>,
    I::IntoIter: Clone,
{
    expr.matches_mut(start, iter)
}

/// The indices `iter` produces must be `Ch` ends.
/// This is unlike [`CharIndices`] that iterates over `char` starts.
///
/// [`CharIndices`]: core::str::CharIndices
pub fn find<E, Idx, Ch, I>(mut expr: E, mut start: Idx, iter: I) -> Option<(Range<Idx>, E::Capture)>
where
    E: ExprMut<Idx, Ch>,
    Idx: Clone,
    I: IntoIterator<Item = (Idx, Ch)>,
    I::IntoIter: Clone,
{
    let mut iter = iter.into_iter();
    loop {
        if let Some((RangeTo { end }, capture)) = expr.matches_mut(start.clone(), iter.clone()) {
            return Some((start..end, capture));
        } else if let Some((new_start, _)) = iter.next() {
            start = new_start;
        } else {
            return None;
        }
    }
}