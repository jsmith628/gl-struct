use super::*;

pub unsafe trait PixelSrc<F:ClientFormat> {
    fn pixels(&self) -> PixelPtr<F>;
}
pub unsafe trait PixelDst<F:ClientFormat>: PixelSrc<F> {
    fn pixels_mut(&mut self) -> PixelPtrMut<F>;
}
pub unsafe trait FromPixels<F:ClientFormat>: PixelSrc<F> {
    type GL: GLVersion;
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<F>)>(gl:&Self::GL, count: usize, get:G) -> Self;
}

unsafe impl<F:ClientFormat,P:Pixel<F>> PixelSrc<F> for [P] {
    fn pixels(&self) -> PixelPtr<F> { PixelPtr::Slice(P::format(), self.as_ptr() as *const GLvoid) }
}
unsafe impl<F:ClientFormat,P:Pixel<F>> PixelDst<F> for [P] {
    fn pixels_mut(&mut self) -> PixelPtrMut<F> {
        PixelPtrMut::Slice(P::format(), self.as_mut_ptr() as *mut GLvoid)
    }
}

macro_rules! impl_pixel_src_deref {
    (for<$($a:lifetime,)* $P:ident> $ty:ty) => {
        unsafe impl<$($a,)* F:ClientFormat, $P:Pixel<F>> PixelSrc<F> for $ty {
            fn pixels(&self) -> PixelPtr<F> { (&**self).pixels() }
        }
    }
}

macro_rules! impl_pixel_dst_deref {
    (for<$($a:lifetime,)* $P:ident> $ty:ty) => {
        impl_pixel_src_deref!(for<$($a,)* $P> $ty);
        unsafe impl<$($a,)* F:ClientFormat, $P:Pixel<F>> PixelDst<F> for $ty {
            fn pixels_mut(&mut self) -> PixelPtrMut<F> { (&mut **self).pixels_mut() }
        }
    }
}

impl_pixel_dst_deref!(for<P> Box<[P]>);
impl_pixel_src_deref!(for<P> Rc<[P]>);
impl_pixel_src_deref!(for<P> Arc<[P]>);
impl_pixel_dst_deref!(for<P> Vec<P>);

impl_pixel_src_deref!(for<'a,P> &'a [P]);
impl_pixel_dst_deref!(for<'a,P> &'a mut [P]);
impl_pixel_src_deref!(for<'a,P> Cow<'a,[P]>);


unsafe impl<F:ClientFormat,P:Pixel<F>> FromPixels<F> for Box<[P]> {
    type GL = GL10;
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<F>)>(_:&Self::GL, count: usize, get:G) -> Self {
        let mut dest = Box::new_uninit_slice(count);
        get(PixelPtrMut::Slice(P::format(), dest.as_mut_ptr() as *mut GLvoid));
        dest.assume_init()
    }
}

unsafe impl<F:ClientFormat,P:Pixel<F>> FromPixels<F> for Vec<P> {
    type GL = GL10;
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<F>)>(gl:&Self::GL, count: usize, get:G) -> Self {
        Self::from(Box::from_pixels(gl, count, get))
    }
}

unsafe impl<'a,F:ClientFormat,P:Pixel<F>> FromPixels<F> for Cow<'a,[P]> {
    type GL = GL10;
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<F>)>(gl:&Self::GL, count: usize, get:G) -> Self {
        Self::from(Vec::from_pixels(gl, count, get))
    }
}

unsafe impl<F:ClientFormat,P:Pixel<F>> FromPixels<F> for Rc<[P]> {
    type GL = GL10;
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<F>)>(_:&Self::GL, count: usize, get:G) -> Self {
        let mut dest = Rc::new_uninit_slice(count);
        get(PixelPtrMut::Slice(P::format(), Rc::get_mut_unchecked(&mut dest) as *mut _ as *mut GLvoid));
        dest.assume_init()
    }
}

unsafe impl<F:ClientFormat,P:Pixel<F>> FromPixels<F> for Arc<[P]> {
    type GL = GL10;
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<F>)>(_:&Self::GL, count: usize, get:G) -> Self {
        let mut dest = Arc::new_uninit_slice(count);
        get(PixelPtrMut::Slice(P::format(), Arc::get_mut_unchecked(&mut dest) as *mut _ as *mut GLvoid));
        dest.assume_init()
    }
}

macro_rules! impl_pixel_src_buf {
    (for<$($a:lifetime,)* $P:ident, $A:ident> $ty:ty) => {
        unsafe impl<$($a,)* F:ClientFormat, $P:Pixel<F>, $A:Initialized> PixelSrc<F> for $ty {
            fn pixels(&self) -> PixelPtr<F> {
                PixelPtr::Buffer($P::format(), self.id(), Slice::from(self).offset() as *const GLvoid)
            }
        }
    }
}

macro_rules! impl_pixel_dst_buf {
    (for<$($a:lifetime,)* $P:ident, $A:ident> $ty:ty) => {
        impl_pixel_src_buf!(for<$($a,)* $P, $A> $ty);
        unsafe impl<$($a,)* F:ClientFormat, $P:Pixel<F>, $A:Initialized> PixelDst<F> for $ty {
            fn pixels_mut(&mut self) -> PixelPtrMut<F> {
                PixelPtrMut::Buffer($P::format(), self.id(), SliceMut::from(self).offset() as *mut GLvoid)
            }
        }
    }
}

impl_pixel_dst_buf!(for<P,A> Buffer<[P],A>);
impl_pixel_src_buf!(for<'a,P,A> Slice<'a,[P],A>);
impl_pixel_dst_buf!(for<'a,P,A> SliceMut<'a,[P],A>);

unsafe impl<F:ClientFormat,P:Pixel<F>,A:Initialized> FromPixels<F> for Buffer<[P],A> {
    default type GL = GL44;
    default unsafe fn from_pixels<G:FnOnce(PixelPtrMut<F>)>(_:&Self::GL, count: usize, get:G) -> Self {
        //For persistent Buffers:
        //we assume the GLs are supported as if A is NonPersistent, the specialization covers it

        let mut buf = Buffer::gen(&assume_supported()).storage_uninit_slice(&assume_supported(), count, None);
        get(PixelPtrMut::Buffer(P::format(), buf.id(), SliceMut::from(&mut buf).offset() as *mut GLvoid));
        buf.assume_init()
    }
}

unsafe impl<F:ClientFormat,P:Pixel<F>,A:NonPersistent> FromPixels<F> for Buffer<[P],A> {
    type GL = GL15;
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<F>)>(gl:&Self::GL, count: usize, get:G) -> Self {
        let mut buf = Buffer::gen(gl).uninit_slice(count, None);
        get(PixelPtrMut::Buffer(P::format(), buf.id(), SliceMut::from(&mut buf).offset() as *mut GLvoid));
        buf.assume_init()
    }
}
