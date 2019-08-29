
use super::*;

use buffer::{RawBuffer};
use std::mem::*;

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub struct InvalidPixelRowAlignment(pub u8);

display_from_debug!(InvalidPixelRowAlignment);
impl ::std::fmt::Debug for InvalidPixelRowAlignment {
    #[inline]
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub struct PixelRowAlignment(u8);

display_from_debug!(PixelRowAlignment);
impl ::std::fmt::Debug for PixelRowAlignment {
    #[inline]
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub const ALIGN_1: PixelRowAlignment = PixelRowAlignment(1);
pub const ALIGN_2: PixelRowAlignment = PixelRowAlignment(2);
pub const ALIGN_4: PixelRowAlignment = PixelRowAlignment(4);
pub const ALIGN_8: PixelRowAlignment = PixelRowAlignment(8);

impl TryFrom<u8> for PixelRowAlignment {
    type Error = InvalidPixelRowAlignment;
    #[inline] fn try_from(a:u8) -> Result<Self,Self::Error> {
        match a {
            1 | 2 | 4 | 8 => Ok(PixelRowAlignment(a)),
            _ => Err(InvalidPixelRowAlignment(a))
        }
    }
}

impl Into<u8> for PixelRowAlignment {
    #[inline] fn into(self) -> u8 {self.0}
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

pub unsafe trait PixelData<F:ClientFormat> {

    #[inline] fn swap_bytes(&self) -> bool {false}
    #[inline] fn lsb_first(&self) -> bool {false}

    #[inline] fn alignment(&self) -> PixelRowAlignment {ALIGN_4}
    #[inline] fn row_length(&self) -> usize {0}
    #[inline] fn image_height(&self) -> usize {0}

    #[inline] fn skip_pixels(&self) -> usize {0}
    #[inline] fn skip_rows(&self) -> usize {0}
    #[inline] fn skip_images(&self) -> usize {0}

    fn format_type(&self) -> F;
    fn count(&self) -> usize;
    fn size(&self) -> usize;

    fn pixels<'a>(
        &'a self, target:&'a mut BindingLocation<RawBuffer>
    ) -> (Option<Binding<'a,RawBuffer>>, *const GLvoid);
}

pub unsafe trait PixelDataMut<F:ClientFormat>: PixelData<F> {
    fn pixels_mut<'a>(
        &'a mut self, target:&'a mut BindingLocation<RawBuffer>
    ) -> (Option<Binding<'a,RawBuffer>>, *mut GLvoid);
}

pub unsafe trait PixelType<F: ClientFormat>: Sized+Copy+Clone+PartialEq {
    fn format_type() -> F;
    fn swap_bytes() -> bool;
    fn lsb_first() -> bool;
}

unsafe impl<F:ClientFormat,T:PixelType<F>> PixelData<F> for [T] {
    #[inline] fn swap_bytes(&self) -> bool {T::swap_bytes()}
    #[inline] fn lsb_first(&self) -> bool {T::lsb_first()}

    #[inline] fn alignment(&self) -> PixelRowAlignment { PixelRowAlignment(align_of::<T>().min(8) as u8) }

    #[inline] fn format_type(&self) -> F {T::format_type()}
    #[inline] fn count(&self) -> usize {self.len()}
    #[inline] fn size(&self) -> usize {size_of_val(self)}

    #[inline] fn pixels<'a>(
        &'a self, _:&'a mut BindingLocation<RawBuffer>
    ) -> (Option<Binding<'a,RawBuffer>>, *const GLvoid) {
        (None, &self[0] as *const T as *const GLvoid)
    }
}

unsafe impl<F:ClientFormat,T:PixelType<F>> PixelDataMut<F> for [T] {
    #[inline] fn pixels_mut<'a>(
        &'a mut self, _:&'a mut BindingLocation<RawBuffer>
    ) -> (Option<Binding<'a,RawBuffer>>, *mut GLvoid) {
        (None, &mut self[0] as *mut T as *mut GLvoid)
    }
}
