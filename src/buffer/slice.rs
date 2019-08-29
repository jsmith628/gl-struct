use super::*;
use crate::gl;

use std::convert::TryInto;
use std::slice::SliceIndex;

#[derive(Clone, Copy)]
pub struct BSlice<'a, T:?Sized, A:BufferAccess> {
    ptr: *const T,
    offset: usize,
    buf: PhantomData<&'a Buf<T, A>>
}

pub struct BSliceMut<'a, T:?Sized, A:BufferAccess> {
    pub(super) ptr: *mut T,
    offset: usize,
    buf: PhantomData<&'a Buf<T, A>>
}

impl<'a,U:?Sized,T:?Sized+Unsize<U>,A:BufferAccess> CoerceUnsized<BSlice<'a,U,A>> for BSlice<'a,T,A> {}
impl<'a,U:?Sized,T:?Sized+Unsize<U>,A:BufferAccess> CoerceUnsized<BSliceMut<'a,U,A>> for BSliceMut<'a,T,A> {}

impl<'a,T:?Sized,A:BufferAccess> !Sync for BSlice<'a,T,A> {}
impl<'a,T:?Sized,A:BufferAccess> !Send for BSlice<'a,T,A> {}
impl<'a,T:?Sized,A:BufferAccess> !Sync for BSliceMut<'a,T,A> {}
impl<'a,T:?Sized,A:BufferAccess> !Send for BSliceMut<'a,T,A> {}

//
//Constructing slices
//

impl<'a, T:?Sized, A:BufferAccess> From<BSliceMut<'a,T,A>> for BSlice<'a,T,A> {
    #[inline] fn from(bmut: BSliceMut<'a,T,A>) -> Self {BSlice{ptr: bmut.ptr, offset: bmut.offset, buf:PhantomData}}
}

impl<'a, T:?Sized, A:BufferAccess> From<&'a Buf<T,A>> for BSlice<'a,T,A> {
    #[inline] fn from(bref: &'a Buf<T,A>) -> Self {BSlice{ptr: bref.ptr, offset: 0, buf:PhantomData}}
}

impl<'a, T:?Sized, A:BufferAccess> From<&'a mut Buf<T,A>> for BSliceMut<'a,T,A> {
    #[inline] fn from(bref: &'a mut Buf<T,A>) -> Self {BSliceMut{ptr: bref.ptr, offset: 0, buf:PhantomData}}
}

impl<T:?Sized,A:BufferAccess> Buf<T,A> {
    #[inline] pub fn as_slice(&self) -> BSlice<T,A> {BSlice::from(self)}
    #[inline] pub fn as_slice_mut(&mut self) -> BSliceMut<T,A> {BSliceMut::from(self)}
}

impl<'a,T:Sized,A:BufferAccess> BSlice<'a,[T],A> {
    pub fn index<U:?Sized,I:SliceIndex<[T],Output=U>>(&self,i:I) -> BSlice<'a,U,A> {
        unsafe {
            let null_ptr = {
                let mut raw = BufPtr{rust:self.ptr};
                raw.c = null();
                &*raw.rust
            };
            let indexed = &null_ptr[i];

            BSlice {
                ptr: BufPtr{rust:indexed}.rust_mut,
                offset: self.offset + BufPtr{rust:indexed}.c.offset_from(null()) as usize,
                buf: PhantomData
            }
        }
    }

}

impl<'a,T:Sized,A:BufferAccess> BSliceMut<'a,[T],A> {
    #[inline]
    pub fn index<U:?Sized,I:SliceIndex<[T],Output=U>>(&self,i:I) -> BSlice<'a,U,A> {
        self.as_immut().index(i)
    }

    pub fn index_mut<U:?Sized,I:SliceIndex<[T],Output=U>>(&mut self,i:I) -> BSliceMut<'a,U,A> {
        unsafe {
            let null_ptr = {
                let mut raw = BufPtr{rust_mut:self.ptr};
                raw.c = null();
                &mut *raw.rust_mut
            };
            let indexed = &mut null_ptr[i];

            BSliceMut {
                ptr: BufPtr{rust_mut: indexed}.rust_mut,
                offset: self.offset + BufPtr{rust_mut:indexed}.c.offset_from(null()) as usize,
                buf: PhantomData
            }
        }
    }
}


//TODO: splitting


//
//Basic methods
//

impl<'a,T:?Sized,A:BufferAccess> BSliceMut<'a,T,A> {

    #[inline] fn map_buf<F:for<'b> FnOnce(&'b Buf<T,A>)->U, U>(&self, f:F) -> U {self.as_immut().map_buf(f)}

    #[inline] pub fn id(&self) -> GLuint {self.map_buf(|buf| buf.id())}
    #[inline] pub fn size(&self) -> usize {self.map_buf(|buf| buf.size())}
    #[inline] pub fn align(&self) -> usize {self.map_buf(|buf| buf.align())}
    #[inline] pub fn offset(&self) -> usize {self.offset}

    #[inline] pub unsafe fn get_subdata_raw(&self, data: *mut T) { self.as_immut().get_subdata_raw(data) }

    #[inline] pub fn as_immut(&self) -> BSlice<'a,T,A> {
        BSlice{ptr:self.ptr, offset:self.offset, buf:PhantomData}
    }
}

impl<'a,T:Sized,A:BufferAccess> BSliceMut<'a,[T],A> {
    #[inline] pub fn len(&self) -> usize {self.map_buf(|buf| buf.len())}
}

impl<'a,T:?Sized,A:BufferAccess> BSlice<'a,T,A> {

    #[inline]
    fn map_buf<F:for<'b> FnOnce(&'b Buf<T,A>)->U, U>(&self, f:F) -> U {
        let buf = Buf{ptr: self.ptr as *mut T, access:PhantomData};
        let res = f(&buf);
        forget(buf);
        res
    }

    #[inline] pub fn id(&self) -> GLuint {self.map_buf(|buf| buf.id())}
    #[inline] pub fn size(&self) -> usize {self.map_buf(|buf| buf.size())}
    #[inline] pub fn align(&self) -> usize {self.map_buf(|buf| buf.align())}
    #[inline] pub fn offset(&self) -> usize {self.offset}

    pub unsafe fn get_subdata_raw(&self, data: *mut T) {
        let mut target = BufferTarget::CopyReadBuffer.as_loc();
        gl::GetBufferSubData(
            target.bind_slice(self).target_id(),
            self.offset as GLintptr,
            size_of_val(&*data) as GLsizeiptr,
            BufPtr{rust_mut: data}.gl_mut
        );
    }

    pub(super) unsafe fn _into_box(&self) -> Box<T> {
        //Manually allocate a pointer on the head that we will store the data in
        let data = Global.alloc(::std::alloc::Layout::for_value(&*self.ptr)).unwrap().as_ptr();

        //next, construct a *mut T pointer from the u8 pointer we just allocated using the metadata
        //stored in this buf
        let mut ptr = BufPtr{ rust: self.ptr };
        ptr.c_mut = data;

        //next, copy the data into the newly created heap-pointer
        self.get_subdata_raw(ptr.rust_mut);

        //finally, make and return a Box to own the heap data
        Box::from_raw(ptr.rust_mut)
    }

}

impl<'a,T:Sized,A:BufferAccess> BSlice<'a,[T],A> {
    #[inline] pub fn len(&self) -> usize {self.map_buf(|buf| buf.len())}
}

//
//Reading subdata
//

impl<'a,T:GPUCopy+?Sized,A:BufferAccess> BSlice<'a,T,A> {
    #[inline] pub fn get_subdata_ref(&self, data: &mut T) {
        if size_of_val(data) != self.size() {
            panic!("Destination size not equal to source size: {} != {}", size_of_val(data), self.size())
        }
        unsafe {self.get_subdata_raw(data)}
    }
    #[inline] pub fn get_subdata_box(&self) -> Box<T> { unsafe { self._into_box() } }
}

impl<'a,T:GPUCopy+Sized, A:BufferAccess> BSlice<'a,T,A> {
    #[inline] pub fn get_subdata(&self) -> T {
        unsafe {
            let mut data = uninitialized();
            self.get_subdata_raw(&mut data as *mut T);
            data
        }
    }
}

impl<'a,T:GPUCopy+?Sized,A:BufferAccess> BSliceMut<'a,T,A> {
    #[inline] pub fn get_subdata_ref(&self, data: &mut T) {self.as_immut().get_subdata_ref(data)}
    #[inline] pub fn get_subdata_box(&self) -> Box<T> {self.as_immut().get_subdata_box()}
}

impl<'a,T:GPUCopy+Sized,A:BufferAccess> BSliceMut<'a,T,A> {
    #[inline] pub fn get_subdata(&self) -> T {self.as_immut().get_subdata()}
}



//
//Writing subdata
//

impl<'a, T:?Sized, A:WriteAccess> BSliceMut<'a,T,A> {
    pub unsafe fn subdata_raw(&mut self, data: *const T) {
        let mut target = BufferTarget::CopyWriteBuffer.as_loc();
        gl::BufferSubData(
            target.bind_slice_mut(self).target_id(),
            self.offset as GLintptr,
            self.size() as GLsizeiptr,
            BufPtr{rust:data}.gl
        );
    }
}

impl<'a,T:Sized,A:WriteAccess> BSliceMut<'a,T,A> {
    #[inline]
    pub fn subdata(&mut self, data: T) {
        unsafe {
            if data.needs_drop() {
                //we need to make sure that the destructor on the data is run if it is a Drop type
                drop(self.replace_subdata(data));
            } else {
                //else, we can just overwrite the data
                self.subdata_raw(&data);
                forget(data); //note, we need to make sure the destructor of data is NOT run
            }
        }
    }

    #[inline]
    pub fn replace_subdata(&mut self, data: T) -> T {
        unsafe {
            let mut old_data = uninitialized::<T>();
            self.get_subdata_raw(&mut old_data);
            self.subdata_raw(&data);
            forget(data); //note, we need to make sure the destructor of data is NOT run
            return old_data;
        }
    }
}

impl<'a,T:Sized,A:WriteAccess> BSliceMut<'a,[T],A> {
    #[inline]
    pub fn replace_subdata_range(&mut self, data: &mut [T]) {
        unsafe {
            let mut temp_data = self.as_immut()._into_box();
            data.swap_with_slice(&mut temp_data);
            self.subdata_raw(&*temp_data);

            //now, deallocate the box
            let layout = Layout::for_value(&*temp_data);
            Global.dealloc(
                NonNull::new_unchecked(BufPtr{rust_mut: Box::into_raw(temp_data)}.c_mut),
                layout
            )
        }
    }
}

impl<'a,T:GPUCopy+?Sized,A:WriteAccess> BSliceMut<'a,T,A> {
    #[inline]
    pub fn subdata_ref(&mut self, data: &T) {
        if self.size() != size_of_val(data) {
            panic!("Destination size not equal to source size: {} != {}", self.size(), size_of_val(data))
        }
        unsafe {self.subdata_raw(data)}
    }
}

//
//Copying data between buffers
//

impl<'a,T:?Sized,A:BufferAccess> BSlice<'a,T,A> {
    pub unsafe fn copy_subdata_raw(&self, other:&mut BSliceMut<'a,T,A>) {
        let mut t1 = BufferTarget::CopyReadBuffer.as_loc();
        let mut t2 = BufferTarget::CopyWriteBuffer.as_loc();
        gl::CopyBufferSubData(
            t1.bind_slice(self).target_id(),
            t2.bind_slice_mut(other).target_id(),
            self.offset as GLintptr,
            other.offset as GLintptr,
            self.size() as GLsizeiptr
        );
    }
}

impl<'a,T:GPUCopy+?Sized,A:BufferAccess> BSlice<'a,T,A> {
    #[inline] pub fn copy_subdata(&self, other:&mut BSliceMut<'a,T,A>) {
        if other.size() != self.size() {
            panic!("Destination size not equal to source size: {} < {}", other.size(), self.size())
        }
        unsafe{self.copy_subdata_raw(other)}
    }
}

//
//Texture data transfer
//

use image_format::{ClientFormat,PixelType,PixelData,PixelDataMut,PixelRowAlignment};

unsafe impl<'a,F:ClientFormat,T:PixelType<F>,A:BufferAccess> PixelData<F> for BSlice<'a,[T],A> {
    #[inline] fn swap_bytes(&self) -> bool {T::swap_bytes()}
    #[inline] fn lsb_first(&self) -> bool {T::lsb_first()}

    #[inline] fn alignment(&self) -> PixelRowAlignment { (align_of::<T>().min(8) as u8).try_into().unwrap() }

    #[inline] fn format_type(&self) -> F {T::format_type()}
    #[inline] fn count(&self) -> usize {BSlice::len(self)}
    #[inline] fn size(&self) -> usize {BSlice::size(self)}

    #[inline] fn pixels<'b>(
        &'b self, target:&'b mut BindingLocation<RawBuffer>
    ) -> (Option<Binding<'b,RawBuffer>>, *const GLvoid) {
        (Some(target.bind_slice(self)), self.offset as *const GLvoid)
    }
}

unsafe impl<'a,F:ClientFormat,T:PixelType<F>,A:BufferAccess> PixelData<F> for BSliceMut<'a,[T],A> {
    #[inline] fn swap_bytes(&self) -> bool {T::swap_bytes()}
    #[inline] fn lsb_first(&self) -> bool {T::lsb_first()}

    #[inline] fn alignment(&self) -> PixelRowAlignment { (align_of::<T>().min(8) as u8).try_into().unwrap() }

    #[inline] fn format_type(&self) -> F {T::format_type()}
    #[inline] fn count(&self) -> usize {BSliceMut::len(self)}
    #[inline] fn size(&self) -> usize {BSliceMut::size(self)}

    #[inline] fn pixels<'b>(
        &'b self, target:&'b mut BindingLocation<RawBuffer>
    ) -> (Option<Binding<'b,RawBuffer>>, *const GLvoid) {
        (Some(target.bind_slice_mut(self)), self.offset as *const GLvoid)
    }
}

unsafe impl<'a,F:ClientFormat,T:PixelType<F>,A:BufferAccess> PixelDataMut<F> for BSliceMut<'a,[T],A> {
    #[inline] fn pixels_mut<'b>(
        &'b mut self, target:&'b mut BindingLocation<RawBuffer>
    ) -> (Option<Binding<'b,RawBuffer>>, *mut GLvoid) {
        (Some(target.bind_slice_mut(self)), self.offset as *mut GLvoid)
    }
}
