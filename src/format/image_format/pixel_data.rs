
use super::*;
use crate::object::*;

use std::mem::*;
use std::fmt::{Debug, Formatter};

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub struct InvalidPixelRowAlignment(pub u8);

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub struct PixelRowAlignment(u8);

display_from_debug!(PixelRowAlignment);
impl Debug for PixelRowAlignment {
    #[inline] fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result { write!(f, "{}", self.0) }
}

pub const ALIGN_1: PixelRowAlignment = PixelRowAlignment(1);
pub const ALIGN_2: PixelRowAlignment = PixelRowAlignment(2);
pub const ALIGN_4: PixelRowAlignment = PixelRowAlignment(4);
pub const ALIGN_8: PixelRowAlignment = PixelRowAlignment(8);

impl Into<u8> for PixelRowAlignment { #[inline] fn into(self) -> u8 {self.0} }

impl TryFrom<u8> for PixelRowAlignment {
    type Error = InvalidPixelRowAlignment;
    #[inline] fn try_from(a:u8) -> Result<Self,Self::Error> {
        match a {
            1 | 2 | 4 | 8 => Ok(PixelRowAlignment(a)),
            _ => Err(InvalidPixelRowAlignment(a))
        }
    }
}

pub unsafe trait Pixel<F: ClientFormat>: Copy+PartialEq {
    fn format() -> F;
}

pub enum Pixels<'a,F:ClientFormat,P:Pixel<F>> {
    Slice(F, &'a [P]),
    Buffer(F, Slice<'a,[P],ReadOnly>)
}

pub enum PixelsMut<'a,F:ClientFormat,P:Pixel<F>> {
    Slice(F, &'a mut [P]),
    Buffer(F, SliceMut<'a,[P],ReadOnly>)
}

pub trait PixelData<F:ClientFormat> {
    type Pixel: Pixel<F>;

    #[inline] fn swap_bytes(&self) -> bool {false}
    #[inline] fn lsb_first(&self) -> bool {false}

    #[inline] fn alignment(&self) -> PixelRowAlignment {ALIGN_1}
    #[inline] fn row_length(&self) -> usize {0}
    #[inline] fn image_height(&self) -> usize {0}

    #[inline] fn skip_pixels(&self) -> usize {0}
    #[inline] fn skip_rows(&self) -> usize {0}
    #[inline] fn skip_images(&self) -> usize {0}

    fn pixels<'a>(&'a self) -> Pixels<'a,F,Self::Pixel>;
}

pub trait PixelDataMut<F:ClientFormat>: PixelData<F> {
    fn pixels_mut<'a>(&'a mut self) -> PixelsMut<'a,F,Self::Pixel>;
}

pub(crate) unsafe fn apply_packing_settings<F:ClientFormat,P:PixelData<F>+?Sized>(pixels:&P) {
    gl::PixelStorei(gl::PACK_SWAP_BYTES, pixels.swap_bytes() as GLint);
    gl::PixelStorei(gl::PACK_LSB_FIRST, pixels.lsb_first() as GLint);
    gl::PixelStorei(gl::PACK_ALIGNMENT, pixels.alignment().0 as GLint);
    gl::PixelStorei(gl::PACK_ROW_LENGTH, pixels.row_length() as GLint);
    gl::PixelStorei(gl::PACK_IMAGE_HEIGHT, pixels.image_height() as GLint);
    gl::PixelStorei(gl::PACK_SKIP_PIXELS, pixels.skip_pixels() as GLint);
    gl::PixelStorei(gl::PACK_SKIP_ROWS, pixels.skip_rows() as GLint);
    gl::PixelStorei(gl::PACK_SKIP_IMAGES, pixels.skip_images() as GLint);
}

pub(crate) unsafe fn apply_unpacking_settings<F:ClientFormat,P:PixelData<F>+?Sized>(pixels:&P) {
    gl::PixelStorei(gl::UNPACK_SWAP_BYTES, pixels.swap_bytes() as GLint);
    gl::PixelStorei(gl::UNPACK_LSB_FIRST, pixels.lsb_first() as GLint);
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, pixels.alignment().0 as GLint);
    gl::PixelStorei(gl::UNPACK_ROW_LENGTH, pixels.row_length() as GLint);
    gl::PixelStorei(gl::UNPACK_IMAGE_HEIGHT, pixels.image_height() as GLint);
    gl::PixelStorei(gl::UNPACK_SKIP_PIXELS, pixels.skip_pixels() as GLint);
    gl::PixelStorei(gl::UNPACK_SKIP_ROWS, pixels.skip_rows() as GLint);
    gl::PixelStorei(gl::UNPACK_SKIP_IMAGES, pixels.skip_images() as GLint);
}

impl<F:ClientFormat, P:Pixel<F>> PixelData<F> for [P] {
    type Pixel = P;
    fn pixels<'a>(&'a self) -> Pixels<'a,F,Self::Pixel> { Pixels::Slice(P::format(), self) }
}

impl<F:ClientFormat, P:Pixel<F>> PixelDataMut<F> for [P] {
    fn pixels_mut<'a>(&'a mut self) -> PixelsMut<'a,F,Self::Pixel> { PixelsMut::Slice(P::format(), self) }
}

impl<F:ClientFormat, P:Pixel<F>, A:Initialized> PixelData<F> for Buffer<[P],A> {
    type Pixel = P;
    fn pixels<'a>(&'a self) -> Pixels<'a,F,Self::Pixel> {
        Pixels::Buffer(P::format(), self.downgrade_ref().as_slice())
    }
}

impl<F:ClientFormat, P:Pixel<F>, A:Initialized> PixelDataMut<F> for Buffer<[P],A> {
    fn pixels_mut<'a>(&'a mut self) -> PixelsMut<'a,F,Self::Pixel> {
        PixelsMut::Buffer(P::format(), self.downgrade_mut().as_slice_mut())
    }
}

impl<'a, F:ClientFormat, P:Pixel<F>, A:Initialized> PixelData<F> for Slice<'a,[P],A> {
    type Pixel = P;
    fn pixels<'b>(&'b self) -> Pixels<'b,F,Self::Pixel> {
        Pixels::Buffer(P::format(), self.downgrade())
    }
}

impl<'a, F:ClientFormat, P:Pixel<F>, A:Initialized> PixelData<F> for SliceMut<'a,[P],A> {
    type Pixel = P;
    fn pixels<'b>(&'b self) -> Pixels<'b,F,Self::Pixel> {
        Pixels::Buffer(P::format(), self.as_immut().downgrade())
    }
}

impl<'a,F:ClientFormat, P:Pixel<F>, A:Initialized> PixelDataMut<F> for SliceMut<'a,[P],A> {
    fn pixels_mut<'b>(&'b mut self) -> PixelsMut<'b,F,Self::Pixel> {
        PixelsMut::Buffer(P::format(), self.index_mut(..).downgrade())
    }
}
