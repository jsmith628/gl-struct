use super::*;

macro_rules! impl_pixel_src_deref {
    (for<$($a:lifetime,)* $P:ident> $ty:ty $(where $($where:tt)*)?) => {
        impl<$($a,)* $P:PixelSrc+?Sized> PixelSrc for $ty $(where $($where)*)? {
            type Pixels = $P::Pixels;
            fn pixel_ptr(&self) -> PixelPtr<$P::Pixels> { (&**self).pixel_ptr() }
        }
    }
}

macro_rules! impl_pixel_dst_deref {
    (for<$($a:lifetime,)* $P:ident> $ty:ty $(where $($where:tt)*)?) => {
        impl_pixel_src_deref!(for<$($a,)* $P> $ty $(where $($where)*)?);
        impl<$($a,)* $P:PixelDst+?Sized> PixelDst for $ty $(where $($where)*)? {
            fn pixel_ptr_mut(&mut self) -> PixelPtrMut<$P::Pixels> { (&mut **self).pixel_ptr_mut() }
        }
    }
}

impl_pixel_dst_deref!(for<P> Box<P>);
impl_pixel_src_deref!(for<P> Rc<P>);
impl_pixel_src_deref!(for<P> Arc<P>);
impl_pixel_src_deref!(for<'a,P> &'a P);
impl_pixel_dst_deref!(for<'a,P> &'a mut P);
impl_pixel_src_deref!(for<'a,P> Cow<'a,P> where P:ToOwned);

impl<P> FromPixels for Box<[P]> {
    type GL = GL10;
    type Hint = ();
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<[P]>)>(_:&GL10, _:(), size: usize, get:G) -> Self {
        let mut dest = Box::new_uninit_slice(size);
        get(PixelPtrMut::Slice((&mut *dest) as *mut [MaybeUninit<P>] as *mut [P]));
        dest.assume_init()
    }
}

impl<P> FromPixels for Rc<[P]> {
    type GL = GL10;
    type Hint = ();
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<[P]>)>(_:&GL10, _:(), count: usize, get:G) -> Self {
        let mut dest = Rc::new_uninit_slice(count);
        get(PixelPtrMut::Slice(Rc::get_mut_unchecked(&mut dest) as *mut [MaybeUninit<P>] as *mut [P]));
        dest.assume_init()
    }
}

impl<P> FromPixels for Arc<[P]> {
    type GL = GL10;
    type Hint = ();
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<[P]>)>(_:&GL10, _:(), count: usize, get:G) -> Self {
        let mut dest = Arc::new_uninit_slice(count);
        get(PixelPtrMut::Slice(Arc::get_mut_unchecked(&mut dest) as *mut [MaybeUninit<P>] as *mut [P]));
        dest.assume_init()
    }
}

impl<'a,P:PixelSrc+ToOwned+?Sized> FromPixels for Cow<'a,P> where
    P::Owned: PixelSrc<Pixels=P::Pixels> + FromPixels
{

    type GL = <P::Owned as FromPixels>::GL;
    type Hint = <P::Owned as FromPixels>::Hint;

    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<P::Pixels>)>(
        gl:&Self::GL, hint:Self::Hint, count: usize, get:G
    ) -> Self {
        Cow::Owned(<P::Owned as FromPixels>::from_pixels(gl,hint,count,get))
    }

}
