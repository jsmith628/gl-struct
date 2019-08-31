
use super::*;
use crate::gl;

use std::slice::SliceIndex;
use std::ops::{Deref, DerefMut, CoerceUnsized};

pub struct BMap<'a, T:?Sized, A:BufferAccess> {
    ptr: *mut T,
    offset: usize,
    id: GLuint,
    buf: PhantomData<&'a mut Buffer<T,A>>
}

impl<'a,U:?Sized,T:?Sized+Unsize<U>,A:BufferAccess> CoerceUnsized<BMap<'a,U,A>> for BMap<'a,T,A> {}

impl<'a,T:?Sized,A:BufferAccess> !Sync for BMap<'a,T,A> {}
impl<'a,T:?Sized,A:BufferAccess> !Send for BMap<'a,T,A> {}

impl<'a,T:?Sized,A:BufferAccess> Drop for BMap<'a,T,A> {
    fn drop(&mut self) {
        unsafe {
            let status;
            let mut target = BufferTarget::CopyWriteBuffer.as_loc();

            if !<A::Persistent as Boolean>::VALUE {
                //if the map is not persistent, we need to fully unmap the buffer
                if gl::UnmapNamedBuffer::is_loaded() {
                    status = gl::UnmapNamedBuffer(self.id);
                } else {
                    status = gl::UnmapBuffer(target.bind_raw(self.id).unwrap().target_id());
                }
            } else {
                //else, we need to flush any writes that happened in this range
                status = 1;
                if <A::Write as Boolean>::VALUE {
                    if gl::FlushMappedNamedBufferRange::is_loaded() {
                        gl::FlushMappedNamedBufferRange(
                            target.bind_raw(self.id).unwrap().target_id(),
                            self.offset as GLsizeiptr,
                            size_of_val(&*self.ptr) as GLsizeiptr
                        );
                    } else {
                        gl::FlushMappedBufferRange(
                            target.bind_raw(self.id).unwrap().target_id(),
                            self.offset as GLsizeiptr,
                            size_of_val(&*self.ptr) as GLsizeiptr
                        );
                    }
                }
            }

            //panic if the buffer was corrupted.
            //Normally, this shouldn't happen since the rust memory rules should prevent us from doing
            //anything that would cause this to happen, but obviously bugs happen and unsafe code
            //could cause this to happen
            if status==0 { panic!("Buffer id={} corrupted!", self.id); }
        }
    }
}

impl<'a,T:?Sized,A:BufferAccess> BMap<'a,T,A> {
    pub fn as_ptr(this: &Self) -> *const T { this.ptr }
    pub fn as_mut_ptr(this: &mut Self) -> *mut T { this.ptr }

    pub fn id(this: &Self) -> GLuint { this.id }
    pub fn size(this: &Self) -> usize { unsafe {size_of_val(&*this.ptr)} }
    pub fn align(this: &Self) -> usize { unsafe {align_of_val(&*this.ptr)} }
    pub fn offset(this: &Self) -> usize { this.offset }
}

impl<'a,T:?Sized,A:ReadAccess> Deref for BMap<'a,T,A> {
    type Target = T;
    #[inline] fn deref(&self) -> &T { unsafe{&*self.ptr} }
}

impl<'a,T:?Sized,A:ReadAccess+WriteAccess> DerefMut for BMap<'a,T,A> {
    #[inline] fn deref_mut(&mut self) -> &mut T { unsafe{&mut *self.ptr} }
}

impl<'a,T:Sized,A:WriteAccess> BMap<'a,T,A> {
    #[inline] pub unsafe fn write_direct(&mut self, data:T) { copy_nonoverlapping(&data, self.ptr, 1) }
    #[inline] pub fn write(&mut self, data:T) where T:GPUCopy { unsafe{*self.ptr = data;} }
}

impl<'a,T:Sized,A:WriteAccess> BMap<'a,[T],A> {

    #[inline]
    pub unsafe fn write_direct_at(&mut self, i:usize, data:&[T]) {
        assert!(i+data.len()<(&*self.ptr).len(), "attempted to write out-of-bounds");
        copy_nonoverlapping(data.as_ptr(), &mut (*self.ptr)[i], data.len())
    }

    #[inline]
    pub fn write_at<U:Sized,I:SliceIndex<[T],Output=U>>(&mut self, i:I, data:U) where T:GPUCopy {
        unsafe { (*self.ptr)[i] = data; }
    }
}

unsafe fn check_allignment(map:*const GLvoid, align: usize) {
    assert_eq!((map as usize) % align, 0, "invalid map alignment for type");
}

//
//MapBuffer
//

unsafe fn map_access<B:BufferAccess>() -> GLenum {
    match (<B::Read as Boolean>::VALUE, <B::Write as Boolean>::VALUE) {
        (true, false) => gl::READ_ONLY,
        (false, true) => gl::WRITE_ONLY,
        (true, true) => gl::READ_WRITE,
        (false, false) => 0,
    }
}

impl<T:?Sized, A:BufferAccess> Buffer<T,A> {
    unsafe fn map_raw<'a,B:BufferAccess>(&'a mut self) -> BMap<'a,T,B> {
        let mut ptr = BufPtr { rust_mut: self.ptr };

        if gl::MapNamedBuffer::is_loaded() {
            ptr.gl_mut = gl::MapNamedBuffer(self.id(), map_access::<B>());
        } else {
            let mut target = BufferTarget::CopyWriteBuffer.as_loc();
            ptr.gl_mut = gl::MapBuffer(target.bind_buf(self).target_id(), map_access::<B>());
        }

        check_allignment(ptr.gl, self.align());

        BMap {
            ptr: ptr.rust_mut,
            id: self.id(),
            offset: 0,
            buf: PhantomData
        }
    }
}

//Note: we require a mutable reference because otherwise, we could read from the buffer GPU side
//(ie. use this buffer to render something) while mapped, which is dissallowed for everything but
//persistence mapping

unsafe fn map_range_flags<B:BufferAccess>() -> GLbitfield {
    let mut flags = 0;
    if <B::Read as Boolean>::VALUE {flags |= gl::MAP_READ_BIT;}
    if <B::Write as Boolean>::VALUE {flags |= gl::MAP_WRITE_BIT;}
    if <B::Persistent as Boolean>::VALUE {flags |= gl::MAP_PERSISTENT_BIT;}
    flags
}

impl<T:?Sized, A:ReadAccess+NonPersistentAccess> Buffer<T,A> {
    #[inline] pub fn map<'a>(&'a mut self) -> BMap<'a,T,Read> { unsafe{self.map_raw()} }
}

impl<T:?Sized, A:WriteAccess+NonPersistentAccess> Buffer<T,A> {
    #[inline] pub fn map_write<'a>(&'a mut self) -> BMap<'a,T,Write> { unsafe{self.map_raw()} }
}

impl<T:?Sized, A:ReadAccess+WriteAccess+NonPersistentAccess> Buffer<T,A> {
    #[inline] pub fn map_mut<'a>(&'a mut self) -> BMap<'a,T,ReadWrite> { unsafe{self.map_raw()} }
}

//
//MapBufferRange
//

//Note: we cannot simply implement a public map_range method on BSlice or BSliceMut, as then, we could
//split the buffer and then map it multiple times, which is not allowed, even for persistent mapping.
//However, for transparency of the base GL api, we can have a raw one
//Also, we only use mutable references here for the same reason as above

impl<'a,T:?Sized,A:BufferAccess> BSliceMut<'a,T,A> {
    unsafe fn map_range_raw<'b,B:BufferAccess>(self) -> BMap<'b,T,B> {
        let mut target = BufferTarget::CopyWriteBuffer.as_loc();
        let mut ptr = BufPtr { rust_mut: self.ptr };

        if <B::Persistent as Boolean>::VALUE || gl::MapBufferRange::is_loaded() || gl::MapNamedBufferRange::is_loaded() {

            let flags = map_range_flags::<B>();
            if gl::MapNamedBufferRange::is_loaded() {
                ptr.gl_mut = gl::MapNamedBufferRange(
                    self.id(), self.offset() as GLintptr, self.size() as GLsizeiptr, flags
                );
            } else {
                ptr.gl_mut = gl::MapBufferRange(
                    target.bind_slice_mut(&self).target_id(),
                    self.offset() as GLintptr, self.size() as GLsizeiptr, flags
                );
            }

        } else {

            ptr.gl_mut = {
                if gl::MapNamedBuffer::is_loaded() {
                    gl::MapNamedBuffer(self.id(), map_access::<B>())
                } else {
                    gl::MapBuffer(target.bind_slice_mut(&self).target_id(), map_access::<B>())
                }
            }.offset(self.offset() as isize);

        }

        check_allignment(ptr.gl, self.align());

        BMap {
            ptr: &mut *ptr.rust_mut,
            id: self.id(),
            offset: self.offset(),
            buf: PhantomData
        }
    }
}

impl<T:Sized,A:ReadAccess> Buffer<[T],A> {
    #[inline]
    pub fn map_range<'a,U:?Sized,I:SliceIndex<[T],Output=U>>(&'a mut self, i:I) -> BMap<'a,U,Read> {
        unsafe { self.as_slice_mut().index_mut(i).map_range_raw() }
    }
}

impl<T:Sized,A:WriteAccess> Buffer<[T],A> {
    #[inline]
    pub fn map_range_write<'a,U:?Sized,I:SliceIndex<[T],Output=U>>(&'a mut self, i:I) -> BMap<'a,U,Write> {
        unsafe { self.as_slice_mut().index_mut(i).map_range_raw() }
    }
}

impl<T:Sized,A:ReadAccess+WriteAccess> Buffer<[T],A> {
    #[inline]
    pub fn map_range_mut<'a,U:?Sized,I:SliceIndex<[T],Output=U>>(&'a mut self, i:I) -> BMap<'a,U,ReadWrite> {
        unsafe { self.as_slice_mut().index_mut(i).map_range_raw() }
    }
}

//
//Persistent mapping
//

impl<'a,T:?Sized,A:BufferAccess> BSlice<'a,T,A> {
    unsafe fn get_map_pointer_raw<'b,B:BufferAccess>(this:*const Self) -> BMap<'b,T,B> {
        let mut ptr = MaybeUninit::uninit();

        if gl::GetNamedBufferPointerv::is_loaded() {
            gl::GetNamedBufferPointerv((&*this).id(), gl::BUFFER_MAP_POINTER, ptr.as_mut_ptr());
        } else {
            let mut target = BufferTarget::CopyReadBuffer.as_loc();
            gl::GetBufferPointerv(target.bind_slice(&*this).target_id(), gl::BUFFER_MAP_POINTER, ptr.as_mut_ptr());
        }

        if ptr.get_ref().is_null() {
            let mut buf_size = MaybeUninit::uninit();
            let flags = map_range_flags::<A>() | gl::MAP_UNSYNCHRONIZED_BIT; //needs to be the A flags because this is for persistent maps

            if gl::GetNamedBufferParameteriv::is_loaded() && gl::MapNamedBufferRange::is_loaded() {
                gl::GetNamedBufferParameteriv((&*this).id(), gl::BUFFER_SIZE, buf_size.as_mut_ptr());
                *ptr.get_mut() = gl::MapNamedBufferRange(
                    (&*this).id(), 0, buf_size.assume_init() as GLsizeiptr, flags
                );
            } else {
                let mut target = BufferTarget::CopyReadBuffer.as_loc();
                let binding = target.bind_slice(&*this);
                gl::GetBufferParameteriv(binding.target_id(), gl::BUFFER_SIZE, buf_size.as_mut_ptr());
                *ptr.get_mut() = gl::MapBufferRange(
                    binding.target_id(), 0, buf_size.assume_init() as GLsizeiptr, flags
                );
            }

            check_allignment(*ptr.as_ptr(), (&*this).align());

        }

        if <B::Read as Boolean>::VALUE {gl::MemoryBarrier(gl::CLIENT_MAPPED_BUFFER_BARRIER_BIT);}

        BMap {
            ptr: BufPtr{gl_mut: ptr.assume_init()}.rust_mut,
            id: (&*this).id(),
            offset: (&*this).offset(),
            buf: PhantomData
        }

    }
}

impl<'a,T:?Sized,A:ReadAccess+PersistentAccess> BSlice<'a,T,A> {
    #[inline] pub fn get_map_pointer(&self) -> BMap<T,PersistentRead> {
        unsafe {Self::get_map_pointer_raw(self)}
    }
}

impl<'a,T:?Sized,A:ReadAccess+PersistentAccess> BSliceMut<'a,T,A> {
    #[inline] pub fn get_map_pointer(&self) -> BMap<T,PersistentRead> {
        unsafe {BSlice::get_map_pointer_raw(&self.as_immut())}
    }
}

impl<'a,T:?Sized,A:WriteAccess+PersistentAccess> BSliceMut<'a,T,A> {
    #[inline] pub fn get_map_pointer_write(&mut self) -> BMap<T,PersistentWrite> {
        unsafe {BSlice::get_map_pointer_raw(&self.as_immut())}
    }
}

impl<'a,T:?Sized,A:WriteAccess+ReadAccess+PersistentAccess> BSliceMut<'a,T,A> {
    #[inline] pub fn get_map_pointer_mut(&mut self) -> BMap<T,PersistentReadWrite> {
        unsafe {BSlice::get_map_pointer_raw(&self.as_immut())}
    }
}
