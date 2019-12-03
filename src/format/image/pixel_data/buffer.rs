use super::*;

macro_rules! impl_pixel_src_buf {
    (for<$($a:lifetime,)* $P:ident, $A:ident> $ty:ty) => {
        impl<$($a,)* $P, $A:Initialized> PixelSrc for $ty {
            type Pixels = [$P];
            fn pixel_ptr(&self) -> PixelPtr<[$P]> {
                PixelPtr::Buffer(
                    self.id(),
                    slice_from_raw_parts(Slice::from(self).offset() as *const P, self.len())
                )
            }
        }
    }
}

macro_rules! impl_pixel_dst_buf {
    (for<$($a:lifetime,)* $P:ident, $A:ident> $ty:ty) => {
        impl_pixel_src_buf!(for<$($a,)* $P, $A> $ty);
        impl<$($a,)* $P, $A:Initialized> PixelDst for $ty {
            fn pixel_ptr_mut(&mut self) -> PixelPtrMut<[P]> {
                let slice = SliceMut::from(self);
                PixelPtrMut::Buffer(
                    slice.id(), slice_from_raw_parts_mut(slice.offset() as *mut P, slice.len())
                )
            }
        }
    }
}

impl_pixel_dst_buf!(for<P,A> Buffer<[P],A>);
impl_pixel_src_buf!(for<'a,P,A> Slice<'a,[P],A>);
impl_pixel_dst_buf!(for<'a,P,A> SliceMut<'a,[P],A>);

macro_rules! impl_compressed_src_buf {
    (for<$($a:lifetime,)* $F:ident, $A:ident> $ty:ty) => {
        impl<$($a,)* $F:SpecificCompressed, $A:Initialized> PixelSrc for $ty {
            type Pixels = CompressedPixels<F>;
            fn pixel_ptr(&self) -> PixelPtr<CompressedPixels<F>> {
                PixelPtr::Buffer(
                    self.id(),
                    slice_from_raw_parts(
                        Slice::from(self).offset() as *const F::Block,
                        self.size() / size_of::<F::Block>()
                    ) as *const CompressedPixels<F>
                )
            }
        }
    }
}

macro_rules! impl_compressed_dst_buf {
    (for<$($a:lifetime,)* $F:ident, $A:ident> $ty:ty) => {
        impl_compressed_src_buf!(for<$($a,)* $F, $A> $ty);
        impl<$($a,)* $F:SpecificCompressed, $A:Initialized> PixelDst for $ty {
            fn pixel_ptr_mut(&mut self) -> PixelPtrMut<CompressedPixels<F>> {
                let slice = SliceMut::from(self);
                PixelPtrMut::Buffer(
                    slice.id(),
                    slice_from_raw_parts_mut(
                        slice.offset() as *mut F::Block,
                        slice.size() / size_of::<F::Block>()
                    ) as *mut CompressedPixels<F>
                )
            }
        }
    }
}

impl_compressed_dst_buf!(for<F,A> Buffer<CompressedPixels<F>,A>);
impl_compressed_src_buf!(for<'a,F,A> Slice<'a,CompressedPixels<F>,A>);
impl_compressed_dst_buf!(for<'a,F,A> SliceMut<'a,CompressedPixels<F>,A>);

impl<P,A:Initialized> FromPixels for Buffer<[P],A> {
    default type GL = GL44;
    type Hint = CreationHint;

    default unsafe fn from_pixels<G:FnOnce(PixelPtrMut<[P]>)>(
        _:&Self::GL, hint:CreationHint, count: usize, get:G
    ) -> Self {
        //For persistent Buffers:
        //we assume the GLs are supported as if A is NonPersistent, the specialization covers it
        let mut buf = Buffer::gen(&assume_supported())
            .storage_uninit_slice(&assume_supported(), count, hint.map(|c| c.1));

        get(PixelPtrMut::Buffer((&mut buf).id(), slice_from_raw_parts_mut(null_mut(), count)));
        buf.assume_init()
    }

}

impl<P,A:NonPersistent> FromPixels for Buffer<[P],A> {
    type GL = GL15;
    unsafe fn from_pixels<G:FnOnce(PixelPtrMut<[P]>)>(
        gl:&Self::GL, hint:CreationHint, count: usize, get:G
    ) -> Self {
        let mut buf = Buffer::gen(gl).uninit_slice(count, hint);
        get(PixelPtrMut::Buffer((&mut buf).id(), slice_from_raw_parts_mut(null_mut(), count)));
        buf.assume_init()
    }
}
