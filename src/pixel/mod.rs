
use super::*;

use crate::format::*;
use crate::buffer::*;
use crate::version::*;

use std::ptr::*;
use std::mem::*;

use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

pub use self::compressed::*;

mod deref;
mod vec;
mod buffer;
mod compressed;

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub enum Pixels<'a,P:?Sized> {
    Slice(&'a P),
    Buffer(GL_ARB_pixel_buffer_object, Slice<'a,P,ReadOnly>)
}

impl<'a,P:?Sized> Pixels<'a,P> {
    pub fn size(&self) -> usize {
        match self {
            Self::Slice(ptr) => ::std::mem::size_of_val(ptr),
            Self::Buffer(_, ptr) => ptr.size(),
        }
    }
}

impl<'a,P> Pixels<'a,[P]> {
    pub fn is_empty(&self) -> bool { self.len()==0 }
    pub fn len(&self) -> usize {
        match self {
            Self::Slice(ptr) => ptr.len(),
            Self::Buffer(_,ptr) => ptr.len(),
        }
    }
}

pub enum PixelsMut<'a,P:?Sized> {
    Slice(&'a mut P),
    Buffer(GL_ARB_pixel_buffer_object, SliceMut<'a,P,ReadOnly>)
}

impl<'a,P:?Sized> PixelsMut<'a,P> {
    pub fn size(&self) -> usize {
        match self {
            Self::Slice(ptr) => ::std::mem::size_of_val(ptr),
            Self::Buffer(_, ptr) => ptr.size(),
        }
    }
}

impl<'a,P> PixelsMut<'a,[P]> {
    pub fn is_empty(&self) -> bool { self.len()==0 }
    pub fn len(&self) -> usize {
        match self {
            Self::Slice(ptr) => ptr.len(),
            Self::Buffer(_,ptr) => ptr.len(),
        }
    }
}

pub trait PixelSrc {
    type Pixels: ?Sized;
    type GL: GLVersion;
    fn pixels(&self, gl:Self::GL) -> Pixels<Self::Pixels>;
}
pub trait PixelDst: PixelSrc {
    fn pixels_mut(&mut self, gl:Self::GL) -> PixelsMut<Self::Pixels>;
}
pub trait FromPixels: PixelSrc {
    type Hint;
    unsafe fn from_pixels<G:FnOnce(PixelsMut<Self::Pixels>)>(
        gl:&Self::GL, hint:Self::Hint, size: usize, get:G
    ) -> Self;
}

impl<P:Pixel> PixelSrc for [P] {
    type Pixels = [P];
    type GL = (); //no extra extensions needed for pixel transfer to an array
    fn pixels(&self, _:Self::GL) -> Pixels<[P]> { Pixels::Slice(self) }
}
impl<P:Pixel> PixelDst for [P] {
    fn pixels_mut(&mut self, _:Self::GL) -> PixelsMut<[P]> { PixelsMut::Slice(self) }
}

//NOTE: compressed pixel transfer was added alongside support for compressed internal formats
//so we don't need to test for any extra OpenGL version or extension
impl<F:SpecificCompressed> PixelSrc for CompressedPixels<F> {
    type Pixels = CompressedPixels<F>;
    type GL = ();
    fn pixels(&self, _:Self::GL) -> Pixels<CompressedPixels<F>> { Pixels::Slice(self) }
}
impl<F:SpecificCompressed> PixelDst for CompressedPixels<F> {
    fn pixels_mut(&mut self, _:Self::GL) -> PixelsMut<CompressedPixels<F>> { PixelsMut::Slice(self) }
}

impl<'a,P:?Sized> PixelSrc for Pixels<'a,P> {
    type Pixels = P;
    type GL = ();
    fn pixels(&self, _:Self::GL) -> Pixels<P> { *self }
}

impl<'a,P:?Sized> PixelSrc for PixelsMut<'a,P> {
    type Pixels = P;
    type GL = ();
    fn pixels(&self, _:Self::GL) -> Pixels<P> {
        match self {
            Self::Slice(ptr) => Pixels::Slice(&**ptr),
            Self::Buffer(gl,ptr) => Pixels::Buffer(*gl,ptr.as_slice()),
        }
    }
}

impl<'a,P:?Sized> PixelDst for PixelsMut<'a,P> {
    fn pixels_mut(&mut self, _:Self::GL) -> PixelsMut<P> {
        match self {
            Self::Slice(ptr) => PixelsMut::Slice(&mut **ptr),
            Self::Buffer(gl,ptr) => PixelsMut::Buffer(*gl,ptr.as_mut_slice()),
        }
    }
}
