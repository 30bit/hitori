use crate::capture::CaptureMut;
use core::ops::RangeTo;

pub trait ExprMut<C, Idx, Ch>
where
    C: CaptureMut,
    Idx: Clone,
{
    fn matches<I>(
        &mut self,
        capture: &mut C,
        start: Idx,
        iter: I,
    ) -> Result<Option<RangeTo<Idx>>, <C as CaptureMut>::Error>
    where
        I: IntoIterator<Item = (Idx, Ch)>,
        I::IntoIter: Clone;
}

pub trait Expr<C, Idx, Ch>: ExprMut<C, Idx, Ch>
where
    C: CaptureMut,
    Idx: Clone,
{
    fn matches<I>(
        &self,
        capture: &mut C,
        start: Idx,
        iter: I,
    ) -> Result<Option<RangeTo<Idx>>, <C as CaptureMut>::Error>
    where
        I: IntoIterator<Item = (Idx, Ch)>,
        I::IntoIter: Clone;
}

macro_rules! impl_mut_for_mut {
    ($ty:ty) => {
        impl<'a, C, Idx, Ch, E> ExprMut<C, Idx, Ch> for $ty
        where
            C: CaptureMut,
            Idx: Clone,
            E: ExprMut<C, Idx, Ch>,
        {
            #[inline]
            fn matches<I>(
                &mut self,
                capture: &mut C,
                start: Idx,
                iter: I,
            ) -> Result<Option<RangeTo<Idx>>, <C as CaptureMut>::Error>
            where
                I: IntoIterator<Item = (Idx, Ch)>,
                I::IntoIter: Clone,
            {
                E::matches(self, capture, start, iter)
            }
        }
    };
}

impl_mut_for_mut!(&mut E);

#[cfg(feature = "box")]
#[cfg_attr(doc, doc(cfg(feature = "box")))]
impl_mut_for_mut!(alloc::boxed::Box<E>);

macro_rules! impl_for_const {
    ($ty:ty: $trait:ident$(($mut:ident))?) => {
        impl<'a, C, Idx, Ch, E> $trait<C, Idx, Ch> for $ty
        where
            C: CaptureMut,
            Idx: Clone,
            E: Expr<C, Idx, Ch>,
        {
            #[inline]
            fn matches<I>(
                &$($mut )?self,
                capture: &mut C,
                start: Idx,
                iter: I,
            ) -> Result<Option<RangeTo<Idx>>, <C as CaptureMut>::Error>
            where
                Idx: Clone,
                I: IntoIterator<Item = (Idx, Ch)>,
                I::IntoIter: Clone,
            {
                <E as Expr<C, Idx, Ch>>::matches(self, capture, start, iter)
            }
        }
    };
}

impl_for_const!(&E: ExprMut(mut));

impl_for_const!(&E: Expr);

#[cfg(feature = "box")]
#[cfg_attr(doc, doc(cfg(feature = "box")))]
impl_for_const!(alloc::boxed::Box<E>: Expr);
