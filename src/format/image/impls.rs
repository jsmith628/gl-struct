use super::*;

macro_rules! impl_img_src_slice {
    (for<$($a:lifetime,)* $P:ident $(, $A:ident:$bound:ident)* > $ty:ty) => {
        unsafe impl<$($a,)* $($A:$bound,)* $P> ImageSrc for $ty {

            type Pixel = $P;

            fn swap_bytes(&self) -> bool {false}
            fn lsb_first(&self) -> bool {false}
            fn row_alignment(&self) -> PixelRowAlignment {PixelRowAlignment(1)}

            fn width(&self) -> usize {self.len()}
            fn height(&self) -> usize {1}
            fn depth(&self) -> usize {1}

            fn pixels(&self) -> PixelPtr<[$P]> { self.pixel_ptr() }

        }
    }
}

macro_rules! impl_img_dst_slice {
    (for<$($a:lifetime,)* $P:ident $(, $A:ident:$bound:ident)* > $ty:ty) => {
        impl_img_src_slice!(for<$($a,)* $P $(, $A:$bound)* > $ty);
        unsafe impl<$($a,)* $($A:$bound,)* $P> ImageDst for $ty {
            fn pixels_mut(&mut self) -> PixelPtrMut<[$P]> { self.pixel_ptr_mut() }
        }
    }
}

macro_rules! impl_own_img_slice {
    (for<$($a:lifetime,)* $P:ident $(, $A:ident:$bound:ident)* > $ty:ty) => {
        unsafe impl<$($a,)* $($A:$bound,)* $P> OwnedImage for $ty {

            type GL = <Self as FromPixels>::GL;
            type Hint = <Self as FromPixels>::Hint;

            unsafe fn from_gl<G:FnOnce(PixelStoreSettings, PixelPtrMut<[$P]>)>(
                gl:&Self::GL, hint: Self::Hint, dim: [usize;3], get:G
            ) -> Self {
                let count = pixel_count(dim);
                let settings = Default::default();
                Self::from_pixels(gl, hint, count, |ptr| get(settings, ptr))
            }

        }
    }
}

impl_img_dst_slice!(for<P> [P]);
impl_own_img_slice!(for<P> Box<[P]>);

impl_img_dst_slice!(for<P> Vec<P>);
impl_own_img_slice!(for<P> Vec<P>);

impl_own_img_slice!(for<P> Rc<[P]>);
impl_own_img_slice!(for<P> Arc<[P]>);

// impl_img_src_slice!(for<'a,P> Cow<'a,[P]>);
// impl_own_img_slice!(for<'a,P> Cow<'a,[P]>);

impl_img_src_slice!(for<'a,P,A:Initialized> Slice<'a,[P],A>);
impl_img_dst_slice!(for<'a,P,A:Initialized> SliceMut<'a,[P],A>);

impl_img_dst_slice!(for<P,A:Initialized> Buffer<[P],A>);
impl_own_img_slice!(for<P,A:Initialized> Buffer<[P],A>);

macro_rules! impl_img_src_deref {
    (for<$($a:lifetime,)* $P:ident> $ty:ty) => {
        unsafe impl<$($a,)* $P:ImageSrc+?Sized> ImageSrc for $ty {

            type Pixel = $P::Pixel;

            fn swap_bytes(&self) -> bool { (&**self).swap_bytes() }
            fn lsb_first(&self) -> bool { (&**self).lsb_first() }
            fn row_alignment(&self) -> PixelRowAlignment { (&**self).row_alignment() }

            fn skip_pixels(&self) -> usize { (&**self).skip_pixels() }
            fn skip_rows(&self) -> usize { (&**self).skip_rows() }
            fn skip_images(&self) -> usize { (&**self).skip_images() }

            fn row_length(&self) -> usize { (&**self).row_length() }
            fn image_height(&self) -> usize { (&**self).image_height() }

            fn width(&self) -> usize { (&**self).width() }
            fn height(&self) -> usize { (&**self).height() }
            fn depth(&self) -> usize { (&**self).depth() }

            fn dim(&self) -> [usize; 3] { (&**self).dim() }
            fn pixel_count(&self) -> usize { (&**self).pixel_count() }
            fn settings(&self) -> PixelStoreSettings { (&**self).settings() }

            fn pixels(&self) -> PixelPtr<[Self::Pixel]> { (&**self).pixels() }

        }
    }
}

macro_rules! impl_img_dst_deref {
    (for<$($a:lifetime,)* $P:ident> $ty:ty) => {
        impl_img_src_deref!(for<$($a,)* $P> $ty);
        unsafe impl<$($a,)* $P:ImageDst+?Sized> ImageDst for $ty {
            fn pixels_mut(&mut self) -> PixelPtrMut<[Self::Pixel]> { (&mut **self).pixels_mut() }
        }
    }
}

impl_img_src_deref!(for<'a,P> &'a P);
impl_img_dst_deref!(for<'a,P> &'a mut P);

impl_img_dst_deref!(for<P> Box<P>);

impl_img_src_deref!(for<P> Rc<P>);
impl_img_src_deref!(for<P> Arc<P>);
