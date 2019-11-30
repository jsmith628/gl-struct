use super::*;
use super::pixel::*;

pub use self::align::*;
pub use self::pixel_data::*;
pub use self::pixel_store::*;

mod align;
mod pixel_data;
mod pixel_store;

use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;
use std::convert::TryInto;
use object::buffer::*;
use context::*;

pub trait ImageSrc<F:ClientFormat>: PixelSrc<F> {
    fn swap_bytes(&self) -> bool;
    fn lsb_first(&self) -> bool;
    fn row_alignment(&self) -> PixelRowAlignment;

    fn skip_pixels(&self) -> usize {0}
    fn skip_rows(&self) -> usize {0}
    fn skip_images(&self) -> usize {0}

    fn row_length(&self) -> usize {0}
    fn image_height(&self) -> usize {0}

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

}

pub trait ImageDst<F:ClientFormat> = ImageSrc<F> + PixelDst<F>;

pub trait OwnedImage<F:ClientFormat>: ImageSrc<F> {
    type GL: GLVersion;
    unsafe fn from_gl<G:FnOnce(PixelStoreSettings, PixelPtrMut<F>)>(
        gl:&Self::GL, dim: [usize;3], get:G
    ) -> Self;
}

macro_rules! impl_img_src_slice {
    (for<$($a:lifetime,)* $P:ident $(, $A:ident:$bound:ident)* > $ty:ty) => {
        impl<$($a,)* $($A:$bound,)* F:ClientFormat, $P:Pixel<F>> ImageSrc<F> for $ty {

            fn swap_bytes(&self) -> bool {$P::swap_bytes()}
            fn lsb_first(&self) -> bool {$P::lsb_first()}
            fn row_alignment(&self) -> PixelRowAlignment {PixelRowAlignment(1)}

            fn width(&self) -> usize {self.len()}
            fn height(&self) -> usize {1}
            fn depth(&self) -> usize {1}

        }
    }
}

pub(self) fn pixel_count(dim: [usize;3]) -> usize {
    dim[0].checked_mul(dim[1])
        .and_then(|m| m.checked_mul(dim[2]))
        .expect("Overflow when computing buffer size")
}

macro_rules! impl_own_img_slice {
    (for<$($a:lifetime,)* $P:ident $(, $A:ident:$bound:ident)* > $ty:ty) => {
        impl<$($a,)* $($A:$bound,)* F:ClientFormat, $P:Pixel<F>> OwnedImage<F> for $ty {
            type GL = <Self as FromPixels<F>>::GL;
            unsafe fn from_gl<G:FnOnce(PixelStoreSettings, PixelPtrMut<F>)>(
                gl:&Self::GL, dim: [usize;3], get:G
            ) -> Self {
                let count = pixel_count(dim);

                let settings = PixelStoreSettings {
                    swap_bytes: $P::swap_bytes(),
                    lsb_first: $P::lsb_first(),
                    row_alignment: PixelRowAlignment(1),
                    skip_pixels: 0, skip_rows: 0, skip_images: 0,
                    row_length: 0, image_height: 0,
                };

                Self::from_pixels(gl, count, |ptr| get(settings, ptr))
            }
        }
    }
}

impl_img_src_slice!(for<P> [P]);

impl_img_src_slice!(for<P> Box<[P]>);
impl_own_img_slice!(for<P> Box<[P]>);

impl_img_src_slice!(for<P> Vec<P>);
impl_own_img_slice!(for<P> Vec<P>);

impl_img_src_slice!(for<P> Rc<[P]>);
impl_own_img_slice!(for<P> Rc<[P]>);

impl_img_src_slice!(for<P> Arc<[P]>);
impl_own_img_slice!(for<P> Arc<[P]>);

impl_img_src_slice!(for<'a,P> Cow<'a,[P]>);
impl_own_img_slice!(for<'a,P> Cow<'a,[P]>);

impl_img_src_slice!(for<'a,P,A:Initialized> Slice<'a,[P],A>);
impl_img_src_slice!(for<'a,P,A:Initialized> SliceMut<'a,[P],A>);

impl_img_src_slice!(for<P,A:Initialized> Buffer<[P],A>);
impl_own_img_slice!(for<P,A:Initialized> Buffer<[P],A>);
