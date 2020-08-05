use super::*;
use crate::pixel::*;
use crate::buffer::*;
use crate::version::*;

pub use self::align::*;
pub use self::pixel_data::*;
pub use self::pixel_store::*;
pub use self::client_image::*;
pub use self::client_sub_image::*;

mod align;
mod pixel_data;
mod pixel_store;
mod client_image;
mod client_sub_image;
mod impls;

use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;
use std::convert::TryInto;

pub unsafe trait ImageSrc {

    type Pixels: ?Sized;

    fn swap_bytes(&self) -> bool;
    fn lsb_first(&self) -> bool;
    fn row_alignment(&self) -> PixelRowAlignment;

    fn row_length(&self) -> usize;
    fn image_height(&self) -> usize;

    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn depth(&self) -> usize;

    fn skip_pixels(&self) -> usize;
    fn skip_rows(&self) -> usize;
    fn skip_images(&self) -> usize;

    fn pixels(&self) -> PixelPtr<Self::Pixels>;

}

pub unsafe trait ImageDst: ImageSrc {
    fn pixels_mut(&mut self) -> PixelPtrMut<Self::Pixels>;
}

pub unsafe trait OwnedImage: ImageSrc {
    type GL: GLVersion;
    type Hint;
    unsafe fn from_gl<G:FnOnce(PixelStore, PixelPtrMut<Self::Pixels>)>(
        gl:&Self::GL, hint:Self::Hint, dim: [usize;3], get:G
    ) -> Self;
}

pub trait UncompressedImage: ImageSrc { type Pixel; }
pub trait CompressedImage: ImageSrc { type Format: SpecificCompressed; }

pub trait TexImageSrc<F:InternalFormat> =
    UncompressedImage + ImageSrc<Pixels=[<Self as UncompressedImage>::Pixel]>
where
    <Self as UncompressedImage>::Pixel: Pixel<<F as InternalFormat>::ClientFormat>;

pub trait TexImageDst<F:InternalFormat> = TexImageSrc<F> + ImageDst;
pub trait OwnedTexImage<F:InternalFormat> = TexImageSrc<F> + OwnedImage;

pub trait CompressedImageSrc =
    CompressedImage + ImageSrc<Pixels=CompressedPixels<<Self as CompressedImage>::Format>>;

pub trait CompressedImageDst = CompressedImageSrc + ImageDst;
pub trait OwnedCompressedImage = CompressedImageSrc + OwnedImage;



pub(self) fn pixel_count(dim: [usize;3]) -> usize {
    dim[0].checked_mul(dim[1])
        .and_then(|m| m.checked_mul(dim[2]))
        .expect("Overflow when computing buffer size")
}
