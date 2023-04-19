use core::ops::Range;

/// Single [`ExprMut`] match
#[derive(Clone)]
pub struct Match<Idx, C, I> {
    /// Index [`Range`] of matched subsequence of characters
    pub range: Range<Idx>,
    /// Captured ranges
    pub capture: C,
    /// The rest of the `iter` argument (i.e. where matched is skipped)
    pub iter_remainder: I,
    /// Was the `iter` advanced before or during the match
    pub is_iter_advanced: bool,
}

/// Matching expression with a mutable state
pub trait ExprMut<Idx, Ch> {
    type Capture;

    /// *See [`starts_with`](crate::generic::starts_with)*
    fn starts_with_mut<I>(
        &mut self,
        start: Idx,
        is_first: bool,
        iter: I,
    ) -> Option<Match<Idx, Self::Capture, I::IntoIter>>
    where
        I: IntoIterator<Item = (Idx, Ch)>,
        I::IntoIter: Clone;
}

/// Matching expression with an immutable state
pub trait Expr<Idx, Ch>: ExprMut<Idx, Ch> {
    /// *See [`starts_with`](crate::generic::starts_with)*
    fn starts_with<I>(
        &self,
        start: Idx,
        is_first: bool,
        iter: I,
    ) -> Option<Match<Idx, Self::Capture, I::IntoIter>>
    where
        I: IntoIterator<Item = (Idx, Ch)>,
        I::IntoIter: Clone;
}

macro_rules! impl_mut_for_mut {
    ($ty:ty) => {
        impl<'a, Idx, Ch, E: ExprMut<Idx, Ch>> ExprMut<Idx, Ch> for $ty {
            type Capture = E::Capture;

            #[inline]
            fn starts_with_mut<I>(
                &mut self,
                start: Idx,
                is_first: bool,
                iter: I,
            ) -> Option<Match<Idx, Self::Capture, I::IntoIter>>
            where
                I: IntoIterator<Item = (Idx, Ch)>,
                I::IntoIter: Clone,
            {
                E::starts_with_mut(self, start, is_first, iter)
            }
        }
    };
}

impl_mut_for_mut!(&mut E);

#[cfg(feature = "alloc")]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
impl_mut_for_mut!(alloc::boxed::Box<E>);

macro_rules! impl_for_const {
    ($ty:ty: ExprMut) => {
        impl_for_const!($ty: ExprMut::starts_with_mut(mut, Capture));
    };
    ($ty:ty: Expr) => {
        impl_for_const!($ty: Expr::starts_with);
    };
    ($ty:ty: $trait:ident::$starts_with:ident$(($mut:ident, $capture:ident))?) => {
        impl<'a, Idx, Ch, E: Expr<Idx, Ch>> $trait<Idx, Ch> for $ty {
            $(type $capture = E::Capture;)?

            #[inline]
            fn $starts_with<I>(
                &$($mut)?self,
                start: Idx,
                is_first: bool,
                iter: I
            ) -> Option<Match<Idx, Self::Capture, I::IntoIter>>
            where
                I: IntoIterator<Item = (Idx, Ch)>,
                I::IntoIter: Clone,
            {
                E::starts_with(self, start, is_first, iter)
            }
        }
    };
}

impl_for_const!(&E: ExprMut);

impl_for_const!(&E: Expr);

#[cfg(feature = "alloc")]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
impl_for_const!(alloc::boxed::Box<E>: Expr);
