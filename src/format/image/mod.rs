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

    fn skip_pixels(&self) -> usize {0}
    fn skip_rows(&self) -> usize {0}
    fn skip_images(&self) -> usize {0}

    fn row_length(&self) -> usize {0}
    fn image_height(&self) -> usize {0}

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

macro_rules! impl_img_src_slice {
    (for<$($a:lifetime,)* $P:ident> $ty:ty) => {
        impl<$($a,)* F:ClientFormat, $P:Pixel<F>> ImageSrc<F> for $ty {

            fn swap_bytes(&self) -> bool {$P::swap_bytes()}
            fn lsb_first(&self) -> bool {$P::lsb_first()}
            fn row_alignment(&self) -> PixelRowAlignment {PixelRowAlignment(1)}

            fn width(&self) -> NonZeroUsize {NonZeroUsize::new(self.len()).expect("Images must not be zero-sized")}
            fn height(&self) -> NonZeroUsize {NonZeroUsize::new(1).unwrap()}
            fn depth(&self) -> NonZeroUsize {NonZeroUsize::new(1).unwrap()}

        }
    }
}

macro_rules! impl_own_img_slice {
    (for<$($a:lifetime,)* $P:ident> $ty:ty) => {
        impl<$($a,)* F:ClientFormat, $P:Pixel<F>> OwnedImage<F> for $ty {
            unsafe fn from_gl<GL:FnOnce(PixelStoreSettings, PixelPtrMut<F>)>(
                dim: [NonZeroUsize;3], gl:GL
            ) -> Self {
                let count = dim[0].get()
                    .checked_mul(dim[1].get())
                    .and_then(|m| m.checked_mul(dim[2].get()))
                    .expect("Overflow when computing buffer size");

                let settings = PixelStoreSettings {
                    swap_bytes: $P::swap_bytes(),
                    lsb_first: $P::lsb_first(),
                    row_alignment: PixelRowAlignment(1),
                    skip_pixels: 0, skip_rows: 0, skip_images: 0,
                    row_length: 0, image_height: 0,
                };

                Self::from_pixels(count, |ptr| gl(settings, ptr))
            }
        }
    }
}

impl_img_src_slice!(for<P> [P]);

impl_img_src_slice!(for<P> Box<[P]>);
impl_own_img_slice!(for<P> Box<[P]>);

impl_img_src_slice!(for<P> Vec<P>);
impl_own_img_slice!(for<P> Vec<P>);

impl_img_src_slice!(for<P> ::std::rc::Rc<[P]>);
impl_own_img_slice!(for<P> ::std::rc::Rc<[P]>);

impl_img_src_slice!(for<P> ::std::sync::Arc<[P]>);
impl_own_img_slice!(for<P> ::std::sync::Arc<[P]>);

impl_img_src_slice!(for<'a,P> ::std::borrow::Cow<'a,[P]>);
impl_own_img_slice!(for<'a,P> ::std::borrow::Cow<'a,[P]>);
