use super::*;
use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;
use object::buffer::*;

pub unsafe trait PixelSrc<F:ClientFormat> {
    fn pixels(&self) -> PixelPtr<F>;
}
pub unsafe trait PixelDst<F:ClientFormat>: PixelSrc<F> {
    fn pixels_mut(&mut self) -> PixelPtrMut<F>;
}
pub unsafe trait FromPixels<F:ClientFormat>: PixelSrc<F> {
    unsafe fn from_pixels<GL:FnOnce(PixelPtrMut<F>)>(count: usize, gl:GL) -> Self;
}

unsafe impl<F:ClientFormat,P:Pixel<F>> PixelSrc<F> for [P] {
    fn pixels(&self) -> PixelPtr<F> { PixelPtr::Slice(P::format(), self.as_ptr() as *const GLvoid) }
}
unsafe impl<F:ClientFormat,P:Pixel<F>> PixelDst<F> for [P] {
    fn pixels_mut(&mut self) -> PixelPtrMut<F> {
        PixelPtrMut::Slice(P::format(), self.as_mut_ptr() as *mut GLvoid)
    }
}

macro_rules! impl_pixel_src_deref {
    (for<$($a:lifetime,)* $P:ident> $ty:ty) => {
        unsafe impl<$($a,)* F:ClientFormat, $P:Pixel<F>> PixelSrc<F> for $ty {
            fn pixels(&self) -> PixelPtr<F> { (&**self).pixels() }
        }
    }
}

macro_rules! impl_pixel_dst_deref {
    (for<$($a:lifetime,)* $P:ident> $ty:ty) => {
        impl_pixel_src_deref!(for<$($a,)* $P> $ty);
        unsafe impl<$($a,)* F:ClientFormat, $P:Pixel<F>> PixelDst<F> for $ty {
            fn pixels_mut(&mut self) -> PixelPtrMut<F> { (&mut **self).pixels_mut() }
        }
    }
}

impl_pixel_dst_deref!(for<P> Box<[P]>);
impl_pixel_src_deref!(for<P> Rc<[P]>);
impl_pixel_src_deref!(for<P> Arc<[P]>);
impl_pixel_dst_deref!(for<P> Vec<P>);
impl_pixel_src_deref!(for<'a,P> Cow<'a, [P]>);


unsafe impl<F:ClientFormat,P:Pixel<F>> FromPixels<F> for Box<[P]> {
    unsafe fn from_pixels<GL:FnOnce(PixelPtrMut<F>)>(count: usize, gl:GL) -> Self {
        let mut dest = Box::new_uninit_slice(count);
        gl(PixelPtrMut::Slice(P::format(), dest.as_mut_ptr() as *mut GLvoid));
        dest.assume_init()
    }
}

unsafe impl<F:ClientFormat,P:Pixel<F>> FromPixels<F> for Vec<P> {
    unsafe fn from_pixels<GL:FnOnce(PixelPtrMut<F>)>(count: usize, gl:GL) -> Self {
        Self::from(Box::from_pixels(count, gl))
    }
}

unsafe impl<'a,F:ClientFormat,P:Pixel<F>> FromPixels<F> for Cow<'a,[P]> {
    unsafe fn from_pixels<GL:FnOnce(PixelPtrMut<F>)>(count: usize, gl:GL) -> Self {
        Self::from(Vec::from_pixels(count, gl))
    }
}

unsafe impl<F:ClientFormat,P:Pixel<F>> FromPixels<F> for Rc<[P]> {
    unsafe fn from_pixels<GL:FnOnce(PixelPtrMut<F>)>(count: usize, gl:GL) -> Self {
        let mut dest = Rc::new_uninit_slice(count);
        gl(PixelPtrMut::Slice(P::format(), Rc::get_mut_unchecked(&mut dest) as *mut _ as *mut GLvoid));
        dest.assume_init()
    }
}

unsafe impl<F:ClientFormat,P:Pixel<F>> FromPixels<F> for Arc<[P]> {
    unsafe fn from_pixels<GL:FnOnce(PixelPtrMut<F>)>(count: usize, gl:GL) -> Self {
        let mut dest = Arc::new_uninit_slice(count);
        gl(PixelPtrMut::Slice(P::format(), Arc::get_mut_unchecked(&mut dest) as *mut _ as *mut GLvoid));
        dest.assume_init()
    }
}

macro_rules! impl_pixel_src_buf {
    (for<$($a:lifetime,)* $P:ident, $A:ident> $ty:ty) => {
        unsafe impl<$($a,)* F:ClientFormat, $P:Pixel<F>, $A:Initialized> PixelSrc<F> for $ty {
            fn pixels(&self) -> PixelPtr<F> {
                PixelPtr::Buffer($P::format(), self.id(), Slice::from(self).offset() as *const GLvoid)
            }
        }
    }
}

macro_rules! impl_pixel_dst_buf {
    (for<$($a:lifetime,)* $P:ident, $A:ident> $ty:ty) => {
        impl_pixel_src_buf!(for<$($a,)* $P, $A> $ty);
        unsafe impl<$($a,)* F:ClientFormat, $P:Pixel<F>, $A:Initialized> PixelDst<F> for $ty {
            fn pixels_mut(&mut self) -> PixelPtrMut<F> {
                PixelPtrMut::Buffer($P::format(), self.id(), SliceMut::from(self).offset() as *mut GLvoid)
            }
        }
    }
}

impl_pixel_dst_buf!(for<P,A> Buffer<[P],A>);
impl_pixel_src_buf!(for<'a,P,A> Slice<'a,[P],A>);
impl_pixel_dst_buf!(for<'a,P,A> SliceMut<'a,[P],A>);
