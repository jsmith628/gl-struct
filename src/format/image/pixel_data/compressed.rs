use super::*;

macro_rules! impl_compressed_from_block {
    (for<$F:ident $(, $A:ident:$bound:ident)*> $ty:ty as $arr:ty) => {
        impl<$F:SpecificCompressed $(, $A:$bound)*> FromPixels for $ty {

            type GL = <$arr as FromPixels>::GL;
            type Hint = <$arr as FromPixels>::Hint;

            unsafe fn from_pixels<G:FnOnce(PixelPtrMut<CompressedPixels<$F>>)>(
                gl:&Self::GL, hint:Self::Hint, count: usize, get:G
            ) -> Self {
                let blocks = <$arr as FromPixels>::from_pixels(
                    gl, hint, count,
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
