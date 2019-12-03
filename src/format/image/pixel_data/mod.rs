use super::*;
use std::ptr::*;
use std::mem::*;

pub trait PixelSrc {
    type Pixels: ?Sized;
    fn pixel_ptr(&self) -> PixelPtr<Self::Pixels>;
}
pub trait PixelDst: PixelSrc {
    fn pixel_ptr_mut(&mut self) -> PixelPtrMut<Self::Pixels>;
}
pub trait FromPixels: PixelSrc {
    type GL: GLVersion;
    type Hint;
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<Self::Pixels>)>(
        gl:&Self::GL, hint:Self::Hint, size: usize, get:G
    ) -> Self;
}

impl<P> PixelSrc for [P] {
    type Pixels = [P];
    fn pixel_ptr(&self) -> PixelPtr<[P]> { PixelPtr::Slice(self) }
}
impl<P> PixelDst for [P] {
    fn pixel_ptr_mut(&mut self) -> PixelPtrMut<[P]> { PixelPtrMut::Slice(self) }
}

impl<F:SpecificCompressed> PixelSrc for CompressedPixels<F> {
    type Pixels = CompressedPixels<F>;
    fn pixel_ptr(&self) -> PixelPtr<Self> { PixelPtr::Slice(self) }
}
impl<F:SpecificCompressed> PixelDst for CompressedPixels<F> {
    fn pixel_ptr_mut(&mut self) -> PixelPtrMut<Self> { PixelPtrMut::Slice(self) }
}

mod deref;
mod vec;
mod buffer;
mod compressed;
