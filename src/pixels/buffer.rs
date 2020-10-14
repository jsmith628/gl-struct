use super::*;

macro_rules! impl_pixel_src_buf {
    (for<$($a:lifetime,)* $P:ident, $A:ident> $ty:ty) => {
        impl<$($a,)* $P:PixelData+?Sized, $A:BufferStorage> PixelSrc for $ty {
            type Pixels = $P;
            type GL = GL_ARB_pixel_buffer_object;
            fn pixels(&self) -> Pixels<$P, GL_ARB_pixel_buffer_object> {
                Pixels::from_buf(self.as_slice())
            }
        }
    }
}

macro_rules! impl_pixel_dst_buf {
    (for<$($a:lifetime,)* $P:ident, $A:ident> $ty:ty) => {
        impl_pixel_src_buf!(for<$($a,)* $P, $A> $ty);
        impl<$($a,)* $P:PixelData+?Sized, $A:BufferStorage> PixelDst for $ty {
            fn pixels_mut(&mut self) -> PixelsMut<$P, GL_ARB_pixel_buffer_object> {
                PixelsMut::from_buf(self.as_mut_slice())
            }
        }
    }
}

impl_pixel_dst_buf!(for<P,A> Buffer<P,A>);
impl_pixel_src_buf!(for<'a,P,A> Slice<'a,P,A>);
impl_pixel_dst_buf!(for<'a,P,A> SliceMut<'a,P,A>);

// impl<P,A:BufferStorage> FromPixels for Buffer<[P],A> {
//     default type GL = GL44;
//     type Hint = CreationHint;
//
//     default unsafe fn from_pixels<G:FnOnce(PixelPtrMut<[P]>)>(
//         _:&Self::GL, hint:CreationHint, count: usize, get:G
//     ) -> Self {
//         //For persistent Buffers:
//         //we assume the GLs are supported as if A is NonPersistent, the specialization covers it
//         let mut buf = Buffer::create(&assume_supported::<GL_ARB_vertex_buffer_object>())
//             .storage_uninit_slice(&assume_supported::<GL_ARB_buffer_storage>(), count, hint.map(|c| c.1));
//
//         get(PixelPtrMut::Buffer((&mut buf).id(), slice_from_raw_parts_mut(null_mut(), count)));
//         buf.assume_init()
//     }
//
// }
//
// impl<P,A:NonPersistent> FromPixels for Buffer<[P],A> {
//     type GL = GL15;
//     unsafe fn from_pixels<G:FnOnce(PixelPtrMut<[P]>)>(
//         gl:&Self::GL, hint:CreationHint, count: usize, get:G
//     ) -> Self {
//         let mut buf = Buffer::create(gl).uninit_slice(count, hint);
//         get(PixelPtrMut::Buffer((&mut buf).id(), slice_from_raw_parts_mut(null_mut(), count)));
//         buf.assume_init()
//     }
// }
