use super::*;
use crate::Target;

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

impl BindingLocation<UninitBuf> {
    #[inline]
    pub fn bind_buf<'a,T:?Sized,A:BufferAccess>(&'a mut self, buf: &'a Buf<T,A>) -> Binding<'a,UninitBuf> {
        unsafe { self.target().bind(buf.id()); }
        Binding(self, buf.id())
    }

    #[inline]
    pub fn bind_slice<'a,T:?Sized,A:BufferAccess>(&'a mut self, buf: &'a BSlice<'a,T,A>) -> Binding<'a,UninitBuf> {
        unsafe { self.target().bind(buf.id()); }
        Binding(self, buf.id())
    }

    #[inline]
    pub fn bind_slice_mut<'a,T:?Sized,A:BufferAccess>(&'a mut self, buf: &'a BSliceMut<'a,T,A>) -> Binding<'a,UninitBuf> {
        unsafe { self.target().bind(buf.id()); }
        Binding(self, buf.id())
    }
}
