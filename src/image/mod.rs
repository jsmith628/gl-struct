use super::*;
use crate::format::*;
use crate::pixels::*;
use crate::buffer::*;
use crate::version::*;

pub use self::align::*;
pub use self::pixel_store::*;
pub use self::client_image::*;
pub use self::client_sub_image::*;

mod align;
mod pixel_store;
mod client_image;
mod client_sub_image;
// mod impls;

use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;
use std::convert::TryInto;

pub type ImageRef<'a,P,GL> = ClientSubImage<ClientImage<Pixels<'a,P,GL>>>;
pub type ImageMut<'a,P,GL> = ClientSubImage<ClientImage<PixelsMut<'a,P,GL>>>;

pub trait ImageSrc {
    type Pixels: ?Sized;
    type GL: GLVersion;
    fn image(&self) -> ImageRef<Self::Pixels,Self::GL>;
}

pub trait ImageDst: ImageSrc {
    fn image_mut(&mut self) -> ImageMut<Self::Pixels,Self::GL>;
}

pub unsafe trait OwnedImage: ImageSrc {
    type Hint;
    unsafe fn from_gl<G:FnOnce(PixelStore, PixelsMut<Self::Pixels,Self::GL>)>(
        gl:&Self::GL, hint:Self::Hint, dim: [usize;3], get:G
    ) -> Self;
}

pub trait UncompressedImage: ImageSrc { type Pixel; }
pub trait CompressedImage: ImageSrc { type Format: SpecificCompressed; }

pub trait TexImageSrc<F:InternalFormat> =
    UncompressedImage + ImageSrc<Pixels=[<Self as UncompressedImage>::Pixel]>
where
    <Self as UncompressedImage>::Pixel: UncompressedPixelData<<F as InternalFormat>::PixelLayout>;

pub trait TexImageDst<F:InternalFormat> = TexImageSrc<F> + ImageDst;
pub trait OwnedTexImage<F:InternalFormat> = TexImageSrc<F> + OwnedImage;

pub trait CompressedImageSrc =
    CompressedImage + ImageSrc<Pixels=Cmpr<<Self as CompressedImage>::Format>>;

pub trait CompressedImageDst = CompressedImageSrc + ImageDst;
pub trait OwnedCompressedImage = CompressedImageSrc + OwnedImage;



pub(self) fn pixel_count(dim: [usize;3]) -> usize {
    dim[0].checked_mul(dim[1])
        .and_then(|m| m.checked_mul(dim[2]))
        .expect("Overflow when computing buffer size")
}
