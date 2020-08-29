use super::*;

unsafe impl<P:Pixel> PixelSrc for Vec<P> {
    type Pixels = [P];
    type GL = P::GL;
    fn pixel_ptr(&self) -> PixelPtr<[P]> { (&**self).pixel_ptr() }
}

unsafe impl<P:Pixel> PixelDst for Vec<P> {
    fn pixel_ptr_mut(&mut self) -> PixelPtrMut<[P]> { (&mut **self).pixel_ptr_mut() }
}

impl<P:Pixel> FromPixels for Vec<P> {

    type Hint = Option<usize>;

    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<[P]>)>(
        _: &Self::GL, hint: Option<usize>, size: usize, get: G
    ) -> Self {
        let mut vec = Vec::with_capacity(size.max(hint.unwrap_or(0)));
        get(PixelPtrMut::Slice((&mut *vec) as *mut [P]));
        vec.set_len(size);
        vec
    }

}
