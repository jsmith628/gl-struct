
use crate::gl;
use crate::gl::types::*;
use crate::{GLVersion, GL15, GL44, GLError, assume_supported};
use crate::{Resource, Target, Binding, BindingLocation};

use std::alloc::{Global, Alloc, Layout};
use std::marker::{PhantomData, Unsize};
use std::slice::from_raw_parts;
use std::ops::CoerceUnsized;
use std::convert::TryInto;
use std::ptr::*;
use std::mem::*;

use trait_arith::{Boolean, True, False};

pub use self::raw::*;
pub use self::hint::*;
pub use self::access::*;
pub use self::storage::*;
pub use self::map::*;
pub use self::slice::*;
pub use self::cmp::*;
pub use self::pixel_transfer::*;
pub use self::attrib_array::*;

mod raw;
mod hint;
mod access;
mod storage;
mod map;
mod slice;
mod cmp;
mod pixel_transfer;
mod attrib_array;

pub type Buf<T,A> = Buffer<T,A>;
pub type RawBuf = RawBuffer;

pub(self) union BufPtr<T:?Sized> {
    gl: *const GLvoid,
    gl_mut: *mut GLvoid,
    c: *const u8,
    c_mut: *mut u8,
    rust: *const T,
    rust_mut: *mut T,
    buf: GLuint,
}

pub struct Buffer<T:?Sized, A:BufferAccess> {
    ptr: *mut T,
    access: PhantomData<A>
}

impl<U:?Sized, T:?Sized+Unsize<U>, A:BufferAccess> CoerceUnsized<Buffer<U,A>> for Buffer<T,A> {}
impl<T:?Sized, A:BufferAccess> !Sync for Buffer<T,A> {}
impl<T:?Sized, A:BufferAccess> !Send for Buffer<T,A> {}

impl<T:?Sized, A:BufferAccess> Buffer<T,A> {

    //
    //Basic information methods
    //

    #[inline] pub fn id(&self) -> GLuint { unsafe {BufPtr{rust_mut: self.ptr}.buf} }
    #[inline] pub fn size(&self) -> usize { unsafe {size_of_val(&*self.ptr)} }
    #[inline] pub fn align(&self) -> usize { unsafe {align_of_val(&*self.ptr)} }

    #[inline] pub fn gl(&self) -> GL15 { unsafe { assume_supported::<GL15>() } }

    //
    //Wrappers for glGetParameteriv
    //

    unsafe fn get_parameter_iv(&self, value:GLenum) -> GLint {
        let mut dest = MaybeUninit::uninit();
        if gl::GetNamedBufferParameteriv::is_loaded() {
            gl::GetNamedBufferParameteriv(self.id(), value, dest.as_mut_ptr());
        } else {
            let mut target = BufferTarget::CopyReadBuffer.as_loc();
            gl::GetBufferParameteriv(target.bind_buf(self).target_id(), value, dest.as_mut_ptr());
        }
        dest.assume_init()
    }

    #[inline] pub fn immutable_storage(&self) -> bool {
        unsafe {self.get_parameter_iv(gl::BUFFER_IMMUTABLE_STORAGE) != 0}
    }

    #[inline] pub fn storage_flags(&self) -> StorageFlags {
        unsafe {StorageFlags::from_bits(self.get_parameter_iv(gl::BUFFER_STORAGE_FLAGS) as GLbitfield).unwrap()}
    }

    #[inline] pub fn usage(&self) -> BufferUsage {
        unsafe {(self.get_parameter_iv(gl::BUFFER_USAGE) as GLenum).try_into().unwrap()}
    }

    #[inline] pub fn creation_flags(&self) -> BufferCreationFlags {
        BufferCreationFlags(self.usage(), self.storage_flags())
    }

}

//
//Methods for arrays
//

impl<T:Sized, A:BufferAccess> Buffer<[T],A> {
    #[inline] pub fn len(&self) -> usize { unsafe {(&*self.ptr).len()} }
}

impl<T:?Sized, A:BufferAccess> Buffer<T,A> {
    #[inline] pub fn as_slice(&self) -> BSlice<T,A> {BSlice::from(self)}
    #[inline] pub fn as_slice_mut(&mut self) -> BSliceMut<T,A> {BSliceMut::from(self)}
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

pub(self) fn map_alloc<T:?Sized, F:FnOnce(*mut T)>(meta: *const T, f:F) -> Box<T> {
    unsafe {
        //Manually allocate a pointer on the head that we will store the data in
        let data = Global.alloc(::std::alloc::Layout::for_value(&*meta)).unwrap().as_ptr();

        //next, construct a *mut T pointer from the u8 pointer we just allocated using the metadata
        //stored in this buf
        let mut ptr = BufPtr{ rust: meta };
        ptr.c_mut = data;

        //next, run the thing on the pointer
        f(ptr.rust_mut);

        //finally, make and return a Box to own the heap data
        Box::from_raw(ptr.rust_mut)
    }
}

//
//Reading a buffer into its interior value
//

impl<T:?Sized, A:BufferAccess> Buffer<T,A> {

    ///deallocates the buffer data without running the data's destructor
    #[inline] unsafe fn delete_data(self) {gl::DeleteBuffers(1, &self.id()); forget(self);}

    #[inline] unsafe fn _read_into_box(&self) -> Box<T> {
        map_alloc(self.ptr, |ptr| self.as_slice().get_subdata_raw(ptr))
    }

    pub fn into_box(self) -> Box<T> {
        unsafe {
            //read the data into a box
            let data = self._read_into_box();

            //next, delete the buffer and forget the handle without running the object destructor
            self.delete_data();

            //finally, return the box
            return data;
        }
    }
}

impl<T:Sized, A:BufferAccess> Buffer<T,A> {
    pub fn into_inner(self) -> T {
        unsafe {
            //read the data
            let mut data = MaybeUninit::uninit();
            self.as_slice().get_subdata_raw(data.get_mut() as *mut T);

            //next, delete the buffer and forget the handle without running the object destructor
            self.delete_data();

            //return the data
            data.assume_init()
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

                        let mut ptr = BufPtr { rust_mut: self.ptr};
                        ptr.c = null();

                        Self::storage_raw(&assume_supported(), raw, ptr.rust, Some(self.storage_flags()))
                    }
                }

                impl<U:?Sized,B:NonPersistentAccess> Mirror for Buffer<U,B>  {
                    unsafe fn _mirror(&self) -> Self {
                        let raw = RawBuffer::gen(&self.gl());

                        let mut ptr = BufPtr { rust_mut: self.ptr};
                        ptr.c = null();

                        if self.immutable_storage() {
                            Self::storage_raw(&assume_supported(), raw, ptr.rust, Some(self.storage_flags()))
                        } else {
                            Self::data_raw(raw, ptr.rust, Some(self.usage()))
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

// impl<T:PartialEq<U>+?Sized, U:?Sized, A:BufferAccess> PartialEq<U> for Buffer<T,A> {
//     #[inline] fn eq(&self, rhs:&U) -> bool {}
// }


impl<T:?Sized, A:BufferAccess> Drop for Buffer<T,A> {
    fn drop(&mut self) {
        unsafe {
            //if the data needs to be dropped, read the data into a box so
            //that the box's destructor can run the object's destructor
            if (&*self.ptr).needs_drop_val() {
                drop(self._read_into_box());
            }

            //and finally, delete the buffer
            gl::DeleteBuffers(1, &self.id());
        }

    }
}
