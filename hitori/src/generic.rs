use crate::expr::{ExprMut, Matched};
use core::ops::Range;

/// Checks if an [`Iterator`] starts with [`ExprMut`]-matched characters.
///
/// # Arguments
///
/// - **`start`** – this should be the start of the firs character in the `iter`.
/// - **`is_first`** – tells `expr` whether it is a start of an input.
/// This affects `#[hitori::position(first)]` attribute.
/// - **`iter`** – an iterator over the characters and indices of their **ends**.
/// This is unlike what [`CharIndices`] produces, as the indices there are the
/// starts of the characters. [`string`] module provides [`CharEnds`] iterator
/// that could be used for strings instead.
///
/// [`CharIndices`]: core::str::CharIndices
/// [`string`]: crate::string
/// [`CharEnds`]: crate::string::CharEnds
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

/// Returned by [`find`] and [`string::find`] functions
/// 
/// [`string::find`]: crate::string::find
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Found<Idx, C, I> {
    /// Index [`Range`] of matched subsequence of characters
    pub range: Range<Idx>,
    /// Captured ranges
    pub capture: C,
    /// The rest of the `iter` argument (i.e. matched subsequence of
    /// characters is skipped)
    pub iter_remainder: I,
    /// Was the `iter` advanced before the match (e.g.
    /// `is_first` [`matches`] argument was `false`) or during the match
    /// (e.g. there was an [`Iterator::next`] call)
    pub is_iter_advanced: bool,
}

/// Finds the first subsequence of characters that is matched by [`ExprMut`].
///
/// *See [`matches`] for arguments description*
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
                is_iter_advanced: matched.is_iter_advanced,
            });
        } else if let Some((new_start, _)) = iter.next() {
            start = new_start;
        } else {
            return None;
        }
    }
}
