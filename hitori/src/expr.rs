/// Returned by [`ExprMut::matches_mut`], [`matches`] and [`string::matches`] functions,
/// and as items of [`string::MatchesIter`]
///
/// [`matches`]: crate::generic::matches
/// [`string::matches`]: crate::string::matches
/// [`string::MatchesIter`]: crate::string::MatchesIter
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Matched<Idx, C, I> {
    /// An exclusive end of matched subsequence of characters
    pub end: Idx,
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

/// Partially-regular expression with a mutable state
pub trait ExprMut<Idx, Ch> {
    type Capture;

    /// *See [`matches`](crate::generic::matches)*
    fn matches_mut<I>(
        &mut self,
        start: Idx,
        is_first: bool,
        iter: I,
    ) -> Option<Matched<Idx, Self::Capture, I::IntoIter>>
    where
        I: IntoIterator<Item = (Idx, Ch)>,
        I::IntoIter: Clone;
}

/// Partially-regular expression with an immutable state
pub trait Expr<Idx, Ch>: ExprMut<Idx, Ch> {
    /// *See [`matches`](crate::generic::matches)*
    fn matches<I>(
        &self,
        start: Idx,
        is_first: bool,
        iter: I,
    ) -> Option<Matched<Idx, Self::Capture, I::IntoIter>>
    where
        I: IntoIterator<Item = (Idx, Ch)>,
        I::IntoIter: Clone;
}

macro_rules! impl_mut_for_mut {
    ($ty:ty) => {
        impl<'a, Idx, Ch, E: ExprMut<Idx, Ch>> ExprMut<Idx, Ch> for $ty {
            type Capture = E::Capture;

            #[inline]
            fn matches_mut<I>(
                &mut self,
                start: Idx,
                is_first: bool,
                iter: I,
            ) -> Option<Matched<Idx, Self::Capture, I::IntoIter>>
            where
                I: IntoIterator<Item = (Idx, Ch)>,
                I::IntoIter: Clone,
            {
                E::matches_mut(self, start, is_first, iter)
            }
        }
    };
}

impl_mut_for_mut!(&mut E);

#[cfg(feature = "box")]
#[cfg_attr(doc, doc(cfg(feature = "box")))]
impl_mut_for_mut!(alloc::boxed::Box<E>);

macro_rules! impl_for_const {
    ($ty:ty: ExprMut) => {
        impl_for_const!($ty: ExprMut::matches_mut(mut, Capture));
    };
    ($ty:ty: Expr) => {
        impl_for_const!($ty: Expr::matches);
    };
    ($ty:ty: $trait:ident::$matches:ident$(($mut:ident, $capture:ident))?) => {
        impl<'a, Idx, Ch, E: Expr<Idx, Ch>> $trait<Idx, Ch> for $ty {
            $(type $capture = E::Capture;)?

            #[inline]
            fn $matches<I>(
                &$($mut)?self,
                start: Idx,
                is_first: bool,
                iter: I
            ) -> Option<Matched<Idx, Self::Capture, I::IntoIter>>
            where
                I: IntoIterator<Item = (Idx, Ch)>,
                I::IntoIter: Clone,
            {
                E::matches(self, start, is_first, iter)
            }
        }
    };
}

impl_for_const!(&E: ExprMut);

impl_for_const!(&E: Expr);

#[cfg(feature = "box")]
#[cfg_attr(doc, doc(cfg(feature = "box")))]
impl_for_const!(alloc::boxed::Box<E>: Expr);
