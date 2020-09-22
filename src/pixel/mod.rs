
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
// mod deref;
// mod vec;
mod buffer;

pub trait PixelSrc {
    type Pixels: ?Sized;
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
impl<F:SpecificCompressed> PixelSrc for CompressedPixels<F> {
    type Pixels = CompressedPixels<F>;
    type GL = ();
    fn pixels(&self) -> Pixels<CompressedPixels<F>,()> { Pixels::from_ref(self) }
}
impl<F:SpecificCompressed> PixelDst for CompressedPixels<F> {
    fn pixels_mut(&mut self) -> PixelsMut<CompressedPixels<F>,()> { PixelsMut::from_mut(self) }
}

//
//Might as well add the trivial impl on Pixels
//

impl<'a,P:?Sized,GL:GLVersion> PixelSrc for Pixels<'a,P,GL> {
    type Pixels = P;
    type GL = GL;
    fn pixels(&self) -> Pixels<P,GL> { self.into() }
}

impl<'a,P:?Sized,GL:GLVersion> PixelSrc for PixelsMut<'a,P,GL> {
    type Pixels = P;
    type GL = GL;
    fn pixels(&self) -> Pixels<P,GL> { self.into() }
}

impl<'a,P:?Sized,GL:GLVersion> PixelDst for PixelsMut<'a,P,GL> {
    fn pixels_mut(&mut self) -> PixelsMut<P,GL> { self.into() }
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
