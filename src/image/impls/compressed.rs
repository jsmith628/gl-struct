use super::*;

macro_rules! impl_img_src_compressed {
    (for<$($a:lifetime,)* $F:ident $(, $A:ident:$bound:ident)* > $ty:ty) => {
        unsafe impl<$($a,)* $($A:$bound,)* $F:SpecificCompressed> ImageSrc for $ty {

            type Pixels = CompressedPixels<$F>;

            fn swap_bytes(&self) -> bool {false}
            fn lsb_first(&self) -> bool {false}
            fn row_alignment(&self) -> PixelRowAlignment {PixelRowAlignment(1)}

            fn row_length(&self) -> usize {self.width()}
            fn image_height(&self) -> usize {self.height()}

            fn width(&self) -> usize {self.blocks() * $F::block_width() as usize}
            fn height(&self) -> usize {$F::block_height() as usize}
            fn depth(&self) -> usize {$F::block_depth() as usize}

            fn skip_pixels(&self) -> usize {0}
            fn skip_rows(&self) -> usize {0}
            fn skip_images(&self) -> usize {0}

            fn pixels(&self) -> Pixels<CompressedPixels<$F>> { self.pixels() }

        }
    }
}

macro_rules! impl_img_dst_compressed {
    (for<$($a:lifetime,)* $F:ident $(, $A:ident:$bound:ident)* > $ty:ty) => {
        impl_img_src_compressed!(for<$($a,)* $F $(, $A:$bound)* > $ty);
        unsafe impl<$($a,)* $($A:$bound,)* $F:SpecificCompressed> ImageDst for $ty {
            fn pixels_mut(&mut self) -> PixelsMut<CompressedPixels<$F>> { self.pixels_mut() }
        }
    }
}


impl_img_src_compressed!(for<F> CompressedPixels<F>);

// impl_img_src_slice!(for<'a,P> Cow<'a,[P]>);
// impl_own_img_slice!(for<'a,P> Cow<'a,[P]>);

impl_img_src_compressed!(for<'a,F,A:BufferStorage> Slice<'a,CompressedPixels<F>,A>);
impl_img_dst_compressed!(for<'a,F,A:BufferStorage> SliceMut<'a,CompressedPixels<F>,A>);
impl_img_dst_compressed!(for<F,A:BufferStorage> Buffer<CompressedPixels<F>,A>);
