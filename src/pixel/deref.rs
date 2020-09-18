use super::*;

macro_rules! impl_pixel_src_deref {
    (for<$($a:lifetime,)* $P:ident> $ty:ty $(where $($where:tt)*)?) => {
        impl<$($a,)* $P:PixelSrc+?Sized> PixelSrc for $ty $(where $($where)*)? {
            type Pixels = $P::Pixels;
            type GL = $P::GL;
            fn pixels(&self, gl: $P::GL) -> Pixels<$P::Pixels> { (&**self).pixels(gl) }
        }
    }
}

macro_rules! impl_pixel_dst_deref {
    (for<$($a:lifetime,)* $P:ident> $ty:ty $(where $($where:tt)*)?) => {
        impl_pixel_src_deref!(for<$($a,)* $P> $ty $(where $($where)*)?);
        impl<$($a,)* $P:PixelDst+?Sized> PixelDst for $ty $(where $($where)*)? {
            fn pixels_mut(&mut self, gl: $P::GL) -> PixelsMut<$P::Pixels> { (&mut **self).pixels_mut(gl) }
        }
    }
}

impl_pixel_dst_deref!(for<P> Box<P>);
impl_pixel_src_deref!(for<P> Rc<P>);
impl_pixel_src_deref!(for<P> Arc<P>);
impl_pixel_src_deref!(for<'a,P> &'a P);
impl_pixel_dst_deref!(for<'a,P> &'a mut P);
impl_pixel_src_deref!(for<'a,P> Cow<'a,P> where P:ToOwned);

// impl<P:Pixel> FromPixels for Box<[P]> {
//     type Hint = ();
//     unsafe fn from_pixels<G:FnOnce(PixelsMut<[P]>)>(_:&Self::GL, _:(), size: usize, get:G) -> Self {
//         let mut dest = Box::new_uninit_slice(size);
//         get(PixelsMut::Slice((&mut *dest) as *mut [MaybeUninit<P>] as *mut [P]));
//         dest.assume_init()
//     }
// }
//
// impl<P:Pixel> FromPixels for Rc<[P]> {
//     type Hint = ();
//     unsafe fn from_pixels<G:FnOnce(PixelsMut<[P]>)>(_:&Self::GL, _:(), count: usize, get:G) -> Self {
//         let mut dest = Rc::new_uninit_slice(count);
//         get(PixelsMut::Slice(Rc::get_mut_unchecked(&mut dest) as *mut [MaybeUninit<P>] as *mut [P]));
//         dest.assume_init()
//     }
// }
//
// impl<P:Pixel> FromPixels for Arc<[P]> {
//     type Hint = ();
//     unsafe fn from_pixels<G:FnOnce(PixelsMut<[P]>)>(_:&Self::GL, _:(), count: usize, get:G) -> Self {
//         let mut dest = Arc::new_uninit_slice(count);
//         get(PixelsMut::Slice(Arc::get_mut_unchecked(&mut dest) as *mut [MaybeUninit<P>] as *mut [P]));
//         dest.assume_init()
//     }
// }
//
// macro_rules! impl_compressed_from_block {
//     (for<$($a:lifetime, )* $F:ident $(, $A:ident:$bound:ident)*> $ty:ty as $arr:ty) => {
//         impl<$($a, )* $F:SpecificCompressed $(, $A:$bound)*> FromPixels for $ty {
//
//             type Hint = ();
//
//             unsafe fn from_pixels<G:FnOnce(PixelsMut<CompressedPixels<$F>>)>(
//                 gl:&Self::GL, hint:Self::Hint, count: usize, get:G
//             ) -> Self {
//
//                 let block_pixels = F::block_width() as usize * F::block_height() as usize * F::block_depth() as usize;
//                 let num_blocks = count / block_pixels + if count%block_pixels==0 {0} else {1};
//
//                 let dest = <$arr>::new_uninit_slice(num_blocks);
//                 get(PixelsMut::Slice(
//                     &*dest as *const [MaybeUninit<F::Block>] as *mut [MaybeUninit<F::Block>] as *mut CompressedPixels<F>
//                 ));
//                 transmute::<$arr,$ty>(dest.assume_init())
//             }
//
//         }
//     }
// }
//
// impl_compressed_from_block!(for<F> Box<CompressedPixels<F>> as Box<[F::Block]>);
// impl_compressed_from_block!(for<F> Rc<CompressedPixels<F>> as Rc<[F::Block]>);
// impl_compressed_from_block!(for<F> Arc<CompressedPixels<F>> as Arc<[F::Block]>);
//
// impl<'a,P:PixelSrc+ToOwned+?Sized> FromPixels for Cow<'a,P> where
//     P::Owned: PixelSrc<Pixels=P::Pixels, GL=P::GL> + FromPixels
// {
//
//     type Hint = <P::Owned as FromPixels>::Hint;
//
//     unsafe fn from_pixels<G:FnOnce(PixelsMut<P::Pixels>)>(
//         gl:&Self::GL, hint:Self::Hint, count: usize, get:G
//     ) -> Self {
//         Cow::Owned(<P::Owned as FromPixels>::from_pixels(gl,hint,count,get))
//     }
//
// }
