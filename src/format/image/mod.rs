use super::*;
use super::pixel::*;

pub use self::align::*;
pub use self::pixel_data::*;
pub use self::pixel_store::*;

mod align;
mod pixel_data;
mod pixel_store;

use std::num::*;
use std::convert::TryInto;

pub trait ImageSrc<F:ClientFormat>: PixelSrc<F> {
    fn swap_bytes(&self) -> bool;
    fn lsb_first(&self) -> bool;
    fn row_alignment(&self) -> PixelRowAlignment;

    fn skip_pixels(&self) -> usize;
    fn skip_rows(&self) -> usize;
    fn skip_images(&self) -> usize;

    fn row_length(&self) -> usize;
    fn image_height(&self) -> usize;

    fn width(&self) -> NonZeroUsize;
    fn height(&self) -> NonZeroUsize;
    fn depth(&self) -> NonZeroUsize;

    fn dim(&self) -> [NonZeroUsize; 3] { [self.width(), self.height(), self.depth()] }
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

}

pub trait ImageDst<F:ClientFormat> = ImageSrc<F> + PixelDst<F>;

pub trait OwnedImage<F:ClientFormat>: ImageSrc<F> {
    unsafe fn from_gl<GL:FnOnce(PixelStoreSettings, PixelPtrMut<F>)>(dim: [NonZeroUsize;3], gl:GL) -> Self;
}
