use super::*;
use std::borrow::*;
use std::rc::*;

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

unsafe impl<F:ClientFormat,P:Pixel<F>> PixelSrc<F> for Box<[P]> {
    fn pixels(&self) -> PixelPtr<F> { (&**self).pixels() }
}
unsafe impl<F:ClientFormat,P:Pixel<F>> PixelDst<F> for Box<[P]> {
    fn pixels_mut(&mut self) -> PixelPtrMut<F> { (&mut **self).pixels_mut() }
}
unsafe impl<F:ClientFormat,P:Pixel<F>> FromPixels<F> for Box<[P]> {
    unsafe fn from_pixels<GL:FnOnce(PixelPtrMut<F>)>(count: usize, gl:GL) -> Self {
        let mut dest = Box::new_uninit_slice(count);
        gl(PixelPtrMut::Slice(P::format(), dest.as_mut_ptr() as *mut GLvoid));
        dest.assume_init()
    }
}

unsafe impl<F:ClientFormat,P:Pixel<F>> PixelSrc<F> for Vec<P> {
    fn pixels(&self) -> PixelPtr<F> { (&**self).pixels() }
}
unsafe impl<F:ClientFormat,P:Pixel<F>> PixelDst<F> for Vec<P> {
    fn pixels_mut(&mut self) -> PixelPtrMut<F> { (&mut **self).pixels_mut() }
}
unsafe impl<F:ClientFormat,P:Pixel<F>> FromPixels<F> for Vec<P> {
    unsafe fn from_pixels<GL:FnOnce(PixelPtrMut<F>)>(count: usize, gl:GL) -> Self {
        Self::from(Box::from_pixels(count, gl))
    }
}

unsafe impl<'a,F:ClientFormat,P:Pixel<F>> PixelSrc<F> for Cow<'a,[P]> {
    fn pixels(&self) -> PixelPtr<F> { (&**self).pixels() }
}
unsafe impl<'a,F:ClientFormat,P:Pixel<F>> FromPixels<F> for Cow<'a,[P]> {
    unsafe fn from_pixels<GL:FnOnce(PixelPtrMut<F>)>(count: usize, gl:GL) -> Self {
        Self::from(Vec::from_pixels(count, gl))
    }
}

unsafe impl<F:ClientFormat,P:Pixel<F>> PixelSrc<F> for Rc<[P]> {
    fn pixels(&self) -> PixelPtr<F> { (&**self).pixels() }
}
unsafe impl<F:ClientFormat,P:Pixel<F>> FromPixels<F> for Rc<[P]> {
    unsafe fn from_pixels<GL:FnOnce(PixelPtrMut<F>)>(count: usize, gl:GL) -> Self {
        let mut dest = Rc::new_uninit_slice(count);
        gl(PixelPtrMut::Slice(P::format(), Rc::get_mut_unchecked(&mut dest) as *mut _ as *mut GLvoid));
        dest.assume_init()
    }
}
