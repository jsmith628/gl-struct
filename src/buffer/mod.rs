
use super::*;

use crate::gl;
use crate::version::*;
use crate::pixel::*;
use crate::vertex_array::*;

use std::marker::{PhantomData, Unsize};
use std::slice::{from_raw_parts, SliceIndex};
use std::ops::CoerceUnsized;
use std::convert::TryInto;
use std::alloc::*;
use std::ptr::*;
use std::mem::*;

pub use self::raw::*;
pub use self::target::*;
pub use self::hint::*;
pub use self::storage::*;
pub use self::uninit::*;
pub use self::map::*;
pub use self::slice::*;
pub use self::any::*;

pub mod storage;
pub mod hint;

mod raw;
mod target;
mod uninit;
mod map;
mod slice;
mod any;

pub struct Buffer<T:?Sized, A=ReadWrite> {
    ptr: BufPtr<T>,
    access: PhantomData<A>
}

impl<U:?Sized, T:?Sized+Unsize<U>, A:BufferStorage> CoerceUnsized<Buffer<U,A>> for Buffer<T,A> {}

impl<T:?Sized, A> Buffer<T,A> {
    #[inline] pub fn id(&self) -> GLuint { self.ptr.id() }
    #[inline] pub fn gl(&self) -> GL_ARB_vertex_buffer_object { unsafe { assume_supported() } }

    #[inline] pub fn is(id: GLuint) -> bool { unsafe { gl::IsBuffer(id) != 0 } }

    #[inline] pub fn delete(self) { drop(self); }
    #[inline] pub fn delete_buffers(buffers: Box<[Self]>) { drop(buffers); }

}

impl<T:?Sized, A:BufferStorage> Buffer<T,A> {

    //
    //basic memory information
    //

    #[inline] pub fn size(&self) -> usize { self.ptr.size() }
    #[inline] pub fn align(&self) -> usize { self.ptr.align() }

    //
    //Wrappers for glGetBufferParameter*
    //

    #[inline] pub fn immutable_storage(&self) -> bool { unsafe {self.ptr.immutable_storage()} }
    #[inline] pub fn storage_flags(&self) -> StorageFlags { unsafe {self.ptr.storage_flags()} }
    #[inline] pub fn usage(&self) -> BufferUsage { unsafe {self.ptr.usage()} }
    #[inline] pub fn creation_flags(&self) -> BufferCreationFlags { unsafe {self.ptr.creation_flags()} }

    //
    //Conversion between access types
    //

    #[inline] pub unsafe fn downgrade_unchecked<B:BufferStorage>(self) -> Buffer<T,B> { transmute(self) }
    #[inline] pub unsafe fn downgrade_ref_unchecked<B:BufferStorage>(&self) -> &Buffer<T,B> { transmute(self) }
    #[inline] pub unsafe fn downgrade_mut_unchecked<B:BufferStorage>(&mut self) -> &mut Buffer<T,B> { transmute(self) }

    #[inline]
    pub fn downgrade<B:BufferStorage>(self) -> Buffer<T,B> where A:DowngradesTo<B> {
        unsafe { transmute(self) }
    }

    #[inline]
    pub fn downgrade_ref<B:BufferStorage>(&self) -> &Buffer<T,B> where A:DowngradesTo<B> {
        unsafe { transmute(self) }
    }

    #[inline]
    pub fn downgrade_mut<B:BufferStorage>(&mut self) -> &mut Buffer<T,B> where A:DowngradesTo<B> {
        unsafe { transmute(self) }
    }

    //
    //Slice creation
    //

    #[inline] pub fn as_slice(&self) -> Slice<T,A> {Slice::from(self)}
    #[inline] pub fn as_slice_mut(&mut self) -> SliceMut<T,A> {SliceMut::from(self)}


    //
    //Reading a buffer into a box or stack value
    //

    ///deallocates the buffer data without running the data's destructor
    #[inline] unsafe fn forget_data(self) {gl::DeleteBuffers(1, &self.id()); forget(self);}

    #[inline] unsafe fn _read_into_box(&self) -> Box<T> {
        map_alloc(self.ptr, |ptr| self.as_slice().get_subdata_raw(ptr))
    }

    unsafe fn _read(&self) -> T where T:Sized {
        let mut data = MaybeUninit::uninit();
        self.as_slice().get_subdata_raw(data.get_mut() as *mut T);
        data.assume_init()
    }

    pub fn into_box(self) -> Box<T> {
        unsafe {
            //read the data into a box
            let data = self._read_into_box();

            //next, delete the buffer and forget the handle without running the object destructor
            self.forget_data();

            //finally, return the box
            return data;
        }
    }

    pub fn into_inner(self) -> T where T:Sized {
        unsafe {
            //read the data
            let data = self._read();

            //next, delete the buffer and forget the handle without running the object destructor
            self.forget_data();

            //return the data
            data
        }
    }

    //
    //Buffer invalidation
    //

    pub unsafe fn invalidate_data_raw(&mut self) {
        if self.size()==0 { return; }
        if gl::InvalidateBufferData::is_loaded() {
            gl::InvalidateBufferData(self.id())
        } else if !self.immutable_storage() {
            let (size, usage) = (self.size() as GLsizeiptr, self.usage() as GLenum);
            COPY_WRITE_BUFFER.map_bind(self,
                |b| gl::BufferData(b.target_id(), size, null(), usage)
            );
        }
    }



}

impl<T:Sized, A:BufferStorage> Buffer<T,A> {
    pub fn invalidate_data(mut self) -> Buffer<MaybeUninit<T>, A> {
        unsafe {
            self.invalidate_data_raw();
            transmute(self)
        }
    }
}

impl<T:Sized, A:BufferStorage> Buffer<[T],A> {
    #[inline] pub fn len(&self) -> usize { self.ptr.len() }

    #[inline] pub fn split_at(&self, mid:usize) -> (Slice<[T],A>, Slice<[T],A>) { self.as_slice().split_at(mid) }
    #[inline] pub fn split_at_mut(&mut self, mid:usize) -> (SliceMut<[T],A>, SliceMut<[T],A>) {
        self.as_slice_mut().split_at_mut(mid)
    }

    #[inline] pub fn split_first(&self) -> Option<(Slice<T,A>, Slice<[T],A>)> { self.as_slice().split_first() }
    #[inline] pub fn split_first_mut(&mut self) -> Option<(SliceMut<T,A>, SliceMut<[T],A>)> {
        self.as_slice_mut().split_first_mut()
    }

    #[inline] pub fn split_last(&self) -> Option<(Slice<T,A>, Slice<[T],A>)> { self.as_slice().split_last() }
    #[inline] pub fn split_last_mut(&mut self) -> Option<(SliceMut<T,A>, SliceMut<[T],A>)> {
        self.as_slice_mut().split_last_mut()
    }

    #[inline] pub fn index<U:?Sized,I:SliceIndex<[T],Output=U>>(&self,i:I) -> Slice<U,A> { self.as_slice().index(i) }
    #[inline] pub fn index_mut<U:?Sized,I:SliceIndex<[T],Output=U>>(&mut self,i:I) -> SliceMut<U,A> {
        self.as_slice_mut().index_mut(i)
    }

    //
    //Buffer invalidation
    //

    pub fn invalidate_data(mut self) -> Buffer<[MaybeUninit<T>], A> {
        unsafe {
            self.invalidate_data_raw();
            transmute(self)
        }
    }

    pub fn invalidate_subdata<I:SliceIndex<[T]>>(mut self, i:I) -> Buffer<[MaybeUninit<T>], A> {
        unsafe {
            self.index_mut(i).invalidate_subdata_raw();
            transmute(self)
        }
    }

}

//
//Vertex Attributes
//
impl<T:AttribData, A:BufferStorage> Buffer<[T],A> {

    #[inline]
    pub fn attrib_array(&self) -> AttribArray<T::GLSL> {
        self.as_slice().into()
    }

    #[inline]
    pub fn split_attribs<'a>(&'a self) -> <T::GLSL as SplitAttribs<'a>>::Split
    where T::GLSL: SplitAttribs<'a>
    {
        <T::GLSL as SplitAttribs<'a>>::split_array(self.attrib_array())
    }

}

impl<F:SpecificCompressed, A:BufferStorage> Buffer<CompressedPixels<F>,A> {
    #[inline] pub fn blocks(&self) -> usize { self.ptr.blocks() }
    #[inline] pub fn pixel_count(&self) -> usize { self.ptr.pixel_count() }
}

impl<T, A:BufferStorage> Buffer<MaybeUninit<T>, A> {
    #[inline] pub unsafe fn assume_init(self) -> Buffer<T, A> { transmute(self) }
}

impl<T, A:BufferStorage> Buffer<[MaybeUninit<T>], A> {
    #[inline] pub unsafe fn assume_init(self) -> Buffer<[T], A> { transmute(self) }
}

//
//Helper methods for boxes and heap data
//

pub(self) fn map_dealloc<T:?Sized, U, F:FnOnce(*mut T)->U>(data: Box<T>, f:F) -> U {
    unsafe {
        //turn the box into a pointer
        let ptr = Box::<T>::into_raw(data);

        //run the thing
        let result = f(ptr);

        //deallocate the heap storage without running the object destructor
        dealloc(ptr as *mut u8, Layout::for_value(&*ptr));

        result
    }
}

pub(self) fn map_alloc<T:?Sized, F:FnOnce(*mut T)>(buf: BufPtr<T>, f:F) -> Box<T> {
    unsafe {
        //Manually allocate a pointer on the head that we will store the data in
        let data = alloc(Layout::from_size_align_unchecked(buf.size(), buf.align()));

        //next, construct a *mut T pointer from the u8 pointer we just allocated using the metadata
        //stored in this buf
        let ptr = buf.swap_mut_ptr_unchecked(data as *mut GLvoid);

        //next, run the thing on the pointer
        f(ptr);

        //finally, make and return a Box to own the heap data
        Box::from_raw(ptr)
    }
}

impl<T:?Sized+GPUCopy,A:BufferStorage> Clone for Buffer<T,A> {
    fn clone(&self) -> Self {
        unsafe {
            //allocate storage
            let mut dest = {
                let raw = UninitBuf::create(&self.gl());
                let ptr = self.ptr.swap_ptr_unchecked(null());

                if <A as BufferStorage>::MapPersistent::VALUE || self.immutable_storage() {
                    raw.storage_raw(
                        &assume_supported::<GL_ARB_buffer_storage>(), ptr, Some(self.storage_flags())
                    )
                } else {
                    raw.data_raw(ptr, Some(self.usage())).downgrade_unchecked()
                }
            };

            //copy the data directly
            self.as_slice().copy_subdata_unchecked(&mut dest.as_slice_mut());

            //return the buffer
            dest
        }
    }
}

impl<T:?Sized, A> Drop for Buffer<T,A> {
    fn drop(&mut self) {

        trait _Drop { unsafe fn _drop(&mut self); }

        impl<T:?Sized, A> _Drop for Buffer<T, A> { default unsafe fn _drop(&mut self) {} }
        impl<T:?Sized, A:BufferStorage> _Drop for Buffer<T, A> {
            default unsafe fn _drop(&mut self) {
                if self.ptr.needs_drop() { drop(self._read_into_box()); }
            }
        }

        impl<T:Sized, A:BufferStorage> _Drop for Buffer<T, A> {
            unsafe fn _drop(&mut self) { if self.ptr.needs_drop() { drop(self._read()); } }
        }


        unsafe {
            //run the destructor on the stored data if necessary
            self._drop();

            //finally, delete the buffer
            gl::DeleteBuffers(1, &self.ptr.id());
        }

    }
}
