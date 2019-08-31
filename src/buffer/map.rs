
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

//
//MapBuffer
//

fn map_access<B:BufferAccess>() -> GLenum {
    match (<B::Read as Boolean>::VALUE, <B::Write as Boolean>::VALUE) {
        (true, false) => gl::READ_ONLY,
        (false, true) => gl::WRITE_ONLY,
        (true, true) => gl::READ_WRITE,
        (false, false) => panic!("Invalid map flags"),
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

        if gl::MapBufferRange::is_loaded() || gl::MapNamedBufferRange::is_loaded() {

            let mut flags = 0;
            if <B::Read as Boolean>::VALUE {flags |= gl::MAP_READ_BIT;}
            if <B::Write as Boolean>::VALUE {flags |= gl::MAP_WRITE_BIT;}

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

//TODO persistent mapping
