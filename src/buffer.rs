
glenum!{
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

    pub enum IndexedBufferTarget {
        [TransformFeedbackBuffer  TRANSFORM_FEEDBACK_BUFFER "Transform Feedback Buffer"],
        [UniformBuffer UNIFORM_BUFFER "Uniform Buffer"],
        [AtomicCounterBuffer ATOMIC_COUNTER_BUFFER "Atomic Counter Buffer"],
        [ShaderStorageBuffer SHADER_STORAGE_BUFFER "Shader Storage Buffer"]
    }

    pub enum BufferUsage {
        [StreamDraw STREAM_DRAW "Stream:Draw"],
        [StreamRead STREAM_READ "Stream:Read"],
        [StreamCopy STREAM_COPY "Stream:Copy"],
        [StaticDraw STATIC_DRAW "Static:Draw"],
        [StaticRead STATIC_READ "Static:Read"],
        [StaticCopy STATIC_COPY "Static:Copy"],
        [DynamicDraw DYNAMIC_DRAW "Dynamic:Draw"],
        [DynamicRead DYNAMIC_READ "Dynamic:Read"],
        [DynamicCopy DYNAMIC_COPY "Dynamic:Copy"]
    }

}

pub trait BufferAccess {

    type Read: Boolean;
    type Write: Boolean;

    type FlipReadBit: BufferAccess<Read=<Self::Read as Boolean>::Not, Write=Self::Write>;
    type FlipWriteBit: BufferAccess<Read=Self::Read, Write=<Self::Write as Boolean>::Not>;
    type NoReadBit: BufferAccess<Read=False, Write=Self::Write>;
    type NoWriteBit: BufferAccess<Read=Self::Read, Write=False>;

    #[inline]
    fn storage_flags(_hint: BufferUsage) -> GLbitfield {
        let f1 = if <Self::Read as Boolean>::VALUE { gl::MAP_READ_BIT } else {0};
        let f2 = if <Self::Write as Boolean>::VALUE { gl::MAP_WRITE_BIT } else {0};
        f1 | f2
    }

    #[inline] fn mapping_flags(hint: BufferUsage) -> GLbitfield { Self::storage_flags(hint) }
    #[inline] fn buffer_usage(hint: BufferUsage) -> BufferUsage { hint }
}

pub trait ReadAccess: BufferAccess {}
pub trait WriteAccess: BufferAccess {}

impl<A:BufferAccess<Read=True>> ReadAccess for A {}
impl<A:BufferAccess<Write=True>> WriteAccess for A {}

pub struct CopyOnly;
pub struct Read;
pub struct Write;
pub struct ReadWrite;


impl BufferAccess for CopyOnly {
    type Read = False; type Write = False;
    type FlipReadBit = Read; type FlipWriteBit = Write;
    type NoReadBit = Self; type NoWriteBit = Self;
}
impl BufferAccess for Read {
    type Read = True; type Write = False;
    type FlipReadBit = CopyOnly; type FlipWriteBit = ReadWrite;
    type NoReadBit = CopyOnly; type NoWriteBit = Self;
}
impl BufferAccess for Write {
    type Read = False; type Write = True;
    type FlipReadBit = ReadWrite; type FlipWriteBit = CopyOnly;
    type NoReadBit = Self; type NoWriteBit = CopyOnly;
}
impl BufferAccess for ReadWrite {
    type Read = True; type Write = True;
    type FlipReadBit = Write; type FlipWriteBit = Read;
    type NoReadBit = Write; type NoWriteBit = Read;
}

impl Default for BufferUsage {
    #[inline] fn default() -> Self { BufferUsage::DynamicDraw }
}


use ::*;

use std::any::Any;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, RangeBounds, Bound};
use std::slice::from_raw_parts;
use std::rc::Rc;
use std::alloc::{Alloc, Global};
use std::ptr::{drop_in_place, NonNull};
use std::mem::*;

use trait_arith::{Boolean, True, False};


pub unsafe trait GPUCopy {}
unsafe impl<T:Copy> GPUCopy for T {}
unsafe impl<T:Copy> GPUCopy for [T] {}

macro_rules! impl_tuple_gpucopy {
    ({$($T:ident:$t:ident)*} $Last:ident:$l:ident) => {
        unsafe impl<$($T:Copy,)* $Last: Copy> GPUCopy for ($($T,)* [$Last]) {}
    };
}
impl_tuple!(impl_tuple_gpucopy @with_last);


union Repr<T:?Sized> {
    void: *const GLvoid,
    void_mut: *mut GLvoid,
    rust: *const T,
    rust_mut: *mut T,
    bytes: [usize; 2] //so that all Repr's are 16 bytes long
}

pub struct Buffer<T:?Sized, A:BufferAccess> {
    id: GLuint,
    repr: Repr<T>,
    is_ref: bool,

    offset: GLsizeiptr,
    size: GLsizeiptr,
    capacity: GLsizeiptr,

    usage: BufferUsage,
    p: PhantomData<A>
}

impl BufferTarget {
    unsafe fn bind<T:?Sized, A:BufferAccess>(self, buf: &Buffer<T, A>) { gl::BindBuffer(self as GLenum, buf.id); }
    unsafe fn unbind(self) { gl::BindBuffer(self as GLenum, 0); }
}

impl IndexedBufferTarget {
    pub(crate) unsafe fn bind_range<L:BlockLayout, T:?Sized+Layout<L>, A:BufferAccess>(self, buf: &Buffer<T, A>, binding: GLuint) {
        gl::BindBufferRange(self as GLenum, binding, buf.id, buf.offset, buf.size);
    }
    pub(crate) unsafe fn unbind(self, binding: GLuint) {
        gl::BindBufferBase(self as GLenum, binding, 0);
    }
}

impl<T:?Sized, A:BufferAccess> Buffer<T, A> {

    unsafe fn gen() -> Self {
        let mut id: GLuint = 0;
        gl::GenBuffers(1, &mut id as *mut GLuint);
        Buffer {
            id: id,
            repr: uninitialized(),
            is_ref: false,
            offset: 0,
            size: 0,
            capacity: 0,
            usage: uninitialized(),
            p: PhantomData
        }
    }

    unsafe fn buffer_storage(&mut self, usage: BufferUsage, size: usize, data: *const GLvoid) {
        let target = BufferTarget::CopyWriteBuffer;
        target.bind(self);

        self.size = size as GLsizeiptr;
        self.capacity = self.size;
        self.usage = A::buffer_usage(usage);

        if gl::BufferStorage::is_loaded() {
            gl::BufferStorage(target as GLenum, self.size, data, A::storage_flags(usage));
        } else {
            gl::BufferData(target as GLenum, self.size, data, self.usage as GLenum);
        }

        target.unbind();
    }

    unsafe fn buffer_sub_data(&mut self, size: usize, data: *const GLvoid) {
        let target = BufferTarget::CopyWriteBuffer;
        target.bind(self);
        if self.is_ref {
            gl::BufferSubData(target as GLenum, self.offset, size as GLsizeiptr, data);
        } else {
            self.offset = 0;
            self.size = size as GLsizeiptr;
            self.capacity = self.size;
            gl::BufferData(target as GLenum, self.size, data, self.usage as GLenum);
        }
        target.unbind();
    }

    unsafe fn read_buffer(&self, data: *mut GLvoid) {
        //if we have map, then we can simply readonly-map the buffer to a pointer
        //and perform mem-copy on the bytes, which, depending on the implementation, can be rather fast.
        //However, if we don't have map, we unfortunately need to use glGetBufferSubData
        if gl::MapBufferRange::is_loaded() {
            let map = self._map::<Read>(0);

            let src: *const u8 = transmute(Repr{rust:map.deref()}.void);
            let ptr: *mut u8 = transmute(data);
            ::std::ptr::copy(src, ptr, self.data_size());
        } else {
            let target = BufferTarget::CopyReadBuffer;
            target.bind(self);
            gl::GetBufferSubData(target as GLenum, self.offset, self.size, data);
            target.unbind();
        }
    }

    unsafe fn copy_data(&self, dest: &mut Self) {
        let read_target = BufferTarget::CopyReadBuffer;
        let write_target = BufferTarget::CopyWriteBuffer;

        read_target.bind(self);
        write_target.bind(dest);

        gl::CopyBufferSubData(read_target as GLenum, write_target as GLenum, self.offset, dest.offset, self.size);

        read_target.unbind();
        write_target.unbind();
    }

    unsafe fn allocate(size: usize, hint: BufferUsage) -> Self {
        let mut buf = Self::gen();
        buf.buffer_storage(hint, size, ::std::ptr::null());
        buf
    }

    unsafe fn _map<'b, B:BufferAccess>(&'b self, extra_flags: GLbitfield) -> BMap<'b, T, B> {
        let target = BufferTarget::CopyWriteBuffer;
        target.bind(self);

        let mut repr = Repr { bytes: self.repr.bytes };
        repr.void_mut = gl::MapBufferRange(target as GLenum, self.offset, self.size, B::mapping_flags(self.usage) | extra_flags);
        target.unbind();

        BMap {
            buffer: transmute::<&Buffer<T,A>, &Buffer<T,B>>(self),
            data: &mut *repr.rust_mut
        }

    }

    #[inline]
    pub fn forget(b:Buffer<T,A>) {
        if !b.is_ref {
            unsafe {
                gl::DeleteBuffers(1, &b.id);
                forget(b);
            }
        }
    }

    #[inline] pub fn data_offset(&self) -> usize { self.offset as usize }
    #[inline] pub fn data_size(&self) -> usize { self.size as usize }
    #[inline] pub fn buffer_size(&self) -> usize { self.capacity as usize }
    #[inline] pub fn usage_hint(&self) -> BufferUsage { self.usage }

    #[inline] pub fn gl_provider(&self) -> GL1 { GL1::get_current().unwrap() }

    #[inline]
    unsafe fn _from_box(_gl: &GL1, data: Box<T>) -> Self {
        Self::_from_box_with_hint(_gl, BufferUsage::default(), data)
    }

    unsafe fn _from_box_with_hint(_gl: &GL1, hint: BufferUsage, data: Box<T>) -> Self {
        //generate a buffer handle using openGL
        let mut buf = Self::gen();

        //I think I'm going to hell for this....

        //Basically, use the Repr union to extract the RAW raw void pointer from
        //the not-so-raw raw pointer from the box. This is necessary since unsized types tend to
        //add metadata (like length or vtable pointers) to their pointers that make
        //them not able to be transmuted into *const c_void
        buf.repr.bytes = [0,0];
        buf.repr.rust_mut = Box::<T>::into_raw(data);
        let refr = buf.repr.rust;
        let ptr = buf.repr.void_mut;

        //allocate the memory we need and transfer the data
        buf.buffer_storage(hint, size_of_val(&*refr), ptr);

        //now, we need to dealocate the heap storage of the Box WITHOUT running the destructor of the object.
        Global.dealloc(NonNull::new_unchecked(transmute(ptr)), ::std::alloc::Layout::for_value(&*refr));

        //return our newly created buffer
        buf
    }

    #[inline]
    pub fn as_slice(&self) -> BSlice<T, A> {
        BSlice::new(
            Buffer {
                id: self.id,
                repr: unsafe { Repr{bytes: self.repr.bytes} },
                is_ref: true,
                offset: self.offset,
                size: self.size,
                capacity: self.capacity,
                usage: self.usage,
                p: PhantomData
            }
        )
    }

    #[inline]
    pub fn as_slice_mut(&mut self) -> BSliceMut<T, A> {
        BSliceMut::new(
            Buffer {
                id: self.id,
                repr: unsafe { Repr{bytes: self.repr.bytes} },
                is_ref: true,
                offset: self.offset,
                size: self.size,
                capacity: self.capacity,
                usage: self.usage,
                p: PhantomData
            }
        )
    }

}

impl<T:Sized, A:BufferAccess> Buffer<T, A> {

    #[inline]
    pub unsafe fn uninitialized(_gl: &GL1) -> Self {
        Self::uninitialized_with_hint(_gl, BufferUsage::default())
    }

    #[inline]
    pub unsafe fn uninitialized_with_hint(_gl: &GL1, hint: BufferUsage) -> Self {
        Self::allocate(size_of::<T>(), hint)
    }

    #[inline]
    unsafe fn _new(_gl: &GL1, data: T) -> Self {
        Self::_with_hint(_gl, BufferUsage::default(), data)
    }

    unsafe fn _with_hint(_gl: &GL1, hint: BufferUsage, data: T) -> Self {
        //gen our buffer handle and stuff
        let mut buf = Self::gen();

        //for sized types, a reference is just a fancy void pointer, so we can just transmute a reference
        //to get the GLvoid we need
        buf.repr.rust = &data as *const T;
        let ptr = buf.repr.void;
        buf.buffer_storage(hint, size_of::<T>(), ptr);

        //make sure the destructor does not run
        forget(data);

        buf
    }
}

impl<T:Sized, A:BufferAccess> Buffer<[T], A> {
    #[inline]
    pub unsafe fn uninitialized(_gl: &GL1, count: usize) -> Self {
        Self::uninitialized_with_hint(_gl, BufferUsage::default(), count)
    }

    #[inline]
    pub unsafe fn uninitialized_with_hint(_gl: &GL1, hint: BufferUsage, count: usize) -> Self {
        let mut buf = Self::allocate(size_of::<T>() * count, hint);
        buf.repr.bytes[1] = count;
        buf
    }
}



//so... the read+write access is necessary since T *might* implement drop, and
//in order to drop T from a buffer, we need to to have a mutable map.
//hence, we don't let anyone create an immutable or unreadable buffer with a non-GPUCopy type

impl<T:?Sized, A:ReadAccess+WriteAccess> Buffer<T,A> {
    #[inline] pub fn from_box(gl: &GL1, data: Box<T>) -> Self {unsafe { Self::_from_box(gl, data) } }
    #[inline] pub fn from_box_with_hint(gl: &GL1, hint: BufferUsage, data: Box<T>) -> Self {
        unsafe { Self::_from_box_with_hint(gl, hint, data) }
    }
}

impl<T:Sized, A:ReadAccess+WriteAccess> Buffer<T, A> {
    #[inline] pub fn new(gl: &GL1, data: T) -> Self { unsafe { Self::_new(gl, data) } }
    #[inline] pub fn with_hint(gl: &GL1, hint: BufferUsage, data: T) -> Self {
        unsafe { Self::_with_hint(gl, hint, data) }
    }
}

impl<T:GPUCopy+?Sized> Buffer<T,CopyOnly> {
    #[inline] pub fn immut_from(gl: &GL1, data: Box<T>) -> Self { unsafe { Self::_from_box(gl, data) } }
    #[inline] pub fn immut_from_with_hint(gl: &GL1, hint: BufferUsage, data: Box<T>) -> Self {
        unsafe { Self::_from_box_with_hint(gl, hint, data)}
    }
}

impl<T:GPUCopy+Sized> Buffer<T,CopyOnly> {
    #[inline] pub fn new_immut(gl: &GL1, data: T) -> Self { unsafe { Self::_new(gl, data) } }
    #[inline] pub fn immut_with_hint(gl: &GL1, hint: BufferUsage, data: T) -> Self {
        unsafe { Self::_with_hint(gl, hint, data) }
    }
}

impl<T:GPUCopy+?Sized> Buffer<T,Read> {
    #[inline] pub fn readonly_from(gl: &GL1, data: Box<T>) -> Self { unsafe { Self::_from_box(gl, data) } }
    #[inline] pub fn readonly_from_with_hint(gl: &GL1, hint: BufferUsage, data: Box<T>) -> Self {
        unsafe { Self::_from_box_with_hint(gl, hint, data)}
    }
}

impl<T:GPUCopy+Sized> Buffer<T,Read> {
    #[inline] pub fn new_readonly(gl: &GL1, data: T) -> Self { unsafe { Self::_new(gl, data) } }
    #[inline] pub fn readonly_with_hint(gl: &GL1, hint: BufferUsage, data: T) -> Self {
        unsafe { Self::_with_hint(gl, hint, data) }
    }
}

impl<T:GPUCopy+?Sized> Buffer<T,Write> {
    #[inline] pub fn writeonly_from(gl: &GL1, data: Box<T>) -> Self { unsafe { Self::_from_box(gl, data) } }
    #[inline] pub fn writeonly_from_with_hint(gl: &GL1, hint: BufferUsage, data: Box<T>) -> Self {
        unsafe { Self::_from_box_with_hint(gl, hint, data) }
    }
}

impl<T:GPUCopy+Sized> Buffer<T,Write> {
    #[inline] pub fn new_writeonly(gl: &GL1, data: T) -> Self { unsafe { Self::_new(gl, data) } }
    #[inline] pub fn writeonly_with_hint(gl: &GL1, hint: BufferUsage, data: T) -> Self {
        unsafe { Self::_with_hint(gl, hint, data) }
    }
}

//
//For getting the contents of a buffer, we have both unwrapping methods
//(that get the contents for potentially non-copy types and consume the buffer),
//
//

impl<T:Sized, A:ReadAccess> Buffer<T,A> {
    unsafe fn _read(&self) -> T {
        let mut dest = uninitialized::<T>();
        self.read_buffer(transmute::<*mut T, *mut GLvoid>(&mut dest as *mut T));
        dest
    }

    pub fn into_inner(self) -> T {
        unsafe {
            let data = self._read();
            Buffer::forget(self);
            data
        }
    }
}
impl<T:GPUCopy+Sized, A:ReadAccess> Buffer<T,A> { #[inline] pub fn read(&self) -> T {unsafe{self._read()}} }

impl<T:Sized, A:ReadAccess> Buffer<[T],A> {
    unsafe fn _into_box(&self) -> Box<[T]> {
        if self.len()==0 {return Box::new([]);}

        //allocate the space we need (we *could* use vec![0;self.data_size()])
        //but we don't want to waste time on initialization)
        let mut dest = Vec::<T>::with_capacity(self.len());
        dest.set_len(self.len());

        //read the bytes into the vec
        self.read_buffer(transmute::<*mut T, *mut GLvoid>(&mut dest[0] as *mut T));
        dest.into_boxed_slice()
    }

    pub fn into_box(self) -> Box<[T]> {
        unsafe {
            let data = self._into_box();
            Buffer::forget(self);
            data
        }
    }

}
impl<T:GPUCopy+Sized, A:ReadAccess> Buffer<[T],A> { #[inline] pub fn read_into_box(&self) -> Box<[T]> {unsafe{self._into_box()}} }


impl<T:?Sized, A:BufferAccess> Drop for Buffer<T, A> {
    fn drop(&mut self) {
        if !self.is_ref {

            //They can't say you can't specialize Drop if you outsource it to another trait
            //*insert Roll Safe meme here*

            trait SpecificDrop { unsafe fn specific_drop(&mut self); }
            impl<T:?Sized, A:BufferAccess> SpecificDrop for Buffer<T, A> {
                #[inline] default unsafe fn specific_drop(&mut self) { drop_in_place(&mut *self._map::<ReadWrite>(0)) }
            }
            impl<T:GPUCopy+?Sized, A:BufferAccess> SpecificDrop for Buffer<T, A> { #[inline] unsafe fn specific_drop(&mut self) {} }

            unsafe {
                self.specific_drop();
                gl::DeleteBuffers(1, &self.id);
            }
        }
    }
}

//
//We only want to allow copying of types that are themselves Copy
//or of arrays of Copy objects
//

impl<T:GPUCopy + ?Sized, A:BufferAccess> Buffer<T,A> {
    #[inline] pub fn try_copy_from(&mut self, src: &Self) -> Result<(), GLError> { src.try_copy_to(self) }
    pub fn try_copy_to(&self, dest: &mut Self) -> Result<(), GLError> {
        //If we're dealing with an unsized type, we need to make sure the destination
        //capacity is larger than the source data size. For arrays, the obviously means
        //we want enough space for each element longer length, but for trait objects,
        //we just want to make sure there we can fit the object
        if dest.capacity >= self.size {
            unsafe {
                //we need to copy the metadata else the mapped reference metadata
                //may be wrong for unsized types (for sized types tho, we're fine anyway)
                dest.repr.bytes = self.repr.bytes;
                dest.size = self.size;
                Ok(self.copy_data(dest))
            }
        } else {
            Err(GLError::BufferCopySizeError(self.data_size(), dest.buffer_size()))
        }
    }
}

impl<T:GPUCopy + Sized, A:BufferAccess> Buffer<T,A> {
    //since we have a sized type, there no need for error checking
    //or resizing offsets when we copy
    #[inline] pub fn copy_to(&self, dest: &mut Self) { unsafe { self.copy_data(dest); } }
    #[inline] pub fn copy_from(&mut self, src: &Self) { src.copy_to(self) }
}

//
//Clone works using the copy methods, and thus, we can only clone buffers that can be copied.
//

impl<T:GPUCopy + ?Sized, A:BufferAccess> Clone for Buffer<T, A> {
    fn clone(&self) -> Self {
        unsafe {
            let mut buf = Self::allocate(self.data_size(), self.usage_hint());
            self.copy_data(&mut buf);
            buf
        }
    }
}

//
//For cases where we only need to write data, we can invalidate the buffer's data when mapping
//in order to avoid implicit synchronization
//

//the version for sized data
impl<T:GPUCopy + Sized, A:WriteAccess> Buffer<T, A> {
    pub fn update_data(&mut self, data: T) {
        unsafe {
            if gl::MapBufferRange::is_loaded() {
                self._map::<Write>(gl::MAP_INVALIDATE_RANGE_BIT).write(data);
            } else {
                self.buffer_sub_data(size_of::<T>(), transmute::<_, *const GLvoid>(&data as *const T));
            }
        }
    }
}

//the version for arrays of sized data
impl<T:Copy + Sized, A:WriteAccess> Buffer<[T], A> {
    pub fn update_data<I:Iterator<Item=T>>(&mut self, mut data: I) {
        unsafe {
            if gl::MapBufferRange::is_loaded() {
                let (lb, _) = data.size_hint();
                let len = self.len();
                let mut map = {
                    if lb >= len {
                        self._map::<Write>(gl::MAP_INVALIDATE_RANGE_BIT)
                    } else {
                        self.map_write()
                    }
                };

                let mut i = 0;
                while i<len {
                    if let Some(t) = data.next() {
                        map.write_at(i, t);
                        i+=1;
                    } else {
                        break;
                    }
                }
            } else {
                let data_buffer: Vec<T> = data.collect();
                self.buffer_sub_data(size_of::<T>(), transmute::<_, *const GLvoid>(&data_buffer[0] as *const T));
            }

        }
    }
}




//
//MAPPINGS
//

pub struct BMap<'a, T:?Sized, A: BufferAccess> {
    buffer: &'a Buffer<T, A>,
    data: &'a mut T
}

//
//Mapping should only work if we have the proper permissions/access
//

impl<T:?Sized, A:ReadAccess> Buffer<T, A> {
    #[inline] pub fn map(&self) -> BMap<T,Read> { unsafe { self._map(0) } }
}

impl<T:?Sized, A:WriteAccess> Buffer<T, A> {
    #[inline] pub fn map_write(&mut self) -> BMap<T,Write> { unsafe { self._map(0) } }
}

impl<T:?Sized, A:ReadAccess+WriteAccess> Buffer<T, A> {
    #[inline] pub fn map_mut(&mut self) -> BMap<T,ReadWrite> { unsafe { self._map(0) } }
}

impl<'a, T:?Sized, A:ReadAccess> Deref for BMap<'a, T, A> {
    type Target = T;
    #[inline] fn deref(&self) -> &T { self.data }
}

impl<'a, T:Sized, A:WriteAccess> BMap<'a, T, A> {
    #[inline] pub fn write(&mut self, obj: T) { *self.data = obj }
}

impl<'a, T:Copy, A:WriteAccess> BMap<'a, [T], A> {
    #[inline] pub fn write_at(&mut self, i: usize, obj: T) { self.data[i] = obj }
}

impl<'a, T:?Sized, A:ReadAccess+WriteAccess> DerefMut for BMap<'a, T, A> {
    #[inline] fn deref_mut(&mut self) -> &mut T { self.data }
}

impl<'a, T:?Sized, A:BufferAccess> Drop for BMap<'a, T, A> {
    fn drop(&mut self) {
        unsafe {
            let target = BufferTarget::CopyWriteBuffer;
            target.bind(self.buffer);
            let status = gl::UnmapBuffer(target as GLenum);
            target.unbind();
            if status==0 { panic!("Buffer id={} corrupted!", self.buffer.id); }
        }
    }
}

//
//Downcast
//

impl<A:BufferAccess> Buffer<dyn Any, A> {
    pub fn downcast<T:'static>(self) -> Result<Buffer<T, A>, Self> {
        unsafe {
            let refr = &mut *self.repr.rust_mut;
            match refr.downcast_mut::<T>() {
                Some(cast) => {
                    let mut buf = transmute::<_, Buffer<T, A>>(self);
                    buf.repr.rust_mut = cast as *mut T;
                    Ok(buf)
                },
                None => Err(self)
            }
        }
    }
}

impl<'a, A:BufferAccess> BMap<'a, dyn Any, A> {
    pub fn downcast<T:'static>(mut self) -> Result<BMap<'a, T, A>, Self> {
        {
            let refr = &mut self.data;
            let refr2 = refr.downcast_mut::<T>();
            if let Some(cast) = refr2 {
                unsafe {
                    return Ok(BMap{buffer: transmute(self.buffer), data: transmute(cast)});
                }
            }
        }

        Err(self)
    }
}

//
//Splitting up buffers (which is actually super useful)
//

#[derive(Clone)]
pub struct BSlice<T:?Sized, A:BufferAccess> {
    refr: Rc<Buffer<T, A>>,
}
impl<T:?Sized, A:BufferAccess>  BSlice<T, A> {
    #[inline] fn new(b: Buffer<T, A>) -> Self {BSlice{refr: Rc::new(b)}}
}

impl<T:?Sized, A:BufferAccess> Deref for BSlice<T, A> {
    type Target = Buffer<T, A>;
    #[inline] fn deref(&self) -> &Self::Target { &*self.refr }
}

impl<T:?Sized, A:BufferAccess> AsRef<Buffer<T,A>> for BSlice<T, A> {
    #[inline] fn as_ref(&self) -> &Buffer<T,A> { &*self }
}

impl<T:?Sized, A:BufferAccess> From<BSliceMut<T,A>> for BSlice<T, A> {
    #[inline] fn from(mut_slice: BSliceMut<T,A>) -> Self { BSlice { refr: mut_slice.refr.into() } }
}

pub struct BSliceMut<T:?Sized, A:BufferAccess> {
    refr: Box<Buffer<T, A>>,
}
impl<T:?Sized, A:BufferAccess>  BSliceMut<T, A> {
    #[inline] fn new(b: Buffer<T, A>) -> Self {BSliceMut{refr: Box::new(b)}}
}

impl<T:?Sized, A:BufferAccess> Deref for BSliceMut<T, A> {
    type Target = Buffer<T, A>;
    #[inline] fn deref(&self) -> &Self::Target { self.refr.as_ref() }
}

impl<T:?Sized, A:BufferAccess> DerefMut for BSliceMut<T, A> {
    #[inline] fn deref_mut(&mut self) -> &mut Self::Target { self.refr.as_mut() }
}

impl<T:?Sized, A:BufferAccess> AsRef<Buffer<T,A>> for BSliceMut<T, A> {
    #[inline] fn as_ref(&self) -> &Buffer<T,A> { self.refr.as_ref() }
}

impl<T:?Sized, A:BufferAccess> AsMut<Buffer<T,A>> for BSliceMut<T, A> {
    #[inline] fn as_mut(&mut self) -> &mut Buffer<T,A> { self.refr.as_mut() }
}

impl<T:Sized, A:BufferAccess> Buffer<[T], A> {

    fn check_bounds(&self, i: usize) -> (GLsizeiptr, GLsizeiptr) {
        let unit = size_of::<T>() as GLsizeiptr;
        let offset = self.offset + unit * (i as GLsizeiptr);
        if offset >= self.size {
            panic!("Index out of bounds: {} >= {}", i, self.size / unit);
        } else {
            (offset, unit)
        }
    }

    fn check_bounds_incl(&self, i: usize) -> (GLsizeiptr, GLsizeiptr) {
        let unit = size_of::<T>() as GLsizeiptr;
        let offset = self.offset + unit * (i as GLsizeiptr);
        if offset > self.size {
            panic!("Index out of bounds: {} >= {}", i, self.size / unit);
        } else {
            (offset, unit)
        }
    }

    pub fn len(&self) -> usize { self.data_size() / size_of::<T>() }
    pub fn is_empty(&self) -> bool { self.size == 0 }

    fn _slice<R:RangeBounds<usize>>(&self, r:R) -> Buffer<[T], A> {

        let start = match r.start_bound() {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => *i+1,
            Bound::Unbounded => 0
        };

        let end = match r.end_bound() {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => if *i==0 {0} else {*i-1},
            Bound::Unbounded => self.len()
        };

        let (start_offset, _) = self.check_bounds(start);
        let (end_offset, _) = self.check_bounds_incl(end);

        let size = end - start;
        let byte_size = end_offset - start_offset;

        unsafe {
            Buffer {
                id: self.id,
                repr: Repr{ rust: from_raw_parts(transmute(self.repr.void), size).as_ref() as *const [T] },
                is_ref: true,

                offset: start_offset,
                size: byte_size,
                capacity: byte_size,

                usage: self.usage,
                p: PhantomData
            }
        }

    }

    #[inline] pub fn get(&self, i:usize) -> BSlice<T, A> {unsafe {transmute(self.slice(i..i+1))}}
    #[inline] pub fn get_mut(&mut self, i:usize) -> BSliceMut<T, A> {unsafe {transmute(self.slice_mut(i..i+1))} }

    #[inline] pub fn slice<R:RangeBounds<usize>>(&self, r:R) -> BSlice<[T], A> { BSlice::new(self._slice(r)) }
    #[inline] pub fn slice_mut<R:RangeBounds<usize>>(&mut self, r:R) -> BSliceMut<[T], A> {
        BSliceMut::new(self._slice(r))
    }

    #[inline] pub fn split_at<R:RangeBounds<usize>>(&self, mid: usize) -> (BSlice<[T], A>, BSlice<[T], A>) {
        (self.slice(0..mid), self.slice(mid..))
    }
    #[inline] pub fn split_at_mut<R:RangeBounds<usize>>(&mut self, mid: usize) -> (BSliceMut<[T], A>, BSliceMut<[T], A>) {
        (BSliceMut::new(self._slice(0..mid)), BSliceMut::new(self._slice(mid..)))
    }

}

fn align(offset: usize, alignment: usize) -> usize {
    let error = offset % alignment;
    if error != 0 {
        offset - error + alignment
    } else {
        offset
    }
}

macro_rules! impl_tuple_splitting {

    ({$($T:ident:$t:ident)*} $Last:ident:$l:ident) => {
        impl_tuple_splitting!({$($T)*} $Last split_tuple BSlice &);
        impl_tuple_splitting!({$($T)*} $Last split_tuple_mut BSliceMut &mut );
    };

    ({$($T:ident)*} $Last:ident $fun:ident $slice:ident $($r:tt)* ) => {

        impl<$($T:Sized,)* $Last:?Sized, BA:BufferAccess> Buffer<($($T,)* $Last), BA> {

            pub fn $fun($($r)* self) -> ($($slice<$T, BA>,)* $slice<$Last, BA>) {
                unsafe {
                    #[allow(unused_mut)]
                    let mut offset = 0;
                    let last_size = size_of_val::<$Last>(transmute(self.repr.rust)) as GLsizeiptr;

                    (
                        $($slice::new(
                            Buffer{
                                id: self.id,
                                repr: Repr{bytes: [0,0]},
                                is_ref: true,

                                offset: {
                                    *&mut offset = align(offset, align_of::<$T>());
                                    let i = offset;
                                    *&mut offset = i + size_of::<$T>();
                                    i as GLsizeiptr
                                },
                                size: size_of::<$T>() as GLsizeiptr,
                                capacity: size_of::<$T>() as GLsizeiptr,

                                usage: self.usage,
                                p: PhantomData
                            }
                        ),)*
                        $slice::new(
                            Buffer{
                                id: self.id,
                                repr: Repr{bytes: self.repr.bytes},
                                is_ref: true,

                                offset: align(offset, align_of_val::<$Last>(transmute(self.repr.rust))) as GLsizeiptr,
                                size: last_size,
                                capacity: last_size,

                                usage: self.usage,
                                p: PhantomData
                            }
                        )
                    )
                }
            }
        }
    }


}

impl_tuple!(impl_tuple_splitting @with_last);

#[derive(Clone, Copy)]
pub struct AttribArray<'a, A:GLSLType> {
    buf: &'a Buffer<[u8], CopyOnly>,
    len: usize,
    format: A::AttributeFormat,
    stride: usize,
    offset: usize,
    p: PhantomData<A>
}

impl<T:Copy, A:BufferAccess> Buffer<[T],A> {

    #[inline]
    pub unsafe fn get_attrib_array<'a, G:GLSLType, U:AttributeData<G>>(&'a self, offset:usize) -> AttribArray<'a, G> {
        debug_assert_eq!(U::format().size(), size_of::<U>(), "Invalid value size for given attribute format!");
        debug_assert!(offset+size_of::<U>() <= size_of::<T>(), "Attribute offset too large!");

        AttribArray {
            buf: transmute::<&Buffer<[T],A>,&Buffer<[u8],CopyOnly>>(self),
            len: self.len(),
            format: U::format(),
            stride: size_of::<T>(),
            offset: self.offset as usize + offset,
            p: PhantomData
        }
    }

    #[inline]
    pub fn as_attrib_array<'a, G:GLSLType>(&'a self) -> AttribArray<'a, G> where T:AttributeData<G> {
        unsafe { self.get_attrib_array::<G, T>(0) }
    }

    #[inline]
    pub fn as_attribute<'a, G:GLSLType>(&'a self) -> Attribute<'a, G> where T:AttributeData<G> {
        Attribute::Array(self.as_attrib_array())
    }

}

macro_rules! impl_as_attrib_arrays {

    ($($T:ident:$t:ident)*) => {impl_as_attrib_arrays!(@gen {} $($T:$t)*);};
    (@gen {$($prev:tt)*} A:$t:ident $($rest:tt)*) => {impl_as_attrib_arrays!(@gen {$($prev)* A:A1:$t} $($rest)*);};
    (@gen {$($prev:tt)*} B:$t:ident $($rest:tt)*) => {impl_as_attrib_arrays!(@gen {$($prev)* B:A2:$t} $($rest)*);};
    (@gen {$($prev:tt)*} C:$t:ident $($rest:tt)*) => {impl_as_attrib_arrays!(@gen {$($prev)* C:A3:$t} $($rest)*);};
    (@gen {$($prev:tt)*} D:$t:ident $($rest:tt)*) => {impl_as_attrib_arrays!(@gen {$($prev)* D:A4:$t} $($rest)*);};
    (@gen {$($prev:tt)*} E:$t:ident $($rest:tt)*) => {impl_as_attrib_arrays!(@gen {$($prev)* E:A5:$t} $($rest)*);};
    (@gen {$($prev:tt)*} F:$t:ident $($rest:tt)*) => {impl_as_attrib_arrays!(@gen {$($prev)* F:A6:$t} $($rest)*);};
    (@gen {$($prev:tt)*} G:$t:ident $($rest:tt)*) => {impl_as_attrib_arrays!(@gen {$($prev)* G:A7:$t} $($rest)*);};
    (@gen {$($prev:tt)*} H:$t:ident $($rest:tt)*) => {impl_as_attrib_arrays!(@gen {$($prev)* H:A8:$t} $($rest)*);};
    (@gen {$($prev:tt)*} I:$t:ident $($rest:tt)*) => {impl_as_attrib_arrays!(@gen {$($prev)* I:A9:$t} $($rest)*);};
    (@gen {$($prev:tt)*} J:$t:ident $($rest:tt)*) => {impl_as_attrib_arrays!(@gen {$($prev)* J:Aa:$t} $($rest)*);};
    (@gen {$($prev:tt)*} K:$t:ident $($rest:tt)*) => {impl_as_attrib_arrays!(@gen {$($prev)* K:Ab:$t} $($rest)*);};
    (@gen {$($prev:tt)*} L:$t:ident $($rest:tt)*) => {impl_as_attrib_arrays!(@gen {$($prev)* L:Ac:$t} $($rest)*);};
    (@gen {$($prev:tt)*} ) => {impl_as_attrib_arrays!($($prev)*);};

    (@first $t0:ident $($t:ident)*) => {$t0};

    ($($T:ident:$A:ident:$t:ident)*) => {

        impl<$($T:Copy,)* Access:BufferAccess> Buffer<[($($T),*)], Access> {

            #[inline]
            pub fn as_attrib_arrays<'a, $($A:GLSLType),*>(&'a self) -> ($(AttribArray<'a, $A>),*) where $($T: AttributeData<$A>),*{
                unsafe {
                    let ($($t),*) = ::std::mem::uninitialized::<($($T),*)>();
                    let first: *const u8 = transmute(&impl_as_attrib_arrays!(@first $($t)*));

                    let arrays = (
                        $(self.get_attrib_array::<$A,$T>((transmute::<&$T, *const u8>(&$t).offset_from(first)) as usize)),*
                    );
                    $(forget($t);)*

                    arrays
                }
            }

            #[inline]
            pub fn as_attributes<'a, $($A:GLSLType),*>(&'a self) -> ($(Attribute<'a, $A>),*) where $($T: AttributeData<$A>),* {
                let ($($t),*) = self.as_attrib_arrays();
                ($(Attribute::Array($t)),*)
            }

        }
    };
}

impl_tuple!(impl_as_attrib_arrays);


impl<'a, A:GLSLType> AttribArray<'a,A> {

    #[inline] pub unsafe fn bind(&self) { BufferTarget::ArrayBuffer.bind(&self.buf); }
    #[inline] pub unsafe fn unbind() { BufferTarget::ArrayBuffer.unbind() }

    #[inline] pub fn len(&self) -> usize { self.len }
    #[inline] pub fn format(&self) -> A::AttributeFormat { self.format }
    #[inline] pub fn stride(&self) -> usize { self.stride }
    #[inline] pub fn offset(&self) -> usize { self.offset }

}
