use crate::expr::{ExprMut, Match};

/// Checks if an [`Iterator`] starts with [`ExprMut`]-matched characters.
///
/// # Arguments
///
/// - **`start`** – this should be the start of the first character in the `iter`.
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
pub fn starts_with<E, Idx, Ch, I>(
    mut expr: E,
    start: Idx,
    is_first: bool,
    iter: I,
) -> Option<Match<Idx, E::Capture, I::IntoIter>>
where
    E: ExprMut<Idx, Ch>,
    I: IntoIterator<Item = (Idx, Ch)>,
    I::IntoIter: Clone,
{
    expr.starts_with_mut(start, is_first, iter)
}

/// Finds the first subsequence of characters that is matched by [`ExprMut`].
///
/// *See [`starts_with`] for arguments description*
pub fn find<E, Idx, Ch, I>(
    mut expr: E,
    mut start: Idx,
    is_first: bool,
    iter: I,
) -> Option<Match<Idx, E::Capture, I::IntoIter>>
where
    E: ExprMut<Idx, Ch>,
    Idx: Clone,
    I: IntoIterator<Item = (Idx, Ch)>,
    I::IntoIter: Clone,
{
    let mut iter = iter.into_iter();
    loop {
        if let Some(matched) = expr.starts_with_mut(start.clone(), is_first, iter.clone()) {
            return Some(matched);
        } else if let Some((new_start, _)) = iter.next() {
            start = new_start;
        } else {
            return None;
        }
    }
}
