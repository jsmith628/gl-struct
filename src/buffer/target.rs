use super::*;

use crate::image::{ImageRef, ImageMut};
use crate::pixels::{PixelSrc, Pixels, PixelsMut};
use crate::format::PixelData;

pub(super) static mut COPY_READ_BUFFER: BindingLocation<BufferTarget> = unsafe {
    BindingLocation::new(BufferTarget::CopyReadBuffer)
};

pub(super) static mut COPY_WRITE_BUFFER: BindingLocation<BufferTarget> = unsafe {
    BindingLocation::new(BufferTarget::CopyReadBuffer)
};

pub(crate) static mut ARRAY_BUFFER: BindingLocation<BufferTarget> = unsafe {
    BindingLocation::new(BufferTarget::ArrayBuffer)
};

pub(crate) static mut PIXEL_PACK_BUFFER: BindingLocation<BufferTarget> = unsafe {
    BindingLocation::new(BufferTarget::PixelPackBuffer)
};

pub(crate) static mut PIXEL_UNPACK_BUFFER: BindingLocation<BufferTarget> = unsafe {
    BindingLocation::new(BufferTarget::PixelUnpackBuffer)
};

glenum!{
    ///Binding targets for [glBindBuffer()](gl::BindBuffer()) and OpenGL calls
    ///acting on those targets
    pub enum BufferTarget {
        [ArrayBuffer ARRAY_BUFFER "Array Buffer"],
        [ElementArrayBuffer ELEMENT_ARRAY_BUFFER "Element Array Buffer"],
        [CopyReadBuffer COPY_READ_BUFFER "Copy Read Buffer"],
        [CopyWriteBuffer COPY_WRITE_BUFFER "Copy Write Buffer"],
        [PixelUnpackBuffer PIXEL_UNPACK_BUFFER "Pixel Unpack Buffer"],
        [PixelPackBuffer PIXEL_PACK_BUFFER "Pixel Pack Buffer"],
        [QueryBuffer QUERY_BUFFER "Query Buffer"],
        [TextureBuffer TEXTURE_BUFFER "Texture Buffer"],
        [TransformFeedbackBuffer  TRANSFORM_FEEDBACK_BUFFER "Transform Feedback Buffer"],
        [UniformBuffer UNIFORM_BUFFER "Uniform Buffer"],
        [DrawIndirectBuffer DRAW_INDIRECT_BUFFER "Draw Indirect Buffer"],
        [AtomicCounterBuffer ATOMIC_COUNTER_BUFFER "Atomic Counter Buffer"],
        [DispatchIndirectBuffer DISPATCH_INDIRECT_BUFFER "Dispatch Indirect Buffer"],
        [ShaderStorageBuffer SHADER_STORAGE_BUFFER "Shader Storage Buffer"]
    }

    ///Binding targets for [glBindBufferBase()](gl::BindBufferBase()),
    ///[glBindBufferRange()](gl::BindBufferRange()), and OpenGL calls acting on those targets
    pub enum IndexedBufferTarget {
        [TransformFeedbackBuffer  TRANSFORM_FEEDBACK_BUFFER "Transform Feedback Buffer"],
        [UniformBuffer UNIFORM_BUFFER "Uniform Buffer"],
        [AtomicCounterBuffer ATOMIC_COUNTER_BUFFER "Atomic Counter Buffer"],
        [ShaderStorageBuffer SHADER_STORAGE_BUFFER "Shader Storage Buffer"]
    }
}

impl<T:?Sized,A> Target<Buffer<T,A>> for BufferTarget {
    fn target_id(self) -> GLenum { self as GLenum }
    unsafe fn bind(self, buf:&Buffer<T,A>) { gl::BindBuffer(self.into(), buf.id()) }
    unsafe fn unbind(self) { gl::BindBuffer(self.into(), 0) }
}

impl<T:?Sized> Target<BufPtr<T>> for BufferTarget {
    fn target_id(self) -> GLenum { self as GLenum }
    unsafe fn bind(self, buf:&BufPtr<T>) { gl::BindBuffer(self.into(), buf.id()) }
    unsafe fn unbind(self) { gl::BindBuffer(self.into(), 0) }
}

impl<'a,T:?Sized,A:BufferStorage> Target<Slice<'a,T,A>> for BufferTarget {
    fn target_id(self) -> GLenum { self as GLenum }
    unsafe fn bind(self, buf:&Slice<T,A>) { gl::BindBuffer(self.into(), buf.id()) }
    unsafe fn unbind(self) { gl::BindBuffer(self.into(), 0) }
}

impl<'a,T:?Sized,A:BufferStorage> Target<SliceMut<'a,T,A>> for BufferTarget {
    fn target_id(self) -> GLenum { self as GLenum }
    unsafe fn bind(self, buf:&SliceMut<T,A>) { gl::BindBuffer(self.into(), buf.id()) }
    unsafe fn unbind(self) { gl::BindBuffer(self.into(), 0) }
}

impl<'a,T:?Sized,A:BufferStorage> Target<GLRef<'a,T,A>> for BufferTarget {
    fn target_id(self) -> GLenum { self as GLenum }
    unsafe fn unbind(self) { gl::BindBuffer(self.into(), 0) }
    unsafe fn bind(self, buf:&GLRef<T,A>) { if let GLRef::Buf(ptr) = buf { self.bind(ptr) } }
}

impl<'a,T:?Sized,A:BufferStorage> Target<GLMut<'a,T,A>> for BufferTarget {
    fn target_id(self) -> GLenum { self as GLenum }
    unsafe fn unbind(self) { gl::BindBuffer(self.into(), 0) }
    unsafe fn bind(self, buf:&GLMut<'a,T,A>) { if let GLMut::Buf(ptr) = buf { self.bind(ptr) } }
}

impl<'a,T:?Sized,GL> Target<Pixels<'a,T,GL>> for BufferTarget {
    fn target_id(self) -> GLenum { self as GLenum }
    unsafe fn unbind(self) { gl::BindBuffer(self.into(), 0) }
    unsafe fn bind(self, buf:&Pixels<'a,T,GL>) { self.bind(&buf.borrow()) }
}

impl<'a,T:?Sized,GL> Target<PixelsMut<'a,T,GL>> for BufferTarget {
    fn target_id(self) -> GLenum { self as GLenum }
    unsafe fn unbind(self) { gl::BindBuffer(self.into(), 0) }
    unsafe fn bind(self, buf:&PixelsMut<'a,T,GL>) { self.bind(&buf.borrow()) }
}

impl<'a,T:PixelData+?Sized,GL:GLVersion> Target<ImageRef<'a,T,GL>> for BufferTarget {
    fn target_id(self) -> GLenum { self as GLenum }
    unsafe fn unbind(self) { gl::BindBuffer(self.into(), 0) }
    unsafe fn bind(self, buf:&ImageRef<'a,T,GL>) { self.bind(&buf.base_image().pixels()) }
}

impl<'a,T:PixelData+?Sized,GL:GLVersion> Target<ImageMut<'a,T,GL>> for BufferTarget {
    fn target_id(self) -> GLenum { self as GLenum }
    unsafe fn unbind(self) { gl::BindBuffer(self.into(), 0) }
    unsafe fn bind(self, buf:&ImageMut<'a,T,GL>) { self.bind(&buf.base_image().pixels()) }
}
