use super::*;

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

gl_resource! {
    pub struct RawBuffer {
        gl = GL15,
        target = BufferTarget,
        ident = Buffer,
        gen = GenBuffers,
        bind = BindBuffer,
        is = IsBuffer,
        delete = DeleteBuffers
    }
}

impl !Send for RawBuffer {}
impl !Sync for RawBuffer {}

impl BindingLocation<RawBuffer> {
    #[inline]
    pub fn bind_buf<'a,T:?Sized,A:BufferAccess>(&'a mut self, buf: &'a Buffer<T,A>) -> Binding<'a,RawBuffer> {
        unsafe { self.target().bind(buf.id()); }
        Binding(self, buf.id())
    }

    #[inline]
    pub fn bind_slice<'a,T:?Sized,A:BufferAccess>(&'a mut self, buf: &'a Slice<'a,T,A>) -> Binding<'a,RawBuffer> {
        unsafe { self.target().bind(buf.id()); }
        Binding(self, buf.id())
    }

    #[inline]
    pub fn bind_slice_mut<'a,T:?Sized,A:BufferAccess>(&'a mut self, buf: &'a SliceMut<'a,T,A>) -> Binding<'a,RawBuffer> {
        unsafe { self.target().bind(buf.id()); }
        Binding(self, buf.id())
    }
}


///
///Any type that can be cloned within a [buffer](super::Buffer) by simple byte-wise copies of its data.
///
pub unsafe trait GPUCopy {}
unsafe impl<T:Copy> GPUCopy for T {}
unsafe impl<T:Copy> GPUCopy for [T] {}

macro_rules! impl_tuple_gpucopy {
    ({$($T:ident:$t:ident)*} $Last:ident:$l:ident) => {
        unsafe impl<$($T:GPUCopy,)* $Last: Sized> GPUCopy for ($($T,)* [$Last]) where [$Last]:GPUCopy {}
    };
}
impl_tuple!(impl_tuple_gpucopy @with_last);

///Gives a hint as to if the given value needs its destructor run
pub(super) trait NeedsDropVal { fn needs_drop_val(&self) -> bool; }

impl<T:?Sized> NeedsDropVal for T { #[inline] default fn needs_drop_val(&self) -> bool {true} }
impl<T:Sized> NeedsDropVal for [T] { #[inline] fn needs_drop_val(&self) -> bool {self.len()>0 && needs_drop::<T>()} }
impl<T:Sized> NeedsDropVal for T { #[inline] fn needs_drop_val(&self) -> bool {needs_drop::<T>()} }
