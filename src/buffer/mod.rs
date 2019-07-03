
use crate::gl;
use crate::gl::types::*;
use crate::{GL, GL15, GL44, GLError};
use crate::{Resource, Target, Binding, BindingLocation};

use std::alloc::{GlobalAlloc, Layout, System};
use std::marker::{PhantomData, Unsize};
use std::ptr::null;
use std::slice::from_raw_parts;
use std::ops::CoerceUnsized;
use std::mem::*;

pub use self::binding::*;
pub use self::access::*;
pub use self::map::*;
pub use self::slice::*;
pub use self::attrib_array::*;

mod binding;
mod access;
mod map;
mod slice;
mod attrib_array;


gl_resource! {

    pub struct UninitBuf {
        gl = GL15,
        target = BufferTarget,
        gen = GenBuffers,
        bind = BindBuffer,
        is = IsBuffer,
        delete = DeleteBuffers
    }
}

pub(self) union RawBuf<T:?Sized> {
    gl: *const GLvoid,
    gl_mut: *mut GLvoid,
    c: *const u8,
    c_mut: *mut u8,
    rust: *const T,
    rust_mut: *mut T,
    buf: GLuint,
}

pub struct Buf<T:?Sized, A:BufferAccess> {
    ptr: *mut T,
    access: PhantomData<A>
}


impl<U:?Sized, T:?Sized+Unsize<U>, A:BufferAccess> CoerceUnsized<Buf<U,A>> for Buf<T,A> {}

impl<T:?Sized, A:BufferAccess> !Sync for Buf<T,A> {}
impl<T:?Sized, A:BufferAccess> !Send for Buf<T,A> {}

impl<T:Sized, A:BufferAccess> Buf<[T],A> {
    #[inline] pub fn len(&self) -> usize { self.size() / size_of::<T>() }
}

impl<T:?Sized, A:BufferAccess> Buf<T,A> {

    #[inline] pub fn id(&self) -> GLuint { unsafe {RawBuf{rust_mut: self.ptr}.buf} }
    #[inline] pub fn size(&self) -> usize { unsafe {size_of_val(&*self.ptr)} }
    #[inline] pub fn align(&self) -> usize { unsafe {align_of_val(&*self.ptr)} }

    unsafe fn _upload<F>(uninit: UninitBuf, data: *const T, fun: F) -> Self where F:FnOnce(GLenum, GLsizeiptr, *const GLvoid) {
        //get the size of the object
        let size = size_of_val(&*data);

        //swap out the first half of the data pointer with the buffer id in order to get the void ptr
        //half and to construct the pointer for the buffer object
        let mut raw = RawBuf{ rust: data };
        let ptr = raw.gl;
        raw.buf = uninit.0;

        //upload the data
        let mut target = BufferTarget::CopyWriteBuffer.as_loc();
        fun(target.bind(&uninit).target_id(), size as GLsizeiptr, ptr);

        //make sure we don't delete the buffer by accident
        forget(uninit);

        //now, constuct a buffer with that pointer, where the leading half is the buffer id and the
        //latter half is any object metadata
        Buf {
            ptr: raw.rust_mut,
            access: PhantomData
        }
    }

    #[inline]
    pub unsafe fn storage_raw(_gl: &GL44, uninit: UninitBuf, data: *const T) -> Self {
        Self::_upload(uninit, data,
            |tar, len, ptr| gl::BufferStorage(tar, len, ptr, StorageFlags::from_access::<A>().bits())
        )
    }

    #[inline]
    pub unsafe fn data_raw(gl: &GL15, uninit: UninitBuf, data: *const T) -> Self {
        Self::data_raw_hint(gl, uninit, data, A::default_usage())
    }

    #[inline]
    pub unsafe fn data_raw_hint(_gl: &GL15, uninit: UninitBuf, data: *const T, usage: BufferUsage) -> Self {
        Self::_upload(uninit, data, |tar, len, ptr| gl::BufferData(tar, len, ptr, usage as GLenum))
    }

    #[inline]
    pub unsafe fn from_raw(gl: &GL15, data: *const T) -> Self {
        Self::raw_with_hint(gl, data, A::default_usage())
    }

    #[inline]
    pub unsafe fn raw_with_hint(gl: &GL15, data: *const T, usage: BufferUsage) -> Self {
        let uninit = UninitBuf::gen(gl);
        if let Ok(gl4) = gl.try_as_gl44() {
            Self::storage_raw(&gl4, uninit, data)
        } else {
            Self::data_raw_hint(gl, uninit, data, usage)
        }
    }

}

trait NeedsDrop { fn needs_drop(&self) -> bool; }

impl<T:?Sized> NeedsDrop for T { #[inline] default fn needs_drop(&self) -> bool {true} }
impl<T:Sized> NeedsDrop for [T] { #[inline] fn needs_drop(&self) -> bool {self.len()>0 && needs_drop::<T>()} }
impl<T:Sized> NeedsDrop for T { #[inline] fn needs_drop(&self) -> bool {needs_drop::<T>()} }

impl<T:?Sized, A:BufferAccess> Drop for Buf<T,A> {
    fn drop(&mut self) {

        unsafe {
            //if the data needs to be dropped, read the data into a box so
            //that the box's destructor can run the object's destructor
            if (&*self.ptr).needs_drop() {
                let data = self.as_slice()._into_box();
                drop(data);
            }

            //and finally, delete the buffer
            gl::DeleteBuffers(1, &self.id());
        }

    }
}

fn move_copy<T:?Sized, U, F:FnOnce(*mut T)->U>(data: Box<T>, f:F) -> U {
    unsafe {
        //turn the box into a pointer
        let non_null = Box::<T>::into_raw_non_null(data);
        let ptr = non_null.as_ptr();

        //run the thing
        let result = f(ptr);

        //deallocate the heap storage without running the object destructor
        System.dealloc(non_null.cast().as_ptr(), Layout::for_value(&*ptr));

        result
    }
}

//
//Buffer creation from a Reference
//

impl<T:GPUCopy+?Sized, A:BufferAccess> Buf<T,A> {
    #[inline] pub fn storage(gl: &GL44, uninit: UninitBuf, data: &T) -> Self {
        unsafe { Self::storage_raw(gl, uninit, data as *const T) }
    }
    #[inline] pub fn data(gl: &GL15, uninit: UninitBuf, data: &T) -> Self {
        unsafe { Self::data_raw(gl, uninit, data as *const T) }
    }
    #[inline] pub fn data_hint(gl: &GL15, uninit: UninitBuf, data: &T, usage: BufferUsage) -> Self {
        unsafe { Self::data_raw_hint(gl, uninit, data as *const T, usage) }
    }
    #[inline] pub fn from_ref(gl: &GL15, data: &T) -> Self {
        unsafe { Self::from_raw(gl, data as *const T) }
    }
    #[inline] pub fn from_ref_with_hint(gl: &GL15, data: &T, usage: BufferUsage) -> Self {
        unsafe { Self::raw_with_hint(gl, data as *const T, usage) }
    }
}

impl<T:GPUCopy+?Sized> Buf<T,CopyOnly> {
    #[inline] pub fn new_immut(gl: &GL15, data: &T) -> Self { Self::from_ref(gl, data) }
    #[inline] pub fn immut_with_hint(gl: &GL15, data: &T, usage: BufferUsage) -> Self {
        Self::from_ref_with_hint(gl, data, usage)
    }
}
impl<T:GPUCopy+?Sized> Buf<T,Read> {
    #[inline] pub fn new_readonly(gl: &GL15, data: &T) -> Self { Self::from_ref(gl, data) }
    #[inline] pub fn readonly_with_hint(gl: &GL15, data: &T, usage: BufferUsage) -> Self {
        Self::from_ref_with_hint(gl, data, usage)
    }
}
impl<T:GPUCopy+?Sized> Buf<T,Write> {
    #[inline] pub fn new_writeonly(gl: &GL15, data: &T) -> Self { Self::from_ref(gl, data) }
    #[inline] pub fn writeonly_with_hint(gl: &GL15, data: &T, usage: BufferUsage) -> Self {
        Self::from_ref_with_hint(gl, data, usage)
    }
}
impl<T:GPUCopy+?Sized> Buf<T,ReadWrite> {
    #[inline] pub fn new_readwrite(gl: &GL15, data: &T) -> Self { Self::from_ref(gl, data) }
    #[inline] pub fn readwriter_with_hint(gl: &GL15, data: &T, usage: BufferUsage) -> Self {
        Self::from_ref_with_hint(gl, data, usage)
    }
}

//
//Buffer creation from a Box
//

impl<T:?Sized, A:BufferAccess> Buf<T,A> {
    #[inline] pub fn storage_from_box(gl: &GL44, uninit: UninitBuf, data: Box<T>) -> Self {
        move_copy(data, |ptr| unsafe{Self::storage_raw(gl, uninit, ptr)})
    }
    #[inline] pub fn data_from_box(gl: &GL15, uninit: UninitBuf, data: Box<T>) -> Self {
        move_copy(data, |ptr| unsafe{Self::data_raw(gl, uninit, ptr)})
    }
    #[inline] pub fn data_hint_from_box(gl: &GL15, uninit: UninitBuf, data: Box<T>, usage: BufferUsage) -> Self {
        move_copy(data, |ptr| unsafe{Self::data_raw_hint(gl, uninit, ptr, usage)})
    }
    #[inline] pub fn from_box(gl: &GL15, data: Box<T>) -> Self {
        move_copy(data, |ptr| unsafe{Self::from_raw(gl, ptr)})
    }
    #[inline] pub fn from_box_with_hint(gl: &GL15, data: Box<T>, usage: BufferUsage) -> Self {
        move_copy(data, |ptr| unsafe{Self::raw_with_hint(gl, ptr, usage)})
    }
}

impl<T:?Sized> Buf<T,CopyOnly> {
    #[inline] pub fn immut_from(gl: &GL15, data: Box<T>) -> Self { Self::from_box(gl, data) }
    #[inline] pub fn immut_from_with_hint(gl: &GL15, data: Box<T>, usage: BufferUsage) -> Self {
        Self::from_box_with_hint(gl, data, usage)
    }
}
impl<T:?Sized> Buf<T,Read> {
    #[inline] pub fn readonly_from(gl: &GL15, data: Box<T>) -> Self { Self::from_box(gl, data) }
    #[inline] pub fn readonly_from_with_hint(gl: &GL15, data: Box<T>, usage: BufferUsage) -> Self {
        Self::from_box_with_hint(gl, data, usage)
    }
}
impl<T:?Sized> Buf<T,Write> {
    #[inline] pub fn writeonly_from(gl: &GL15, data: Box<T>) -> Self { Self::from_box(gl, data) }
    #[inline] pub fn writeonly_from_with_hint(gl: &GL15, data: Box<T>, usage: BufferUsage) -> Self {
        Self::from_box_with_hint(gl, data, usage)
    }
}
impl<T:?Sized> Buf<T,ReadWrite> {
    #[inline] pub fn readwrite_from(gl: &GL15, data: Box<T>) -> Self { Self::from_box(gl, data) }
    #[inline] pub fn readwrite_from_with_hint(gl: &GL15, data: Box<T>, usage: BufferUsage) -> Self {
        Self::from_box_with_hint(gl, data, usage)
    }
}

//
//Allocating uninitialized and zeroed space
//

impl<T:Sized, A:BufferAccess> Buf<T,A> {
    #[inline] pub unsafe fn alloc(gl: &GL15) -> Self {Self::from_raw(gl, null::<T>())}
    #[inline] pub unsafe fn alloc_hint(gl: &GL15, usage: BufferUsage) -> Self {Self::raw_with_hint(gl, null::<T>(), usage)}
}

impl<T:Sized, A:BufferAccess> Buf<[T],A> {
    #[inline] pub unsafe fn alloc_count(gl: &GL15, count: usize) -> Self {
        Self::from_raw(gl, from_raw_parts(null::<T>(), count))
    }
    #[inline] pub unsafe fn alloc_count_hint(gl: &GL15, count: usize, usage: BufferUsage) -> Self {
        Self::raw_with_hint(gl, from_raw_parts(null::<T>(), count), usage)
    }
}

impl<T:Sized> Buf<T,CopyOnly> {
    #[inline] pub unsafe fn alloc_immut(gl: &GL15) -> Self {Self::alloc(gl)}
    #[inline] pub unsafe fn alloc_immut_hint(gl: &GL15, usage: BufferUsage) -> Self {Self::alloc_hint(gl, usage)}
}
impl<T:Sized> Buf<[T],CopyOnly> {
    #[inline] pub unsafe fn alloc_immut_count(gl: &GL15, count: usize) -> Self {Self::alloc_count(gl,count)}
    #[inline] pub unsafe fn alloc_immut_count_hint(gl: &GL15, count: usize, usage: BufferUsage) -> Self {
        Self::alloc_count_hint(gl, count, usage)
    }
}

impl<T:Sized> Buf<T,Read> {
    #[inline] pub unsafe fn alloc_readonly(gl: &GL15) -> Self {Self::alloc(gl)}
    #[inline] pub unsafe fn alloc_readonly_hint(gl: &GL15, usage: BufferUsage) -> Self {Self::alloc_hint(gl, usage)}
}
impl<T:Sized> Buf<[T],Read> {
    #[inline] pub unsafe fn alloc_readonly_count(gl: &GL15, count: usize) -> Self {Self::alloc_count(gl,count)}
    #[inline] pub unsafe fn alloc_readonly_count_hint(gl: &GL15, count: usize, usage: BufferUsage) -> Self {
        Self::alloc_count_hint(gl, count, usage)
    }
}

impl<T:Sized> Buf<T,Write> {
    #[inline] pub unsafe fn alloc_writeonly(gl: &GL15) -> Self {Self::alloc(gl)}
    #[inline] pub unsafe fn alloc_writeonly_hint(gl: &GL15, usage: BufferUsage) -> Self {Self::alloc_hint(gl, usage)}
}
impl<T:Sized> Buf<[T],Write> {
    #[inline] pub unsafe fn alloc_writeonly_count(gl: &GL15, count: usize) -> Self {Self::alloc_count(gl,count)}
    #[inline] pub unsafe fn alloc_writeonly_count_hint(gl: &GL15, count: usize, usage: BufferUsage) -> Self {
        Self::alloc_count_hint(gl, count, usage)
    }
}

impl<T:Sized> Buf<T,ReadWrite> {
    #[inline] pub unsafe fn alloc_readwrite(gl: &GL15) -> Self {Self::alloc(gl)}
    #[inline] pub unsafe fn alloc_readwrite_hint(gl: &GL15, usage: BufferUsage) -> Self {Self::alloc_hint(gl, usage)}
}
impl<T:Sized> Buf<[T],ReadWrite> {
    #[inline] pub unsafe fn alloc_readwrite_count(gl: &GL15, count: usize) -> Self {Self::alloc_count(gl,count)}
    #[inline] pub unsafe fn alloc_readwrite_count_hint(gl: &GL15, count: usize, usage: BufferUsage) -> Self {
        Self::alloc_count_hint(gl, count, usage)
    }
}

//
//Reading a buffer into its interior value
//

impl<T:?Sized, A:BufferAccess> Buf<T,A> {

    pub fn forget(self) {
        unsafe { gl::DeleteBuffers(1, &self.id()) };
        forget(self);
    }

    pub fn into_box(self) -> Box<T> {
        unsafe {
            //read the data into a box
            let data = self.as_slice()._into_box();

            //next, delete the buffer and forget the handle without running the object destructor
            self.forget();

            //finally, return the box
            return data;
        }
    }
}

impl<T:Sized, A:BufferAccess> Buf<T,A> {
    pub fn into_inner(self) -> T {
        unsafe {
            let mut data = uninitialized();
            self.as_slice().get_subdata_raw(&mut data as *mut T);
            forget(self);
            data
        }
    }
}
