use super::*;

macro_rules! impl_own_img {
    (for<$($a:lifetime,)* $($A:ident $(:$bound:ident)?),* > $ty:ty) => {
        unsafe impl<$($a,)* $($A $(:$bound)?),*> OwnedImage for $ty {

            type GL = <Self as FromPixels>::GL;
            type Hint = <Self as FromPixels>::Hint;

            unsafe fn from_gl<G:FnOnce(PixelStore, PixelPtrMut<<Self as PixelSrc>::Pixels>)>(
                gl:&Self::GL, hint: Self::Hint, dim: [usize;3], get:G
            ) -> Self {
                let count = pixel_count(dim);
                let settings = Default::default();
                Self::from_pixels(gl, hint, count, |ptr| get(settings, ptr))
            }

        }
    }
}

impl_own_img!(for<P> Vec<P>);

impl_own_img!(for<P> Box<[P]>);
impl_own_img!(for<F:SpecificCompressed> Box<CompressedPixels<F>>);

impl_own_img!(for<P> Rc<[P]>);
impl_own_img!(for<F:SpecificCompressed> Rc<CompressedPixels<F>>);

impl_own_img!(for<P> Arc<[P]>);
impl_own_img!(for<F:SpecificCompressed> Arc<CompressedPixels<F>>);

impl_own_img!(for<P,A:Initialized> Buffer<[P],A>);
impl_own_img!(for<F:SpecificCompressed,A:Initialized> Buffer<CompressedPixels<F>,A>);

unsafe impl<'a,P:ImageSrc+ToOwned+?Sized> OwnedImage for Cow<'a,P> where P::Owned: OwnedImage<Pixels=P::Pixels> {
    type GL = <P::Owned as OwnedImage>::GL;
    type Hint = <P::Owned as OwnedImage>::Hint;

    unsafe fn from_gl<G:FnOnce(PixelStore, PixelPtrMut<<Self as ImageSrc>::Pixels>)>(
        gl:&Self::GL, hint: Self::Hint, dim: [usize;3], get:G
    ) -> Self {
        Cow::Owned(P::Owned::from_gl(gl,hint,dim,get))
    }
}
