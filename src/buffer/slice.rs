use super::*;
use crate::gl;

use std::slice::SliceIndex;

#[derive(Clone, Copy)]
pub struct Slice<'a, T:?Sized, A:BufferAccess> {
    ptr: *const T,
    offset: usize,
    buf: PhantomData<&'a Buffer<T, A>>
}

pub struct SliceMut<'a, T:?Sized, A:BufferAccess> {
    pub(super) ptr: *mut T,
    offset: usize,
    buf: PhantomData<&'a mut Buffer<T, A>>
}

impl<'a,U:?Sized,T:?Sized+Unsize<U>,A:BufferAccess> CoerceUnsized<Slice<'a,U,A>> for Slice<'a,T,A> {}
impl<'a,U:?Sized,T:?Sized+Unsize<U>,A:BufferAccess> CoerceUnsized<SliceMut<'a,U,A>> for SliceMut<'a,T,A> {}

impl<'a,T:?Sized,A:BufferAccess> !Sync for Slice<'a,T,A> {}
impl<'a,T:?Sized,A:BufferAccess> !Send for Slice<'a,T,A> {}
impl<'a,T:?Sized,A:BufferAccess> !Sync for SliceMut<'a,T,A> {}
impl<'a,T:?Sized,A:BufferAccess> !Send for SliceMut<'a,T,A> {}

//
//Constructing slices
//

impl<'a, T:?Sized, A:BufferAccess> From<SliceMut<'a,T,A>> for Slice<'a,T,A> {
    #[inline] fn from(bmut: SliceMut<'a,T,A>) -> Self {Slice{ptr: bmut.ptr, offset: bmut.offset, buf:PhantomData}}
}

impl<'a, T:?Sized, A:BufferAccess> From<&'a Buffer<T,A>> for Slice<'a,T,A> {
    #[inline] fn from(bref: &'a Buffer<T,A>) -> Self {Slice{ptr: bref.ptr, offset: 0, buf:PhantomData}}
}

impl<'a, T:?Sized, A:BufferAccess> From<&'a mut Buffer<T,A>> for Slice<'a,T,A> {
    #[inline] fn from(bref: &'a mut Buffer<T,A>) -> Self {Slice{ptr: bref.ptr, offset: 0, buf:PhantomData}}
}

impl<'a, T:?Sized, A:BufferAccess> From<&'a mut Buffer<T,A>> for SliceMut<'a,T,A> {
    #[inline] fn from(bref: &'a mut Buffer<T,A>) -> Self {SliceMut{ptr: bref.ptr, offset: 0, buf:PhantomData}}
}

impl<'a,T:Sized,A:BufferAccess> Slice<'a,[T],A> {
    #[inline] pub fn len(&self) -> usize {self.map_buf(|buf| buf.len())}

    pub fn index<U:?Sized,I:SliceIndex<[T],Output=U>>(&self,i:I) -> Slice<'a,U,A> {
        unsafe {
            let null_ptr = {
                let mut raw = BufPtr{rust:self.ptr};
                raw.c = null();
                &*raw.rust
            };
            let indexed = &null_ptr[i];

            Slice {
                ptr: BufPtr{rust:indexed}.rust_mut,
                offset: self.offset + BufPtr{rust:indexed}.c.offset_from(null()) as usize,
                buf: PhantomData
            }
        }
    }

}

impl<'a,T:Sized,A:BufferAccess> SliceMut<'a,[T],A> {
    #[inline] pub fn len(&self) -> usize {self.as_immut().len()}

    #[inline]
    pub fn index<U:?Sized,I:SliceIndex<[T],Output=U>>(&self,i:I) -> Slice<'a,U,A> {
        self.as_immut().index(i)
    }

    pub fn index_mut<U:?Sized,I:SliceIndex<[T],Output=U>>(&mut self,i:I) -> SliceMut<'a,U,A> {
        unsafe {
            let null_ptr = {
                let mut raw = BufPtr{rust_mut:self.ptr};
                raw.c_mut = null_mut();
                &mut *raw.rust_mut
            };
            let indexed = &mut null_ptr[i];

            SliceMut {
                ptr: BufPtr{rust_mut: indexed}.rust_mut,
                offset: self.offset + BufPtr{rust_mut:indexed}.c_mut.offset_from(null_mut()) as usize,
                buf: PhantomData
            }
        }
    }
}


//TODO: splitting


//
//Basic methods
//

impl<'a,T:?Sized,A:BufferAccess> Slice<'a,T,A> {

    #[inline]
    fn map_buf<F:for<'b> FnOnce(&'b Buffer<T,A>)->U, U>(&self, f:F) -> U {
        let buf = Buffer{ptr: self.ptr as *mut T, access:PhantomData};
        let res = f(&buf);
        forget(buf);
        res
    }

    #[inline] pub fn id(&self) -> GLuint {self.map_buf(|buf| buf.id())}
    #[inline] pub fn size(&self) -> usize {self.map_buf(|buf| buf.size())}
    #[inline] pub fn align(&self) -> usize {self.map_buf(|buf| buf.align())}
    #[inline] pub fn offset(&self) -> usize {self.offset}

    #[inline] pub(super) unsafe fn _read_into_box(&self) -> Box<T> {
        map_alloc(self.ptr, |ptr| self.get_subdata_raw(ptr))
    }

    pub unsafe fn get_subdata_raw(&self, data: *mut T) {
        if gl::GetNamedBufferSubData::is_loaded() {
            gl::GetNamedBufferSubData(
                self.id(), self.offset() as GLintptr, self.size() as GLintptr, data as *mut GLvoid
            );
        } else {
            let mut target = BufferTarget::CopyReadBuffer.as_loc();
            gl::GetBufferSubData(
                target.bind_slice(self).target_id(),
                self.offset() as GLintptr,
                self.size() as GLintptr,
                data as *mut GLvoid
            );
        }
    }

}

impl<'a,T:?Sized,A:BufferAccess> SliceMut<'a,T,A> {

    #[inline] pub fn id(&self) -> GLuint {self.as_immut().id()}
    #[inline] pub fn size(&self) -> usize {self.as_immut().size()}
    #[inline] pub fn align(&self) -> usize {self.as_immut().align()}
    #[inline] pub fn offset(&self) -> usize {self.offset}

    #[inline] pub unsafe fn get_subdata_raw(&self, data: *mut T) { self.as_immut().get_subdata_raw(data) }

    #[inline] pub fn as_immut(&self) -> Slice<'a,T,A> {
        Slice{ptr:self.ptr, offset:self.offset, buf:PhantomData}
    }
}

//
//Reading subdata: glGetBufferSubData
//

impl<'a,T:GPUCopy+?Sized,A:BufferAccess> Slice<'a,T,A> {

    pub fn get_subdata_ref(&self, data: &mut T) {
        if size_of_val(data) != self.size() {
            panic!("Destination size not equal to source size: {} != {}", size_of_val(data), self.size())
        }
        unsafe {self.get_subdata_raw(data)}
    }

    pub fn get_subdata_box(&self) -> Box<T> {
        unsafe { self._read_into_box() }
    }
}

impl<'a,T:GPUCopy+Sized, A:BufferAccess> Slice<'a,T,A> {
    pub fn get_subdata(&self) -> T {
        unsafe {
            let mut data = MaybeUninit::uninit();
            self.get_subdata_raw(data.as_mut_ptr());
            data.assume_init()
        }
    }
}

impl<'a,T:GPUCopy+?Sized,A:BufferAccess> SliceMut<'a,T,A> {
    #[inline] pub fn get_subdata_ref(&self, data: &mut T) {self.as_immut().get_subdata_ref(data)}
    #[inline] pub fn get_subdata_box(&self) -> Box<T> {self.as_immut().get_subdata_box()}
}

impl<'a,T:GPUCopy+Sized,A:BufferAccess> SliceMut<'a,T,A> {
    #[inline] pub fn get_subdata(&self) -> T {self.as_immut().get_subdata()}
}



//
//Writing subdata: glBufferSubData
//

impl<'a, T:?Sized, A:WriteAccess> SliceMut<'a,T,A> {
    pub unsafe fn subdata_raw(&mut self, data: *const T) {
        if gl::NamedBufferSubData::is_loaded() {
            gl::NamedBufferSubData(
                self.id(), self.offset as GLintptr, self.size() as GLsizeiptr, BufPtr{rust:data}.gl
            );
        } else {
            let mut target = BufferTarget::CopyWriteBuffer.as_loc();
            gl::BufferSubData(
                target.bind_slice_mut(self).target_id(),
                self.offset as GLintptr,
                self.size() as GLsizeiptr,
                BufPtr{rust:data}.gl
            );
        }

    }
}

impl<'a,T:Sized,A:WriteAccess> SliceMut<'a,T,A> {
    #[inline]
    pub fn subdata(&mut self, data: T) {
        unsafe {
            if needs_drop::<T>() {
                //we need to make sure that the destructor on the data is run if it is a Drop type
                drop(self.replace(data));
            } else {
                //else, we can just overwrite the data
                self.subdata_raw(&data);
                forget(data); //note, we need to make sure the destructor of data is NOT run
            }
        }
    }

    #[inline]
    pub fn replace(&mut self, data: T) -> T {
        unsafe {
            //read the buffer data into a temporary variable
            let mut old_data = MaybeUninit::<T>::uninit();
            self.get_subdata_raw(old_data.as_mut_ptr());

            //modify the buffer
            self.subdata_raw(&data);
            forget(data); //note, we need to make sure the destructor of data is NOT run

            return old_data.assume_init();
        }
    }
}

impl<'a,T:Sized,A:WriteAccess> SliceMut<'a,[T],A> {
    #[inline]
    pub fn replace_range(&mut self, data: &mut [T]) {
        unsafe {
            //check bounds
            assert_eq!(data.len(), self.len(), "destination and source have different lengths");

            //read the buffer data into a Box
            let temp_data = self.as_immut()._read_into_box();

            //upload new data to buffer
            self.subdata_raw(data);

            //deallocate the temp-box and copy to the destination slice
            map_dealloc(temp_data, |ptr| copy_nonoverlapping((&*ptr).as_ptr(), data.as_mut_ptr(), data.len()))
        }
    }
}

impl<'a,T:GPUCopy+?Sized,A:WriteAccess> SliceMut<'a,T,A> {
    #[inline]
    pub fn subdata_ref(&mut self, data: &T) {
        assert_eq!(self.size(), size_of_val(data), "destination and source have different lengths");//check bounds
        unsafe {self.subdata_raw(data)}
    }
}

//
//Copying data between buffers: glCopyBufferSubData
//

impl<'a,T:?Sized,A:BufferAccess> Slice<'a,T,A> {
    pub unsafe fn copy_subdata_unchecked(&self, dest:&mut SliceMut<'a,T,A>) {
        let mut t1 = BufferTarget::CopyReadBuffer.as_loc();
        let mut t2 = BufferTarget::CopyWriteBuffer.as_loc();
        gl::CopyBufferSubData(
            t1.bind_slice(self).target_id(),
            t2.bind_slice_mut(dest).target_id(),
            self.offset as GLintptr,
            dest.offset as GLintptr,
            self.size() as GLsizeiptr
        );
    }
}

impl<'a,T:GPUCopy+?Sized,A:BufferAccess> Slice<'a,T,A> {
    #[inline] pub fn copy_subdata(&self, dest:&mut SliceMut<'a,T,A>) {
        assert_eq!(dest.size(), self.size(), "destination and source buffers have different sizes");
        unsafe{ self.copy_subdata_unchecked(dest) }
    }
}
