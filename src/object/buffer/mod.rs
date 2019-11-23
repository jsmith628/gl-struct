
use super::*;

use crate::gl;

use std::alloc::{Global, Alloc, Layout};
use std::marker::{PhantomData, Unsize};
use std::slice::from_raw_parts;
use std::ops::CoerceUnsized;
use std::convert::TryInto;
use std::ptr::*;
use std::mem::*;

pub use self::raw::*;
pub use self::hint::*;
pub use self::access::*;
pub use self::storage::*;
pub use self::map::*;
pub use self::slice::*;
pub use self::any::*;
pub use self::cmp::*;
pub use self::fmt::*;
pub use self::pixel_transfer::*;
pub use self::attrib_array::*;

mod raw;
mod hint;
mod access;
mod storage;
mod map;
mod slice;
mod any;
mod cmp;
mod fmt;
mod pixel_transfer;
mod attrib_array;

pub type Buf<T,A> = Buffer<T,A>;
pub type BufSlice<'a,T,A> = Slice<'a,T,A>;
pub type BufSliceMut<'a,T,A> = SliceMut<'a,T,A>;
pub type BufMap<'a,T,A> = Map<'a,T,A>;

pub struct Buffer<T:?Sized, A> {
    ptr: BufPtr<T>,
    access: PhantomData<A>
}

impl<U:?Sized, T:?Sized+Unsize<U>, A:BufferAccess> CoerceUnsized<Buffer<U,A>> for Buffer<T,A> {}

impl<T:?Sized, A> Buffer<T,A> {
    #[inline] pub fn id(&self) -> GLuint { self.ptr.id() }
    #[inline] pub fn gl(&self) -> GL15 { unsafe { assume_supported::<GL15>() } }
}

impl<T:?Sized, A:BufferAccess> Buffer<T,A> {

    //
    //basic memory information
    //

    #[inline] pub fn size(&self) -> usize { self.ptr.size() }
    #[inline] pub fn align(&self) -> usize { self.ptr.align() }

    //
    //Wrappers for glGetBufferParameter*
    //

    #[inline] pub fn immutable_storage(&self) -> bool { self.ptr.immutable_storage() }
    #[inline] pub fn storage_flags(&self) -> StorageFlags { self.ptr.storage_flags() }
    #[inline] pub fn usage(&self) -> BufferUsage { self.ptr.usage() }
    #[inline] pub fn creation_flags(&self) -> BufferCreationFlags { self.ptr.creation_flags() }

}

//
//Methods for arrays
//

use std::slice::SliceIndex;

impl<T:Sized, A:BufferAccess> Buffer<[T],A> {
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

}

impl<T:?Sized, A:BufferAccess> Buffer<T,A> {
    #[inline] pub fn as_slice(&self) -> Slice<T,A> {Slice::from(self)}
    #[inline] pub fn as_slice_mut(&mut self) -> SliceMut<T,A> {SliceMut::from(self)}
}

//
//Helper methods for boxes and heap data
//

pub(self) fn map_dealloc<T:?Sized, U, F:FnOnce(*mut T)->U>(data: Box<T>, f:F) -> U {
    unsafe {
        //turn the box into a pointer
        let non_null = Box::<T>::into_raw_non_null(data);
        let ptr = non_null.as_ptr();

        //run the thing
        let result = f(ptr);

        //deallocate the heap storage without running the object destructor
        Global.dealloc(non_null.cast(), Layout::for_value(&*ptr));

        result
    }
}

pub(self) fn map_alloc<T:?Sized, F:FnOnce(*mut T)>(buf: BufPtr<T>, f:F) -> Box<T> {
    unsafe {
        //Manually allocate a pointer on the head that we will store the data in
        let data = Global.alloc(Layout::from_size_align_unchecked(buf.size(), buf.align())).unwrap().as_ptr();

        //next, construct a *mut T pointer from the u8 pointer we just allocated using the metadata
        //stored in this buf
        let ptr = buf.swap_mut_ptr_unchecked(data as *mut GLvoid);

        //next, run the thing on the pointer
        f(ptr);

        //finally, make and return a Box to own the heap data
        Box::from_raw(ptr)
    }
}

//
//Reading a buffer into its interior value
//

impl<T:?Sized, A:BufferAccess> Buffer<T,A> {

    ///deallocates the buffer data without running the data's destructor
    #[inline] unsafe fn forget_data(self) {gl::DeleteBuffers(1, &self.id()); forget(self);}

    #[inline] unsafe fn _read_into_box(&self) -> Box<T> {
        map_alloc(self.ptr, |ptr| self.as_slice().get_subdata_raw(ptr))
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
}

impl<T:Sized, A:BufferAccess> Buffer<T,A> {

    unsafe fn _read(&self) -> T {
        let mut data = MaybeUninit::uninit();
        self.as_slice().get_subdata_raw(data.get_mut() as *mut T);
        data.assume_init()
    }

    pub fn into_inner(self) -> T {
        unsafe {
            //read the data
            let data = self._read();

            //next, delete the buffer and forget the handle without running the object destructor
            self.forget_data();

            //return the data
            data
        }
    }
}

impl<T:?Sized+GPUCopy,A:BufferAccess> Clone for Buffer<T,A> {
    fn clone(&self) -> Self {
        unsafe {
            //allocate storage
            let mut uninit = {
                trait Mirror { unsafe fn _mirror(&self) -> Self; }
                impl<U:?Sized,B:BufferAccess> Mirror for Buffer<U,B> {
                    default unsafe fn _mirror(&self) -> Self {
                        let raw = RawBuffer::gen(&self.gl());
                        let ptr = self.ptr.swap_ptr_unchecked(null());

                        Self::storage_raw(&assume_supported(), raw, ptr, Some(self.storage_flags()))
                    }
                }

                impl<U:?Sized,B:NonPersistentAccess> Mirror for Buffer<U,B>  {
                    unsafe fn _mirror(&self) -> Self {
                        let raw = RawBuffer::gen(&self.gl());
                        let ptr = self.ptr.swap_ptr_unchecked(null());

                        if self.immutable_storage() {
                            Self::storage_raw(&assume_supported(), raw, ptr, Some(self.storage_flags()))
                        } else {
                            Self::data_raw(raw, ptr, Some(self.usage()))
                        }
                    }
                }

                self._mirror()
            };

            //copy the data directly
            self.as_slice().copy_subdata_unchecked(&mut uninit.as_slice_mut());

            uninit
        }
    }
}

impl<T:?Sized, A> Drop for Buffer<T,A> {
    default fn drop(&mut self) {

        trait _Drop { unsafe fn _drop(&mut self); }

        impl<T:?Sized, A> _Drop for Buffer<T, A> { default unsafe fn _drop(&mut self) {} }
        impl<T:?Sized, A:BufferAccess> _Drop for Buffer<T, A> {
            default unsafe fn _drop(&mut self) {
                if self.ptr.needs_drop() { drop(self._read_into_box()); }
            }
        }

        impl<T:Sized, A:BufferAccess> _Drop for Buffer<T, A> {
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
