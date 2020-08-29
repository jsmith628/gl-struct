use super::*;

macro_rules! impl_img_src_slice {
    (for<$($a:lifetime,)* $P:ident $(, $A:ident:$bound:ident)* > $ty:ty) => {
        unsafe impl<$($a,)* $($A:$bound,)* $P:Pixel> ImageSrc for $ty {

            type Pixels = [$P];

            fn swap_bytes(&self) -> bool {false}
            fn lsb_first(&self) -> bool {false}
            fn row_alignment(&self) -> PixelRowAlignment {PixelRowAlignment(1)}

            fn row_length(&self) -> usize {self.len()}
            fn image_height(&self) -> usize {1}

            fn width(&self) -> usize {self.len()}
            fn height(&self) -> usize {1}
            fn depth(&self) -> usize {1}

            fn skip_pixels(&self) -> usize {0}
            fn skip_rows(&self) -> usize {0}
            fn skip_images(&self) -> usize {0}

            fn pixels(&self) -> PixelPtr<[$P]> { self.pixel_ptr() }

        }
    }
}

macro_rules! impl_img_dst_slice {
    (for<$($a:lifetime,)* $P:ident $(, $A:ident:$bound:ident)* > $ty:ty) => {
        impl_img_src_slice!(for<$($a,)* $P $(, $A:$bound)* > $ty);
        unsafe impl<$($a,)* $($A:$bound,)* $P: Pixel> ImageDst for $ty {
            fn pixels_mut(&mut self) -> PixelPtrMut<[$P]> { self.pixel_ptr_mut() }
        }
    }
}

impl_img_dst_slice!(for<P> [P]);
impl_img_dst_slice!(for<P> Vec<P>);

// impl_img_src_slice!(for<'a,P> Cow<'a,[P]>);
// impl_own_img_slice!(for<'a,P> Cow<'a,[P]>);

impl_img_src_slice!(for<'a,P,A:BufferStorage> Slice<'a,[P],A>);
impl_img_dst_slice!(for<'a,P,A:BufferStorage> SliceMut<'a,[P],A>);
impl_img_dst_slice!(for<P,A:BufferStorage> Buffer<[P],A>);
