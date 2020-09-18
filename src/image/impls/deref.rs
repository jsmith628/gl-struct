use super::*;

macro_rules! impl_img_src_deref {
    (for<$($a:lifetime,)* $P:ident> $ty:ty $(where $($where:tt)*)?) => {
        unsafe impl<$($a,)* $P:ImageSrc+?Sized> ImageSrc for $ty $(where $($where)*)? {

            type Pixels = $P::Pixels;

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

            fn pixels(&self) -> Pixels<Self::Pixels> { (&**self).pixels() }

        }
    }
}

macro_rules! impl_img_dst_deref {
    (for<$($a:lifetime,)* $P:ident> $ty:ty $(where $($where:tt)*)?) => {
        impl_img_src_deref!(for<$($a,)* $P> $ty $(where $($where)*)? );
        unsafe impl<$($a,)* $P:ImageDst+?Sized> ImageDst for $ty $(where $($where)*)? {
            fn pixels_mut(&mut self) -> PixelsMut<Self::Pixels> { (&mut **self).pixels_mut() }
        }
    }
}

impl_img_src_deref!(for<'a,P> &'a P);
impl_img_dst_deref!(for<'a,P> &'a mut P);

impl_img_dst_deref!(for<P> Box<P>);

impl_img_src_deref!(for<P> Rc<P>);
impl_img_src_deref!(for<P> Arc<P>);

impl_img_src_deref!(for<'a,P> Cow<'a,P> where P:ToOwned);
