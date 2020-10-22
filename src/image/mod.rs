use super::*;
use crate::format::*;
use crate::pixels::*;
use crate::buffer::*;
use crate::version::*;

pub use self::client_image::*;
pub use self::client_sub_image::*;

mod client_image;
mod client_sub_image;

use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;
use std::convert::TryInto;
use std::cell::{Ref, RefMut};

pub type ImageRef<'a,P,GL> = ClientSubImage<ClientImage<Pixels<'a,P,GL>>>;
pub type ImageMut<'a,P,GL> = ClientSubImage<ClientImage<PixelsMut<'a,P,GL>>>;

pub trait ImageSrc {
    type Pixels: PixelData+?Sized;
    type GL: GLVersion;
    fn image(&self) -> ImageRef<Self::Pixels,Self::GL>;
}

pub trait ImageDst: ImageSrc {
    fn image_mut(&mut self) -> ImageMut<Self::Pixels,Self::GL>;
}

//
//Implement on the PixelData types
//

impl<P:Pixel> ImageSrc for [P] {
    type Pixels = [P];
    type GL = ();
    fn image(&self) -> ImageRef<[P],()> { ClientImage::new_1d(self.pixels()).as_sub_image() }
}

impl<P:Pixel> ImageDst for [P] {
    fn image_mut(&mut self) -> ImageMut<[P],()> { ClientImage::new_1d(self.pixels_mut()).as_sub_image() }
}

impl<F:SpecificCompressed> ImageSrc for Cmpr<F> {
    type Pixels = Cmpr<F>;
    type GL = ();
    fn image(&self) -> ImageRef<Cmpr<F>,()> { ClientImage::new_1d(self.pixels()).as_sub_image() }
}

impl<F:SpecificCompressed> ImageDst for Cmpr<F> {
    fn image_mut(&mut self) -> ImageMut<Cmpr<F>,()> { ClientImage::new_1d(self.pixels_mut()).as_sub_image() }
}

//
//Implement on the smart pointers in the std and this crate
//

macro_rules! impl_img_src_deref {
    (for<$($a:lifetime,)* $I:ident $(, $T:ident)*> $ty:ty $(where $($where:tt)*)?) => {
        impl<$($a,)* $I:ImageSrc+?Sized $(, $T)*> ImageSrc for $ty $(where $($where)*)? {
            type Pixels = $I::Pixels;
            type GL = $I::GL;
            fn image(&self) -> ImageRef<$I::Pixels,$I::GL> { (**self).image() }
        }
    }
}

macro_rules! impl_img_dst_deref {
    (for<$($a:lifetime,)* $I:ident $(, $T:ident)*> $ty:ty $(where $($where:tt)*)?) => {
        impl<$($a,)* $I:ImageDst+?Sized $(, $T)*> ImageDst for $ty $(where $($where)*)? {
            fn image_mut(&mut self) -> ImageMut<$I::Pixels,$I::GL> { (**self).image_mut() }
        }
    }
}

impl_img_src_deref!(for<'a,I> &'a I);
impl_img_src_deref!(for<'a,I> &'a mut I);
impl_img_dst_deref!(for<'a,I> &'a mut I);

impl_img_src_deref!(for<I> Box<I>);
impl_img_dst_deref!(for<I> Box<I>);

impl_img_src_deref!(for<I> Rc<I>);
impl_img_src_deref!(for<I> Arc<I>);

impl_img_src_deref!(for<'a,I> Cow<'a,I> where I:ToOwned);

impl_img_src_deref!(for<'a,I> Ref<'a,I>);
impl_img_src_deref!(for<'a,I> RefMut<'a,I>);
impl_img_dst_deref!(for<'a,I> RefMut<'a,I>);

impl_img_src_deref!(for<'a,I,A> Map<'a,I,A> where A:ReadMappable);
impl_img_dst_deref!(for<'a,I,A> Map<'a,I,A> where A:ReadWriteMappable);

//
//Implement for the buffer types
//

macro_rules! impl_img_src_pixels {
    (for<$($a:lifetime,)* GL=$GL:ty, $P:ident $(, $T:ident)*> $ty:ty $(where $($where:tt)*)?) => {
        impl<$($a,)* $P:PixelData+?Sized $(, $T)*> ImageSrc for $ty $(where $($where)*)? {
            type Pixels = $P;
            type GL = $GL;
            fn image(&self) -> ImageRef<$P,$GL> {
                ClientImage::new_1d(self.pixels()).as_sub_image()
            }
        }
    }
}

macro_rules! impl_img_dst_pixels {
    (for<$($a:lifetime,)* GL=$GL:ty, $P:ident $(, $T:ident)*> $ty:ty $(where $($where:tt)*)?) => {
        impl<$($a,)* $P:PixelData+?Sized $(, $T)*> ImageDst for $ty $(where $($where)*)? {
            fn image_mut(&mut self) -> ImageMut<$P,$GL> {
                ClientImage::new_1d(self.pixels_mut()).as_sub_image()
            }
        }
    }
}

impl_img_src_pixels!(for<'a,GL=GL,P,GL> Pixels<'a,P,GL> where GL:GLVersion);
impl_img_src_pixels!(for<'a,GL=GL,P,GL> PixelsMut<'a,P,GL> where GL:GLVersion);
impl_img_dst_pixels!(for<'a,GL=GL,P,GL> PixelsMut<'a,P,GL> where GL:GLVersion);

impl_img_src_pixels!(for<GL=GL_ARB_pixel_buffer_object,P,A> Buffer<P,A> where A:BufferStorage);
impl_img_dst_pixels!(for<GL=GL_ARB_pixel_buffer_object,P,A> Buffer<P,A> where A:BufferStorage);

impl_img_src_pixels!(for<'a,GL=GL_ARB_pixel_buffer_object,P,A> Slice<'a,P,A> where A:BufferStorage);
impl_img_src_pixels!(for<'a,GL=GL_ARB_pixel_buffer_object,P,A> SliceMut<'a,P,A> where A:BufferStorage);
impl_img_dst_pixels!(for<'a,GL=GL_ARB_pixel_buffer_object,P,A> SliceMut<'a,P,A> where A:BufferStorage);


//
//Implement on Vec
//

impl<P:Pixel> ImageSrc for Vec<P> {
    type Pixels = [P];
    type GL = ();
    fn image(&self) -> ImageRef<[P],()> {
        ClientImage::new_1d(self.pixels()).as_sub_image()
    }
}

impl<P:Pixel> ImageDst for Vec<P> {
    fn image_mut(&mut self) -> ImageMut<[P],()> {
        ClientImage::new_1d(self.pixels_mut()).as_sub_image()
    }
}
