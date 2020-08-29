
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

#[derive(Clone, Copy)]
pub enum PixelPtr<P:?Sized> {
    Slice(*const P),
    Buffer(GLuint, *const P)
}

impl<P:?Sized> PixelPtr<P> {
    pub fn size(self) -> usize {
        match self {
            Self::Slice(ptr) => unsafe { ::std::mem::size_of_val(&*ptr) },
            Self::Buffer(_,ptr) => unsafe { ::std::mem::size_of_val(&*ptr) },
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl<P> PixelPtr<[P]> {
    pub fn len(self) -> usize {
        match self {
            Self::Slice(ptr) => unsafe { (&*ptr).len() },
            Self::Buffer(_,ptr) => unsafe { (&*ptr).len() },
        }
    }
}

#[derive(Clone, Copy)]
pub enum PixelPtrMut<P:?Sized> {
    Slice(*mut P),
    Buffer(GLuint, *mut P)
}

impl<P:?Sized> PixelPtrMut<P> {
    pub fn size(self) -> usize {
        match self {
            Self::Slice(ptr) => unsafe { ::std::mem::size_of_val(&*ptr) },
            Self::Buffer(_,ptr) => unsafe { ::std::mem::size_of_val(&*ptr) },
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl<P> PixelPtrMut<[P]> {
    pub fn len(self) -> usize {
        match self {
            Self::Slice(ptr) => unsafe { (&*ptr).len() },
            Self::Buffer(_,ptr) => unsafe { (&*ptr).len() },
        }
    }
}

pub unsafe trait PixelSrc {
    type Pixels: ?Sized;
    type GL: GLVersion;
    fn pixel_ptr(&self) -> PixelPtr<Self::Pixels>;
}
pub unsafe trait PixelDst: PixelSrc {
    fn pixel_ptr_mut(&mut self) -> PixelPtrMut<Self::Pixels>;
}
pub trait FromPixels: PixelSrc {
    type Hint;
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<Self::Pixels>)>(
        gl:&Self::GL, hint:Self::Hint, size: usize, get:G
    ) -> Self;
}

unsafe impl<P:Pixel> PixelSrc for [P] {
    type Pixels = [P];
    type GL = P::GL;
    fn pixel_ptr(&self) -> PixelPtr<[P]> { PixelPtr::Slice(self) }
}
unsafe impl<P:Pixel> PixelDst for [P] {
    fn pixel_ptr_mut(&mut self) -> PixelPtrMut<[P]> { PixelPtrMut::Slice(self) }
}

unsafe impl<F:SpecificCompressed> PixelSrc for CompressedPixels<F> {
    type Pixels = CompressedPixels<F>;
    type GL = F::GL;
    fn pixel_ptr(&self) -> PixelPtr<CompressedPixels<F>> { PixelPtr::Slice(self) }
}
unsafe impl<F:SpecificCompressed> PixelDst for CompressedPixels<F> {
    fn pixel_ptr_mut(&mut self) -> PixelPtrMut<CompressedPixels<F>> { PixelPtrMut::Slice(self) }
}
