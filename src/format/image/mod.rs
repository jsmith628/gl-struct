use super::*;
use super::pixel::*;

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
use object::buffer::*;
use context::*;

pub trait ImageSrc {

    type Pixel;

    fn swap_bytes(&self) -> bool;
    fn lsb_first(&self) -> bool;
    fn row_alignment(&self) -> PixelRowAlignment;

    fn skip_pixels(&self) -> usize {0}
    fn skip_rows(&self) -> usize {0}
    fn skip_images(&self) -> usize {0}

    fn row_length(&self) -> usize {self.width()}
    fn image_height(&self) -> usize {self.height()}

    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn depth(&self) -> usize;

    fn dim(&self) -> [usize; 3] { [self.width(), self.height(), self.depth()] }

    //since the image is already allocated, we can assume that the size does not overflow
    fn pixel_count(&self) -> usize { self.width() * self.height() * self.depth() }

    fn settings(&self) -> PixelStoreSettings {
        PixelStoreSettings {
            swap_bytes: self.swap_bytes(),
            lsb_first: self.lsb_first(),
            row_alignment: self.row_alignment(),
            skip_pixels: self.skip_pixels(),
            skip_rows: self.skip_rows(),
            skip_images: self.skip_images(),
            row_length: self.row_length(),
            image_height: self.image_height(),
        }
    }

    fn pixels(&self) -> PixelPtr<[Self::Pixel]>;

}

pub trait ImageDst: ImageSrc {
    fn pixels_mut(&mut self) -> PixelPtrMut<[Self::Pixel]>;
}

pub trait OwnedImage: ImageSrc {
    type GL: GLVersion;
    type Hint;
    unsafe fn from_gl<G:FnOnce(PixelStoreSettings, PixelPtrMut<[Self::Pixel]>)>(
        gl:&Self::GL, hint:Self::Hint, dim: [usize;3], get:G
    ) -> Self;
}



pub(self) fn pixel_count(dim: [usize;3]) -> usize {
    dim[0].checked_mul(dim[1])
        .and_then(|m| m.checked_mul(dim[2]))
        .expect("Overflow when computing buffer size")
}
