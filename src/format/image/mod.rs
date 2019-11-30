use super::*;
use super::pixel::*;

pub use self::align::*;
pub use self::pixel_data::*;

mod align;
mod pixel_data;

use std::num::*;
use std::convert::TryInto;

pub trait ImageSrc<F:ClientFormat>: PixelSrc<F> {
    fn swap_bytes(&self) -> bool;
    fn lsb_first(&self) -> bool;
    fn row_alignment(&self) -> PixelRowAlignment;

    fn row_length(&self) -> usize;
    fn image_height(&self) -> usize;

    fn width(&self) -> NonZeroUsize;
    fn height(&self) -> NonZeroUsize;
    fn depth(&self) -> NonZeroUsize;

    fn dim(&self) -> [NonZeroUsize; 3] { [self.width(), self.height(), self.depth()] }

}

pub trait ImageDst<F:ClientFormat> = ImageSrc<F> + PixelDst<F>;

pub trait OwnedImage<F:ClientFormat>: ImageSrc<F> {
    unsafe fn from_gl<GL:FnOnce(PixelPtrMut<F>)>(dim: [NonZeroUsize;3], gl:GL) -> Self;
}

pub(crate) unsafe fn apply_unpacking_settings<F:ClientFormat,I:ImageSrc<F>>(img: &I) {
    gl::PixelStorei(gl::UNPACK_SWAP_BYTES,   img.swap_bytes().into());
    gl::PixelStorei(gl::UNPACK_LSB_FIRST,    img.lsb_first().into());
    gl::PixelStorei(gl::UNPACK_ALIGNMENT,    img.row_alignment().0.into());
    gl::PixelStorei(gl::UNPACK_ROW_LENGTH,   img.row_length().try_into().unwrap());
    gl::PixelStorei(gl::UNPACK_IMAGE_HEIGHT, img.image_height().try_into().unwrap());
}

pub(crate) unsafe fn apply_packing_settings<F:ClientFormat,I:ImageSrc<F>>(img: &I) {
    gl::PixelStorei(gl::PACK_SWAP_BYTES,   img.swap_bytes().into());
    gl::PixelStorei(gl::PACK_LSB_FIRST,    img.lsb_first().into());
    gl::PixelStorei(gl::PACK_ALIGNMENT,    img.row_alignment().0.into());
    gl::PixelStorei(gl::PACK_ROW_LENGTH,   img.row_length().try_into().unwrap());
    gl::PixelStorei(gl::PACK_IMAGE_HEIGHT, img.image_height().try_into().unwrap());
}
