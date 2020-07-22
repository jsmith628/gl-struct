
use super::*;
use crate::gl;

use std::slice::SliceIndex;
use std::ops::{Deref, DerefMut, CoerceUnsized};
use std::fmt::{Pointer, Debug, Display, Formatter};
use std::cmp::*;

pub struct Map<'a, T:?Sized, A:Initialized> {
    pub(super) ptr: *mut T,
    pub(super) offset: usize,
    pub(super) id: GLuint,
    pub(super) buf: PhantomData<&'a mut Buffer<T,A>>
}

impl<'a,U:?Sized,T:?Sized+Unsize<U>,A:Initialized> CoerceUnsized<Map<'a,U,A>> for Map<'a,T,A> {}

//
//Formatting traits
//

impl<'a, T:?Sized+Debug, A:ReadMappable> Debug for Map<'a,T,A> {
    fn fmt(&self, f:&mut Formatter) -> ::std::fmt::Result { Debug::fmt(&**self, f) }
}
impl<'a, T:?Sized+Display, A:ReadMappable> Display for Map<'a,T,A> {
    fn fmt(&self, f:&mut Formatter) -> ::std::fmt::Result { Display::fmt(&**self, f) }
}
impl<'a, T:?Sized, A:Initialized> Pointer for Map<'a,T,A> {
    fn fmt(&self, f:&mut Formatter) -> ::std::fmt::Result { Pointer::fmt(&self.ptr, f) }
}

//
//Equality
//

impl<'a,'b,T,U,A,B> PartialEq<Map<'a,U,A>> for Map<'b,T,B> where
    T:?Sized+PartialEq<U>,U:?Sized, A:ReadMappable,B:ReadMappable
{
    fn eq(&self, rhs:&Map<'a,U,A>) -> bool { (&**self).eq(&**rhs) }
    fn ne(&self, rhs:&Map<'a,U,A>) -> bool { (&**self).ne(&**rhs) }
}

impl<'a,'b,T,U,A,B> PartialOrd<Map<'a,U,A>> for Map<'b,T,B> where
    T:?Sized+PartialOrd<U>,U:?Sized,A:ReadMappable,B:ReadMappable
{
    fn partial_cmp(&self, rhs:&Map<'a,U,A>) -> Option<Ordering> { (&**self).partial_cmp(&**rhs) }
}

impl<'a,T:?Sized+Eq,A:ReadMappable> Eq for Map<'a,T,A> {}
impl<'a,T:?Sized+Ord,A:ReadMappable> Ord for Map<'a,T,A> {
    fn cmp(&self, rhs:&Self) -> Ordering { (&**self).cmp(&**rhs) }
}

//
//Map-destruction
//

impl<'a,T:?Sized,A:Initialized> Drop for Map<'a,T,A> {
    fn drop(&mut self) {
        if Self::size(self)==0 { return; }
        unsafe {
            let status;
            let ptr = BufPtr::new(self.id, self.ptr);

            if !<A::MapPersistent as Bit>::VALUE {
                //if the map is not persistent, we need to fully unmap the buffer
                if gl::UnmapNamedBuffer::is_loaded() {
                    status = gl::UnmapNamedBuffer(self.id);
                } else {
                    status = COPY_WRITE_BUFFER.map_bind(&ptr, |b| gl::UnmapBuffer(b.target_id()));
                }
            } else {
                //else, we need to flush any writes that happened in this range
                status = 1;
                if <A::MapWrite as Bit>::VALUE {
                    if gl::FlushMappedNamedBufferRange::is_loaded() {
                        gl::FlushMappedNamedBufferRange(
                            self.id,
                            self.offset as GLsizeiptr,
                            size_of_val(&*self.ptr) as GLsizeiptr
                        );
                    } else {
                        COPY_WRITE_BUFFER.map_bind(&ptr, |b|
                            gl::FlushMappedBufferRange(
                                b.target_id(), self.offset as GLintptr, Self::size(&self) as GLsizeiptr
                            )
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

impl<'a,T:?Sized,A:Initialized> Map<'a,T,A> {
    pub fn id(this: &Self) -> GLuint { this.id }
    pub fn size(this: &Self) -> usize { unsafe {size_of_val(&*this.ptr)} }
    pub fn align(this: &Self) -> usize { unsafe {align_of_val(&*this.ptr)} }
    pub fn offset(this: &Self) -> usize { this.offset }
}

impl<'a,T:?Sized,A:ReadMappable> Deref for Map<'a,T,A> {
    type Target = T;
    #[inline] fn deref(&self) -> &T { unsafe{&*self.ptr} }
}

impl<'a,T:?Sized,A:ReadMappable+WriteMappable> DerefMut for Map<'a,T,A> {
    #[inline] fn deref_mut(&mut self) -> &mut T { unsafe{&mut *self.ptr} }
}

impl<'a,T:Sized,A:WriteMappable> Map<'a,T,A> {
    #[inline] pub unsafe fn write_direct(&mut self, data:T) { copy_nonoverlapping(&data, self.ptr, 1) }
    #[inline] pub fn write(&mut self, data:T) where T:Copy { unsafe{*self.ptr = data;} }
}

impl<'a,T:Sized,A:WriteMappable> Map<'a,[T],A> {

    #[inline]
    pub unsafe fn write_direct_at(&mut self, i:usize, data:&[T]) {
        assert!(i+data.len()<(&*self.ptr).len(), "attempted to write out-of-bounds");
        copy_nonoverlapping(data.as_ptr(), &mut (*self.ptr)[i], data.len())
    }

    #[inline]
    pub fn write_at<U:Sized,I:SliceIndex<[T],Output=U>>(&mut self, i:I, data:U) where T:Copy {
        unsafe { (*self.ptr)[i] = data; }
    }
}

//
//MapBuffer
//

unsafe fn map_access<B:Initialized>() -> GLenum {
    match (<B::MapRead as Bit>::VALUE, <B::MapWrite as Bit>::VALUE) {
        (true, false) => gl::READ_ONLY,
        (false, true) => gl::WRITE_ONLY,
        (true, true) => gl::READ_WRITE,
        (false, false) => 0,
    }
}

impl<T:?Sized, A:Initialized> Buffer<T,A> {
    unsafe fn map_raw<'a,B:Initialized>(&'a mut self) -> Map<'a,T,B> {
        let ptr = self.ptr.swap_mut_ptr(
            if self.size()==0 {
                NonNull::dangling().as_mut()
            } else if gl::MapNamedBuffer::is_loaded() {
                gl::MapNamedBuffer(self.id(), map_access::<B>())
            } else {
                COPY_WRITE_BUFFER.map_bind(self,
                    |b| gl::MapBuffer(b.target_id(), map_access::<B>())
                )
            }
        );

        Map {
            ptr: ptr,
            id: self.id(),
            offset: 0,
            buf: PhantomData
        }
    }
}

impl<T:?Sized,A:NonPersistent> Buffer<T,A> {
    #[inline]
    pub fn map<'a>(&'a mut self) -> Map<'a,T,MapRead> where A:ReadMappable {
        unsafe{self.map_raw()}
    }

    #[inline]
    pub fn map_write<'a>(&'a mut self) -> Map<'a,T,MapWrite> where A:WriteMappable {
        unsafe{self.map_raw()}
    }

    #[inline]
    pub fn map_mut<'a>(&'a mut self) -> Map<'a,T,MapReadWrite> where A:ReadWriteMappable {
        unsafe{self.map_raw()}
    }
}

//
//MapBufferRange
//

unsafe fn map_range_flags<B:Initialized>() -> GLbitfield {
    let mut flags = 0;
    if <B::MapRead as Bit>::VALUE {flags |= gl::MAP_READ_BIT;}
    if <B::MapWrite as Bit>::VALUE {flags |= gl::MAP_WRITE_BIT;}
    if <B::MapPersistent as Bit>::VALUE {
        flags |= gl::MAP_PERSISTENT_BIT;
        if !<B::MapRead as Bit>::VALUE {flags |= gl::MAP_UNSYNCHRONIZED_BIT;}
        if <B::MapWrite as Bit>::VALUE {flags |= gl::MAP_FLUSH_EXPLICIT_BIT;}
    }
    flags
}

//Note: we cannot simply implement a public map_range method on Slice or SliceMut, as then, we could
//split the buffer and then map it multiple times, which is not allowed, even for persistent mapping.

impl<'a,T:?Sized,A:Initialized> SliceMut<'a,T,A> {
    unsafe fn map_range_raw<'b,B:Initialized>(self) -> Map<'b,T,B> {
        let ptr = self.ptr.swap_mut_ptr(
            if self.size()==0 {
                NonNull::dangling().as_mut()
            } else if
                <B::MapPersistent as Bit>::VALUE ||
                gl::MapBufferRange::is_loaded() ||
                gl::MapNamedBufferRange::is_loaded()
            {
                let flags = map_range_flags::<B>();
                if gl::MapNamedBufferRange::is_loaded() {
                    gl::MapNamedBufferRange(
                        self.id(), self.offset() as GLintptr, self.size() as GLsizeiptr, flags
                    )
                } else {
                    COPY_WRITE_BUFFER.map_bind(&self, |b|
                        gl::MapBufferRange(
                            b.target_id(), self.offset() as GLintptr, self.size() as GLsizeiptr, flags
                        )
                    )
                }

            } else {
                if gl::MapNamedBuffer::is_loaded() {
                    gl::MapNamedBuffer(self.id(), map_access::<B>())
                } else {
                    COPY_WRITE_BUFFER.map_bind(&self, |b|
                        gl::MapBuffer(b.target_id(), map_access::<B>())
                    )
                }.offset(self.offset() as isize)
            }
        );

        Map {
            ptr: ptr,
            id: self.id(),
            offset: self.offset(),
            buf: PhantomData
        }
    }
}

//Note: we require a mutable references because all API access of the buffer store are invalid
//while mapped non-persistently

impl<T:Sized,A:NonPersistent> Buffer<[T],A> {

    #[inline]
    pub fn map_range<'a,U,I>(&'a mut self, i:I) -> Map<'a,U,MapRead> where
        U:?Sized,
        I:SliceIndex<[T],Output=U>,
        A:ReadMappable
    {
        unsafe { self.as_slice_mut().index_mut(i).map_range_raw() }
    }

    #[inline]
    pub fn map_range_write<'a,U,I>(&'a mut self, i:I) -> Map<'a,U,MapWrite> where
        U:?Sized,
        I:SliceIndex<[T],Output=U>,
        A:WriteMappable
    {
        unsafe { self.as_slice_mut().index_mut(i).map_range_raw() }
    }

    #[inline]
    pub fn map_range_mut<'a,U,I>(&'a mut self, i:I) -> Map<'a,U,MapReadWrite> where
        U:?Sized,
        I:SliceIndex<[T],Output=U>,
        A:ReadMappable+WriteMappable
    {
        unsafe { self.as_slice_mut().index_mut(i).map_range_raw() }
    }
}

//
//Persistent mapping
//

impl<'a,T:?Sized,A:Persistent> Slice<'a,T,A> {
    unsafe fn get_map_raw<'b,B:Persistent>(this:*const Self) -> Map<'b,T,B> {
        let mut ptr = MaybeUninit::uninit();

        //get the size of the full buffer
        let buf_size = (*this).ptr.buffer_size();

        if buf_size==0 {
            ptr = MaybeUninit::new(NonNull::dangling().as_ptr())
        } else if gl::GetNamedBufferPointerv::is_loaded() {
            gl::GetNamedBufferPointerv((&*this).id(), gl::BUFFER_MAP_POINTER, ptr.as_mut_ptr());
        } else {
            COPY_READ_BUFFER.map_bind(&*this, |b|
                gl::GetBufferPointerv(b.target_id(), gl::BUFFER_MAP_POINTER, ptr.as_mut_ptr())
            );
        }

        //if the pointer is null, we need to map the buffer first
        if buf_size>0 && ptr.get_ref().is_null() {

            //needs to be the A flags because this map will be used for any other maps in the future
            let flags = map_range_flags::<A>();

            //map the buffer
            ptr = MaybeUninit::new(
                if gl::GetNamedBufferParameteriv::is_loaded() && gl::MapNamedBufferRange::is_loaded() {
                    gl::MapNamedBufferRange((&*this).id(), 0, buf_size as GLsizeiptr, flags)
                } else {
                    COPY_READ_BUFFER.map_bind(&*this,
                        |b| gl::MapBufferRange(b.target_id(), 0, buf_size as GLsizeiptr, flags)
                    )
                }
            );

        }

        if (&*this).size()>0 && <B::MapRead as Bit>::VALUE {
            //since we don't use coherent, we have to provide a barrier to tell the GL that we intend to read them
            gl::MemoryBarrier(gl::CLIENT_MAPPED_BUFFER_BARRIER_BIT);

            //TODO: store a proper sync object in the buffer that gets updated every time the buffer
            //is modified. That way, we don't need to block the GPU for *everything* whenever we need a pointer.

            //if we don't wait for any previous writes to finish, then reads may not see them
            gl::Finish();
        }

        Map {
            ptr: (&*this).ptr.swap_mut_ptr(ptr.assume_init().offset((&*this).offset() as isize)),
            id: (&*this).id(),
            offset: (&*this).offset(),
            buf: PhantomData
        }

    }
}

impl<'a,T:?Sized,A:ReadMappable+Persistent> Slice<'a,T,A> {
    #[inline] pub fn get_map(&self) -> Map<T,PersistMapRead> {
        unsafe {Self::get_map_raw(self)}
    }
}

impl<'a,T:?Sized,A:Persistent> SliceMut<'a,T,A> {

    #[inline]
    pub fn get_map(&self) -> Map<T,PersistMapRead> where A:ReadMappable {
        unsafe {Slice::get_map_raw(&self.as_immut())}
    }

    #[inline]
    pub fn get_map_write(&mut self) -> Map<T,PersistMapWrite> where A:WriteMappable {
        unsafe {Slice::get_map_raw(&self.as_immut())}
    }

    #[inline]
    pub fn get_map_mut(&mut self) -> Map<T,PersistMapReadWrite> where A:ReadWriteMappable {
        unsafe {Slice::get_map_raw(&self.as_immut())}
    }

}

impl<T:?Sized,A:Persistent> Buffer<T,A> {

    #[inline]
    pub fn get_map(&self) -> Map<T,PersistMapRead> where A:ReadMappable {
        unsafe {Slice::get_map_raw(&self.as_slice())}
    }

    #[inline]
    pub fn get_map_write(&mut self) -> Map<T,PersistMapWrite> where A:WriteMappable {
        unsafe {Slice::get_map_raw(&self.as_slice())}
    }

    #[inline]
    pub fn get_map_mut(&mut self) -> Map<T,PersistMapReadWrite> where A:ReadWriteMappable {
        unsafe {Slice::get_map_raw(&self.as_slice())}
    }

}
