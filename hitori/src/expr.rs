use core::ops::RangeTo;

pub trait ExprMut<Idx, Ch> {
    type Capture;

    fn matches_mut<I>(&mut self, start: Idx, iter: I) -> Option<(RangeTo<Idx>, Self::Capture)>
    where
        I: IntoIterator<Item = (Idx, Ch)>,
        I::IntoIter: Clone;
}

pub trait Expr<Idx, Ch>: ExprMut<Idx, Ch> {
    fn matches<I>(&self, start: Idx, iter: I) -> Option<(RangeTo<Idx>, Self::Capture)>
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
                iter: I,
            ) -> Option<(RangeTo<Idx>, Self::Capture)>
            where
                I: IntoIterator<Item = (Idx, Ch)>,
                I::IntoIter: Clone,
            {
                E::matches_mut(self, start, iter)
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
            fn $matches<I>(&$($mut)?self, start: Idx, iter: I) -> Option<(RangeTo<Idx>, Self::Capture)>
            where
                I: IntoIterator<Item = (Idx, Ch)>,
                I::IntoIter: Clone,
            {
                E::matches(self, start, iter)
            }
        }
    };
}

impl_for_const!(&E: ExprMut);

impl_for_const!(&E: Expr);

#[cfg(feature = "box")]
#[cfg_attr(doc, doc(cfg(feature = "box")))]
impl_for_const!(alloc::boxed::Box<E>: Expr);
