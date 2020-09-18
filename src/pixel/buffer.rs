use super::*;

macro_rules! impl_pixel_src_buf {
    (for<$($a:lifetime,)* $($T:ident $(:$bound:ident)?),*> $ty:ty; $pixels:ty) => {
        impl<$($a,)* $($T $(:$bound)?),*> PixelSrc for $ty {
            type Pixels = $pixels;
            type GL = GL_ARB_pixel_buffer_object;
            fn pixels(&self, gl: Self::GL) -> Pixels<$pixels> {
                Pixels::Buffer(gl, self.as_slice().downgrade())
            }
        }
    }
}

macro_rules! impl_pixel_dst_buf {
    (for<$($a:lifetime,)* $($T:ident $(:$bound:ident)?),*> $ty:ty; $pixels:ty) => {
        impl_pixel_src_buf!(for<$($a,)* $($T $(:$bound)?),*> $ty; $pixels);
        impl<$($a,)* $($T $(:$bound)?),*> PixelDst for $ty {
            fn pixels_mut(&mut self, gl: Self::GL) -> PixelsMut<$pixels> {
                PixelsMut::Buffer(gl, self.as_mut_slice().downgrade())
            }
        }
    }
}

impl_pixel_dst_buf!(for<P:Pixel,A:BufferStorage> Buffer<[P],A>; [P]);
impl_pixel_src_buf!(for<'a,P:Pixel,A:BufferStorage> Slice<'a,[P],A>; [P]);
impl_pixel_dst_buf!(for<'a,P:Pixel,A:BufferStorage> SliceMut<'a,[P],A>; [P]);

impl_pixel_dst_buf!(for<F:SpecificCompressed,A:BufferStorage> Buffer<CompressedPixels<F>,A>; CompressedPixels<F>);
impl_pixel_src_buf!(for<'a,F:SpecificCompressed,A:BufferStorage> Slice<'a,CompressedPixels<F>,A>; CompressedPixels<F>);
impl_pixel_dst_buf!(for<'a,F:SpecificCompressed,A:BufferStorage> SliceMut<'a,CompressedPixels<F>,A>; CompressedPixels<F>);

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
