use crate::{CaptureMut, ExprMut};
use core::ops::{Range, RangeTo};

/// The indices `iter` produces must be `Ch` ends.
/// This is unlike [`CharIndices`] that iterates over `char` starts.
///
/// [`CharIndices`]: core::str::CharIndices
#[inline]
pub fn matches<E, C, Idx, Ch, I>(
    mut expr: E,
    mut capture: C,
    start: Idx,
    iter: I,
) -> Result<Option<RangeTo<Idx>>, <C as CaptureMut>::Error>
where
    E: ExprMut<C, Idx, Ch>,
    C: CaptureMut,
    Idx: Clone,
    I: IntoIterator<Item = (Idx, Ch)>,
    I::IntoIter: Clone,
{
    expr.matches(&mut capture, start, iter)
}

/// The indices `iter` produces must be `Ch` ends.
/// This is unlike [`CharIndices`] that iterates over `char` starts.
///
/// [`CharIndices`]: core::str::CharIndices
pub fn matches_no_capture<E, Idx, Ch, I>(expr: E, start: Idx, iter: I) -> Option<RangeTo<Idx>>
where
    E: ExprMut<(), Idx, Ch>,
    Idx: Clone,
    I: IntoIterator<Item = (Idx, Ch)>,
    I::IntoIter: Clone,
{
    matches(expr, (), start, iter).unwrap()
}

/// The indices `iter` produces must be `Ch` ends.
/// This is unlike [`CharIndices`] that iterates over `char` starts.
///
/// [`CharIndices`]: core::str::CharIndices
pub fn find<E, C, Idx, Ch, I>(
    mut expr: E,
    mut capture: C,
    mut start: Idx,
    iter: I,
) -> Result<Option<Range<Idx>>, <C as CaptureMut>::Error>
where
    E: ExprMut<C, Idx, Ch>,
    C: CaptureMut,
    Idx: Clone,
    I: IntoIterator<Item = (Idx, Ch)>,
    I::IntoIter: Clone,
{
    let mut iter = iter.into_iter();
    loop {
        if let Some(RangeTo { end }) = expr.matches(&mut capture, start.clone(), iter.clone())? {
            return Ok(Some(start..end));
        } else if let Some((new_start, _)) = iter.next() {
            capture.clear();
            start = new_start;
        } else {
            return Ok(None);
        }
    }
}

/// The indices `iter` produces must be `Ch` ends.
/// This is unlike [`CharIndices`] that iterates over `char` starts.
///
/// [`CharIndices`]: core::str::CharIndices
pub fn find_no_capture<E, Idx, Ch, I>(expr: E, start: Idx, iter: I) -> Option<Range<Idx>>
where
    E: ExprMut<(), Idx, Ch>,
    Idx: Clone,
    I: IntoIterator<Item = (Idx, Ch)>,
    I::IntoIter: Clone,
{
    find(expr, (), start, iter).unwrap()
}
