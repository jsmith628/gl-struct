
use super::*;

use crate::format::*;
use crate::buffer::*;
use crate::version::*;

use std::ptr::*;
use std::mem::*;
use std::ops::*;

use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

use std::marker::PhantomData;

pub use self::pixels::*;
pub use self::compressed::*;

mod pixels;
mod compressed;
mod buffer;

pub trait PixelSrc {
    type Pixels: PixelData+?Sized;
    type GL: GLVersion;
    fn pixels(&self) -> Pixels<Self::Pixels, Self::GL>;
}
pub trait PixelDst: PixelSrc {
    fn pixels_mut(&mut self) -> PixelsMut<Self::Pixels,Self::GL>;
}
pub trait FromPixels: PixelSrc {
    type Hint;
    unsafe fn from_pixels<G:FnOnce(PixelsMut<Self::Pixels,Self::GL>)>(
        gl:&Self::GL, hint:Self::Hint, size: usize, get:G
    ) -> Self;
}

//
//Base impls on slices and compressed pixels
//

impl<P:Pixel> PixelSrc for [P] {
    type Pixels = [P];
    type GL = (); //no extra extensions needed for pixel transfer to an array
    fn pixels(&self) -> Pixels<[P],()> { Pixels::from_ref(self) }
}
impl<P:Pixel> PixelDst for [P] {
    fn pixels_mut(&mut self) -> PixelsMut<[P],()> { PixelsMut::from_mut(self) }
}

//NOTE: compressed pixel transfer was added alongside support for compressed internal formats
//so we don't need to test for any extra OpenGL version or extension
impl<F:SpecificCompressed> PixelSrc for Cmpr<F> {
    type Pixels = Cmpr<F>;
    type GL = ();
    fn pixels(&self) -> Pixels<Cmpr<F>,()> { Pixels::from_ref(self) }
}
impl<F:SpecificCompressed> PixelDst for Cmpr<F> {
    fn pixels_mut(&mut self) -> PixelsMut<Cmpr<F>,()> { PixelsMut::from_mut(self) }
}

//
//Might as well add the trivial impl on Pixels
//

impl<'a,P:PixelData+?Sized,GL:GLVersion> PixelSrc for Pixels<'a,P,GL> {
    type Pixels = P;
    type GL = GL;
    fn pixels(&self) -> Pixels<P,GL> { self.into() }
}

impl<'a,P:PixelData+?Sized,GL:GLVersion> PixelSrc for PixelsMut<'a,P,GL> {
    type Pixels = P;
    type GL = GL;
    fn pixels(&self) -> Pixels<P,GL> { self.into() }
}

impl<'a,P:PixelData+?Sized,GL:GLVersion> PixelDst for PixelsMut<'a,P,GL> {
    fn pixels_mut(&mut self) -> PixelsMut<P,GL> { self.into() }
}

//
//Also, might as well have a GLRef and GLMut impl
//

impl<'a,P:PixelData+?Sized,A:BufferStorage> PixelSrc for GLRef<'a,P,A> {
    type Pixels = P;
    type GL = GL_ARB_pixel_buffer_object;
    fn pixels(&self) -> Pixels<P,GL_ARB_pixel_buffer_object> {
        match self {
            Self::Ref(ptr) => Pixels::from_ref(&**ptr).lock(),
            Self::Buf(ptr) => Pixels::from_buf(ptr.as_slice()),
        }
    }
}

impl<'a,P:PixelData+?Sized,A:BufferStorage> PixelSrc for GLMut<'a,P,A> {
    type Pixels = P;
    type GL = GL_ARB_pixel_buffer_object;
    fn pixels(&self) -> Pixels<P,GL_ARB_pixel_buffer_object> {
        match self {
            Self::Mut(ptr) => Pixels::from_ref(&**ptr).lock(),
            Self::Buf(ptr) => Pixels::from_buf(ptr.as_slice()),
        }
    }
}

impl<'a,P:PixelData+?Sized,A:BufferStorage> PixelDst for GLMut<'a,P,A> {
    fn pixels_mut(&mut self) -> PixelsMut<P,GL_ARB_pixel_buffer_object> {
        match self {
            Self::Mut(ptr) => PixelsMut::from_mut(&mut **ptr).lock(),
            Self::Buf(ptr) => PixelsMut::from_buf(ptr.as_mut_slice()),
        }
    }
}

//
//Auto-impl on Deref
//

impl<T:Deref> PixelSrc for T where T::Target: PixelSrc {
    type Pixels = <T::Target as PixelSrc>::Pixels;
    type GL = <T::Target as PixelSrc>::GL;
    fn pixels(&self) -> Pixels<Self::Pixels,Self::GL> { (&**self).pixels() }
}

impl<T:DerefMut> PixelDst for T where T::Target: PixelDst {
    fn pixels_mut(&mut self) -> PixelsMut<Self::Pixels,Self::GL> { (&mut **self).pixels_mut()  }
}
