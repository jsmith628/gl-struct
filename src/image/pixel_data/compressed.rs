use super::*;

macro_rules! impl_compressed_from_block {
    (for<$($a:lifetime, )* $F:ident $(, $A:ident:$bound:ident)*> $ty:ty as $arr:ty) => {
        impl<$($a, )* $F:SpecificCompressed $(, $A:$bound)*> FromPixels for $ty {

            type GL = <$arr as FromPixels>::GL;
            type Hint = <$arr as FromPixels>::Hint;

            unsafe fn from_pixels<G:FnOnce(PixelPtrMut<CompressedPixels<$F>>)>(
                gl:&Self::GL, hint:Self::Hint, count: usize, get:G
            ) -> Self {

                let block_pixels = F::block_width() as usize * F::block_height() as usize * F::block_depth() as usize;
                let num_blocks = count / block_pixels + if count%block_pixels==0 {0} else {1};

                let blocks = <$arr as FromPixels>::from_pixels(
                    gl, hint, num_blocks,
                    |ptr| get(transmute(ptr))
                );

                transmute::<$arr,$ty>(blocks)
            }

        }
    }
}

impl_compressed_from_block!(for<F> Box<CompressedPixels<F>> as Box<[F::Block]>);
impl_compressed_from_block!(for<F> Rc<CompressedPixels<F>> as Rc<[F::Block]>);
impl_compressed_from_block!(for<F> Arc<CompressedPixels<F>> as Arc<[F::Block]>);
impl_compressed_from_block!(for<F,A:Initialized> Buffer<CompressedPixels<F>,A> as Buffer<[F::Block],A>);
