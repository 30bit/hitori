use core::convert::Infallible;

pub trait CaptureMut {
    type Error;

    fn clear(&mut self);
}

pub trait Capture: CaptureMut {
    fn clear(&self) {}
}

macro_rules! impl_mut_for_mut {
    ($ty:ty) => {
        impl<C> CaptureMut for $ty
        where
            C: CaptureMut,
        {
            type Error = C::Error;

            #[inline]
            fn clear(&mut self) {
                C::clear(self)
            }
        }
    };
}

impl_mut_for_mut!(&mut C);

#[cfg(feature = "box")]
#[cfg_attr(doc, doc(cfg(feature = "box")))]
impl_mut_for_mut!(alloc::boxed::Box<C>);

macro_rules! impl_for_const {
    ($ty:ty: $trait:ident$(($mut:ident, $error:ident))?) => {
        impl<C> $trait for $ty
        where
            C: Capture,
        {
            $(type $error = C::Error;)?

            fn clear(&$($mut )?self) {
                <C as Capture>::clear(self)
            }
        }
    };
}

impl_for_const!(&C: CaptureMut(mut, Error));
impl_for_const!(&C: Capture);

#[cfg(feature = "box")]
#[cfg_attr(doc, doc(cfg(feature = "box")))]
impl_for_const!(alloc::boxed::Box<C>: Capture);

impl CaptureMut for () {
    type Error = Infallible;

    fn clear(&mut self) {}
}

impl Capture for () {}
