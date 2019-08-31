
use crate::gl;
use crate::gl::types::*;
use crate::{GLVersion, GL15, GL44, GLError};
use crate::{Resource, Target, Binding, BindingLocation};

use std::alloc::{Global, Alloc, Layout};
use std::marker::{PhantomData, Unsize};
use std::ptr::{null, NonNull};
use std::slice::from_raw_parts;
use std::ops::CoerceUnsized;
use std::mem::*;

use trait_arith::{Boolean, True, False};

pub use self::raw::*;
pub use self::hint::*;
pub use self::access::*;
pub use self::map::*;
pub use self::slice::*;
pub use self::attrib_array::*;

mod raw;
mod hint;
mod access;
mod map;
mod slice;
mod attrib_array;

pub(self) union BufPtr<T:?Sized> {
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
    #[inline] pub fn len(&self) -> usize { unsafe {(&*self.ptr).len()} }
}

impl<T:?Sized, A:BufferAccess> Buf<T,A> {

    #[inline] pub fn id(&self) -> GLuint { unsafe {BufPtr{rust_mut: self.ptr}.buf} }
    #[inline] pub fn size(&self) -> usize { unsafe {size_of_val(&*self.ptr)} }
    #[inline] pub fn align(&self) -> usize { unsafe {align_of_val(&*self.ptr)} }

    #[inline]
    pub unsafe fn storage_raw(_gl: &GL44, raw: RawBuffer, data: *const T, hint:Option<StorageFlags>) -> Self {

        //get the size of the object
        let size = size_of_val(&*data);

        //swap out the first half of the data pointer with the buffer id in order to get the void ptr
        //half and to construct the pointer for the buffer object
        let mut conv = BufPtr{ rust: data };
        let ptr = conv.gl;
        conv.buf = raw.id();

        //get the creation flags
        let mut flags = 0;
        if <A::Read as Boolean>::VALUE { flags |= gl::MAP_READ_BIT};
        if <A::Write as Boolean>::VALUE { flags |= gl::MAP_WRITE_BIT | gl::DYNAMIC_STORAGE_BIT};
        if <A::Persistent as Boolean>::VALUE { flags |= gl::MAP_PERSISTENT_BIT};
        if let Some(hints) = hint { flags |= gl::CLIENT_STORAGE_BIT & hints.bits() }

        //upload the data
        let mut target = BufferTarget::CopyWriteBuffer.as_loc();
        gl::BufferStorage(target.bind(&raw).target_id(), size as GLsizeiptr, ptr, flags);

        //make sure we don't delete the buffer by accident
        forget(raw);

        //now, constuct a buffer with that pointer, where the leading half is the buffer id and the
        //latter half is any object metadata
        Buf {
            ptr: conv.rust_mut,
            access: PhantomData
        }

    }

}

impl<T:?Sized, A:BufferAccess<Persistent=False>> Buf<T,A>  {

    #[inline]
    pub unsafe fn data_raw(raw: RawBuffer, data: *const T, usage: Option<BufferUsage>) -> Self {
        //get the size of the object
        let size = size_of_val(&*data);

        //swap out the first half of the data pointer with the buffer id in order to get the void ptr
        //half and to construct the pointer for the buffer object
        let mut conv = BufPtr{ rust: data };
        let ptr = conv.gl;
        conv.buf = raw.id();

        //upload the data
        let mut target = BufferTarget::CopyWriteBuffer.as_loc();
        let tar = target.bind(&raw).target_id();
        gl::BufferData(tar, size as GLsizeiptr, ptr, usage.unwrap_or(Default::default()) as GLenum);

        //make sure we don't delete the buffer by accident
        forget(raw);

        //now, constuct a buffer with that pointer, where the leading half is the buffer id and the
        //latter half is any object metadata
        Buf {
            ptr: conv.rust_mut,
            access: PhantomData
        }
    }

    #[inline]
    pub unsafe fn from_raw(gl: &GL15, data: *const T, hint: Option<BufferCreationHint>) -> Self {
        let raw = RawBuffer::gen(gl);
        if let Ok(gl4) = gl.try_as_gl44() {
            Self::storage_raw(&gl4, raw, data, hint.map(|h| h.1))
        } else {
            Self::data_raw(raw, data, hint.map(|h| h.0))
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
        Global.dealloc(non_null.cast(), Layout::for_value(&*ptr));

        result
    }
}

impl<T:?Sized, A:BufferAccess> Buf<T,A> {

    pub fn storage(gl: &GL44, raw: RawBuffer, data: Box<T>, hint: Option<StorageFlags>) -> Self {
        move_copy(data, |ptr| unsafe{Self::storage_raw(gl, raw, ptr, hint)})
    }

    pub fn storage_ref(gl: &GL44, raw: RawBuffer, data: &T, hint: Option<StorageFlags>) -> Self where T:GPUCopy {
        unsafe { Self::storage_raw(gl, raw, data as *const T, hint) }
    }

    pub fn alloc_storage(gl: &GL44, raw: RawBuffer, hint: Option<StorageFlags>) -> Self where T:Sized{
        unsafe { Self::storage_raw(gl, raw, null::<T>(), hint) }
    }

}

impl<T:?Sized, A:BufferAccess<Persistent=False>> Buf<T,A> {

    pub fn data(raw: RawBuffer, data: Box<T>, usage: Option<BufferUsage>) -> Self {
        move_copy(data, |ptr| unsafe{Self::data_raw(raw, ptr, usage)})
    }
    pub fn from_box(gl: &GL15, data: Box<T>, hint: Option<BufferCreationHint>) -> Self {
        move_copy(data, |ptr| unsafe{Self::from_raw(gl, ptr, hint)})
    }

    pub fn data_ref(uninit: RawBuffer, data: &T, usage: Option<BufferUsage>) -> Self where T:GPUCopy{
        unsafe { Self::data_raw(uninit, data as *const T, usage) }
    }
    pub fn from_ref(gl: &GL15, data: &T, hint: Option<BufferCreationHint>) -> Self where T:GPUCopy{
        unsafe { Self::from_raw(gl, data as *const T, hint) }
    }

    pub unsafe fn alloc_data(raw: RawBuffer, usage: Option<BufferUsage>) -> Self where T:Sized {
        Self::data_raw(raw, null::<T>(), usage)
    }
    pub unsafe fn alloc(gl: &GL15, hint: Option<BufferCreationHint>) -> Self where T:Sized {
        Self::from_raw(gl, null::<T>(), hint)
    }

}

impl<T:Sized, A:BufferAccess> Buf<[T],A> {
    pub unsafe fn alloc_storage_count(gl: &GL44, raw:RawBuffer, count: usize, hint: Option<StorageFlags>) -> Self {
        Self::storage_raw(gl, raw, from_raw_parts(null::<T>(), count), hint)
    }
}

impl<T:Sized, A:BufferAccess<Persistent=False>> Buf<[T],A> {
    pub unsafe fn alloc_data_count(raw: RawBuffer, count: usize, usage: Option<BufferUsage>) -> Self {
        Self::data_raw(raw, from_raw_parts(null::<T>(), count), usage)
    }
    pub unsafe fn alloc_count(gl: &GL15, count: usize, hint: Option<BufferCreationHint>) -> Self {
        Self::from_raw(gl, from_raw_parts(null::<T>(), count), hint)
    }
}

macro_rules! impl_creation_method {
    ($trait:ident $ref:ident $box:ident $alloc:ident $alloc_count:ident) => {
        impl<T:?Sized> Buf<T,$trait> {

            #[inline]
            pub fn $ref(gl: &GL15, data: &T, hint: Option<BufferCreationHint>) -> Self where T:GPUCopy {
                Self::from_ref(gl, data, hint)
            }

            #[inline]
            pub fn $box(gl: &GL15, data: Box<T>, hint: Option<BufferCreationHint>) -> Self {
                Self::from_box(gl, data, hint)
            }

            #[inline]
            pub unsafe fn $alloc(gl: &GL15, hint: Option<BufferCreationHint>) -> Self where T:Sized{
                Self::alloc(gl, hint)
            }
        }

        impl<T:Sized> Buf<[T],$trait> {
            #[inline]
            pub unsafe fn $alloc_count(gl: &GL15, count:usize, hint: Option<BufferCreationHint>) -> Self {
                Self::alloc_count(gl, count, hint)
            }
        }
    }
}

impl_creation_method!(CopyOnly copyonly_from_ref new_copyonly alloc_copyonly alloc_copyonly_count);
impl_creation_method!(Read readonly_from_ref new_readonly alloc_readonly alloc_readonly_count);
impl_creation_method!(Write writeonly_from_ref new_writeonly alloc_writeonly alloc_writeonly_count);
impl_creation_method!(ReadWrite readwrite_from_ref new_readwrite alloc_readwrite alloc_readwrite_count);

//
//Reading a buffer into its interior value
//

impl<T:?Sized, A:BufferAccess> Buf<T,A> {

    pub fn into_box(self) -> Box<T> {
        unsafe {
            //read the data into a box
            let data = self.as_slice()._into_box();

            //next, delete the buffer and forget the handle without running the object destructor
            gl::DeleteBuffers(1, &self.id());
            forget(self);

            //finally, return the box
            return data;
        }
    }
}

impl<T:Sized, A:BufferAccess> Buf<T,A> {
    pub fn into_inner(self) -> T {
        unsafe {
            let mut data = MaybeUninit::uninit();
            self.as_slice().get_subdata_raw(data.get_mut() as *mut T);
            forget(self);
            data.assume_init()
        }
    }
}
