use super::*;
use crate::gl;

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub struct Slice<'a, T:?Sized, A:Initialized> {
    pub(super) ptr: BufPtr<T>,
    pub(super) offset: usize,
    pub(super) buf: PhantomData<&'a Buffer<T, A>>
}

pub struct SliceMut<'a, T:?Sized, A:Initialized> {
    pub(super) ptr: BufPtr<T>,
    pub(super) offset: usize,
    pub(super) buf: PhantomData<&'a mut Buffer<T, A>>
}

impl<'a,U:?Sized,T:?Sized+Unsize<U>,A:Initialized> CoerceUnsized<Slice<'a,U,A>> for Slice<'a,T,A> {}
impl<'a,U:?Sized,T:?Sized+Unsize<U>,A:Initialized> CoerceUnsized<SliceMut<'a,U,A>> for SliceMut<'a,T,A> {}

//
//Constructing slices
//

impl<'a, 'b:'a, T:?Sized, A:Initialized> From<&'a Slice<'b,T,A>> for Slice<'b,T,A> {
    #[inline] fn from(bmut: &'a Slice<'b,T,A>) -> Self { *bmut }
}

impl<'a, T:?Sized, A:Initialized> From<SliceMut<'a,T,A>> for Slice<'a,T,A> {
    #[inline] fn from(bmut: SliceMut<'a,T,A>) -> Self {Slice{ptr: bmut.ptr, offset: bmut.offset, buf:PhantomData}}
}

impl<'a, 'b:'a, T:?Sized, A:Initialized> From<&'a SliceMut<'b,T,A>> for Slice<'a,T,A> {
    #[inline] fn from(bmut: &'a SliceMut<'b,T,A>) -> Self {
        Slice{ptr: bmut.ptr, offset: bmut.offset, buf:PhantomData}
    }
}

impl<'a, 'b:'a, T:?Sized, A:Initialized> From<&'a mut SliceMut<'b,T,A>> for SliceMut<'a,T,A> {
    #[inline] fn from(bmut: &'a mut SliceMut<'b,T,A>) -> Self {
        SliceMut{ptr: bmut.ptr, offset: bmut.offset, buf:PhantomData}
    }
}

impl<'a, T:?Sized, A:Initialized> From<&'a Buffer<T,A>> for Slice<'a,T,A> {
    #[inline] fn from(bref: &'a Buffer<T,A>) -> Self {Slice{ptr: bref.ptr, offset: 0, buf:PhantomData}}
}

impl<'a, T:?Sized, A:Initialized> From<&'a mut Buffer<T,A>> for SliceMut<'a,T,A> {
    #[inline] fn from(bref: &'a mut Buffer<T,A>) -> Self {SliceMut{ptr: bref.ptr, offset: 0, buf:PhantomData}}
}

impl<'a,T:?Sized,A:Initialized> Slice<'a,T,A> {

    #[inline]
    unsafe fn into_mut(self) -> SliceMut<'a,T,A> {
        SliceMut{ ptr:self.ptr, offset:self.offset, buf:PhantomData }
    }

    #[inline] pub fn id(&self) -> GLuint {self.ptr.id()}
    #[inline] pub fn size(&self) -> usize {self.ptr.size()}
    #[inline] pub fn align(&self) -> usize {self.ptr.align()}
    #[inline] pub fn offset(&self) -> usize {self.offset}

    #[inline] pub unsafe fn downgrade_unchecked<B:Initialized>(self) -> Slice<'a,T,B> {
        Slice{ptr: self.ptr, offset: self.offset, buf: PhantomData}
    }

    #[inline]
    pub fn downgrade<B:Initialized>(self) -> Slice<'a,T,B> where A:DowngradesTo<B> {
        unsafe { self.downgrade_unchecked() }
    }

    #[inline] pub(super) unsafe fn _read_into_box(&self) -> Box<T> {
        map_alloc(self.ptr, |ptr| self.get_subdata_raw(ptr))
    }

    pub unsafe fn get_subdata_raw(&self, data: *mut T) {
        if self.size()==0 { return; }
        if gl::GetNamedBufferSubData::is_loaded() {
            gl::GetNamedBufferSubData(
                self.id(), self.offset() as GLintptr, self.size() as GLintptr, data as *mut GLvoid
            );
        } else {
            BufferTarget::CopyReadBuffer.as_loc().map_bind(self, |b|
                gl::GetBufferSubData(
                    b.target_id(),
                    self.offset() as GLintptr,
                    self.size() as GLintptr,
                    data as *mut GLvoid
                )
            );
        }
    }

    pub unsafe fn copy_subdata_unchecked<'b>(&self, dest: &mut SliceMut<'b,T,A>) {
        if self.size()==0 || dest.size()==0 { return; }
        BufferTarget::CopyReadBuffer.as_loc().map_bind(self, |b1|
            BufferTarget::CopyWriteBuffer.as_loc().map_bind(dest, |b2|
                gl::CopyBufferSubData(
                    b1.target_id(), b2.target_id(),
                    self.offset as GLintptr, dest.offset as GLintptr,
                    self.size() as GLsizeiptr
                )
            )
        )
    }

    pub fn get_subdata_box(&self) -> Box<T> where T:GPUCopy  {
        unsafe { self._read_into_box() }
    }

    pub fn get_subdata(&self) -> T where T:Copy {
        unsafe {
            let mut data = MaybeUninit::uninit();
            self.get_subdata_raw(data.as_mut_ptr());
            data.assume_init()
        }
    }

}

impl<'a,T:Copy+Sized,A:Initialized> Slice<'a,T,A> {
    #[inline]
    pub fn copy_subdata<'b>(&self, dest:&mut SliceMut<'b,T,A>) {
        unsafe{ self.copy_subdata_unchecked(dest) }
    }
}

impl<'a,T:Copy+Sized,A:Initialized> Slice<'a,[T],A> {
    #[inline]
    pub fn copy_subdata<'b>(&self, dest:&mut SliceMut<'b,[T],A>) {
        assert_eq!(dest.size(), self.size(), "destination and source buffers have different sizes");
        unsafe{ self.copy_subdata_unchecked(dest) }
    }
}

impl<'a,T:Sized,A:Initialized> Slice<'a,[T],A> {
    #[inline] pub fn len(&self) -> usize {self.ptr.len()}

    #[inline]
    pub unsafe fn from_raw_parts(id:GLuint, len:usize, offset:usize) -> Self {
        Slice{ptr: BufPtr::from_raw_parts(id, len), offset: offset, buf:PhantomData}
    }

    pub fn split_at(self, mid:usize) -> (Slice<'a,[T],A>, Slice<'a,[T],A>) {
        assert!(mid<=self.len(), "Split midpoint larger than slice length");
        unsafe {
            (
                Self::from_raw_parts(self.id(), mid, self.offset),
                Self::from_raw_parts(self.id(), self.len() - mid, self.offset + mid*size_of::<T>())
            )
        }
    }

    pub fn split_first(self) -> Option<(Slice<'a,T,A>, Slice<'a,[T],A>)> {
        match self.len()==0 {
            true => None,
            _ => {
                let (first, rest) = self.split_at(1);
                Some((first.index(0), rest))
            }
        }
    }

    pub fn split_last(self) -> Option<(Slice<'a,T,A>, Slice<'a,[T],A>)> {
        match self.len()==0 {
            true => None,
            _ => {
                let (rest, last) = self.split_at(self.len()-1);
                Some((last.index(0), rest))
            }
        }
    }

    pub fn index<U:?Sized,I:SliceIndex<[T],Output=U>>(self,i:I) -> Slice<'a,U,A> {
        unsafe {
            let dangling_slice = from_raw_parts(NonNull::dangling().as_ptr(), self.len());
            let indexed = &dangling_slice[i];

            let offset = (indexed as *const U as *const u8).offset_from(dangling_slice.as_ptr() as *const u8);

            Slice {
                ptr: BufPtr::new(self.id(), indexed as *const U as *mut U),
                offset: self.offset + offset as usize,
                buf: PhantomData
            }
        }
    }

    #[inline]
    pub fn into_attribs(self) -> AttribArray<'a,T::GLSL> where T:AttribData {
        self.into()
    }

    #[inline]
    pub fn split_attribs(self) -> T::AttribArrays where T:SplitAttribs<'a> {
        T::split_buffer(self)
    }

    #[inline]
    pub fn get_subdata_slice(&self, data: &mut [T]) where T:Copy {
        if size_of_val(data) != self.size() {
            panic!("Destination size not equal to source size: {} != {}", size_of_val(data), self.size())
        }
        unsafe {self.get_subdata_raw(data)}
    }

}

impl<'a,F:SpecificCompressed, A:Initialized> Slice<'a,CompressedPixels<F>,A> {
    #[inline] pub fn blocks(&self) -> usize { self.ptr.blocks() }
    #[inline] pub fn pixel_count(&self) -> usize { self.ptr.pixel_count() }
}

impl<'a,T:?Sized,A:Initialized> SliceMut<'a,T,A> {

    #[inline] pub fn id(&self) -> GLuint {self.ptr.id()}
    #[inline] pub fn size(&self) -> usize {self.ptr.size()}
    #[inline] pub fn align(&self) -> usize {self.ptr.align()}
    #[inline] pub fn offset(&self) -> usize {self.offset}

    #[inline] pub fn as_immut(&self) -> Slice<T,A> { Slice::from(self) }

    #[inline] pub fn as_mut(&mut self) -> SliceMut<T,A> { SliceMut::from(self) }

    #[inline] pub unsafe fn downgrade_unchecked<B:Initialized>(self) -> SliceMut<'a,T,B> {
        SliceMut{ptr: self.ptr, offset: self.offset, buf: PhantomData}
    }

    #[inline]
    pub fn downgrade<B:Initialized>(self) -> SliceMut<'a,T,B> where A:DowngradesTo<B> {
        unsafe { self.downgrade_unchecked() }
    }

    #[inline] pub unsafe fn get_subdata_raw(&self, data: *mut T) { self.as_immut().get_subdata_raw(data) }

    #[inline] pub unsafe fn copy_subdata_unchecked<'b>(&self, dest: &mut SliceMut<'b,T,A>) {
        self.as_immut().copy_subdata_unchecked(dest)
    }

    pub unsafe fn invalidate_subdata_raw(&mut self) {
        if self.size()==0 { return; }
        if gl::InvalidateBufferSubData::is_loaded() {
            gl::InvalidateBufferSubData(self.id(), self.offset() as GLintptr, self.size() as GLsizeiptr)
        }
    }

    #[inline] pub fn get_subdata_box(&self) -> Box<T> where T:GPUCopy {self.as_immut().get_subdata_box()}
    #[inline] pub fn get_subdata(&self) -> T where T:Copy+Sized {self.as_immut().get_subdata()}

}

impl<'a,T:Copy+Sized,A:Initialized> SliceMut<'a,[T],A> {
    #[inline] pub fn copy_subdata<'b>(&self, dest:&mut SliceMut<'b,[T],A>) { self.as_immut().copy_subdata(dest) }
}

impl<'a,T:Copy+Sized,A:Initialized> SliceMut<'a,T,A> {
    #[inline] pub fn copy_subdata<'b>(&self, dest:&mut SliceMut<'b,T,A>) { self.as_immut().copy_subdata(dest) }
}

impl<'a,T:Sized,A:Initialized> SliceMut<'a,[T],A> {
    #[inline] pub fn len(&self) -> usize {self.as_immut().len()}

    #[inline]
    pub unsafe fn from_raw_parts(id:GLuint, len:usize, offset:usize) -> Self {
        SliceMut{ptr: BufPtr::from_raw_parts(id, len), offset: offset, buf:PhantomData}
    }

    #[inline] pub fn split_at(self, mid:usize) -> (Slice<'a,[T],A>, Slice<'a,[T],A>) {
        Slice::from(self).split_at(mid)
    }

    #[inline] pub fn split_at_mut(self, mid:usize) -> (SliceMut<'a,[T],A>, SliceMut<'a,[T],A>) {
        unsafe {
            let (s1, s2) = self.split_at(mid);
            (s1.into_mut(), s2.into_mut())
        }
    }

    #[inline] pub fn split_first(self) -> Option<(Slice<'a,T,A>, Slice<'a,[T],A>)> {
        Slice::from(self).split_first()
    }

    #[inline] pub fn split_first_mut(self) -> Option<(SliceMut<'a,T,A>, SliceMut<'a,[T],A>)> {
        unsafe { self.split_first().map(|(s1,s2)| (s1.into_mut(), s2.into_mut())) }
    }

    #[inline] pub fn split_last(self) -> Option<(Slice<'a,T,A>, Slice<'a,[T],A>)> {
        Slice::from(self).split_last()
    }

    #[inline] pub fn split_last_mut(self) -> Option<(SliceMut<'a,T,A>, SliceMut<'a,[T],A>)> {
        unsafe { self.split_last().map(|(s1,s2)| (s1.into_mut(), s2.into_mut())) }
    }

    #[inline]
    pub fn index<U:?Sized,I:SliceIndex<[T],Output=U>>(self,i:I) -> Slice<'a,U,A> {
        Slice::from(self).index(i)
    }

    #[inline]
    pub fn index_mut<U:?Sized,I:SliceIndex<[T],Output=U>>(self,i:I) -> SliceMut<'a,U,A> {
        unsafe { Slice::from(self).index(i).into_mut() }
    }

    #[inline]
    pub fn into_attribs(self) -> AttribArray<'a,T::GLSL> where T:AttribData {
        Slice::from(self).into()
    }

    #[inline]
    pub fn split_attribs(self) -> T::AttribArrays where T:SplitAttribs<'a> {
        T::split_buffer(Slice::from(self))
    }

    #[inline]
    pub fn get_subdata_slice(&self, data: &mut [T]) where T:Copy {
        self.as_immut().get_subdata_slice(data)
    }

}

impl<'a,F:SpecificCompressed, A:Initialized> SliceMut<'a,CompressedPixels<F>,A> {
    #[inline] pub fn blocks(&self) -> usize { self.ptr.blocks() }
    #[inline] pub fn pixel_count(&self) -> usize { self.ptr.pixel_count() }
}

//
//Writing subdata: glBufferSubData
//

impl<'a, T:?Sized, A:Dynamic> SliceMut<'a,T,A> {
    pub unsafe fn subdata_raw(&mut self, data: *const T) {
        if self.size()==0 { return; }

        let void = data as *const GLvoid;
        let size = self.size().min(size_of_val(&*data)) as GLsizeiptr;

        if gl::NamedBufferSubData::is_loaded() {
            gl::NamedBufferSubData(self.id(), self.offset as GLintptr, size, void);
        } else {
            BufferTarget::CopyWriteBuffer.as_loc().map_bind(self,
                |b| gl::BufferSubData( b.target_id(), self.offset as GLintptr, size, void)
            );
        }
    }
}

impl<'a,T:Sized,A:Dynamic> SliceMut<'a,T,A> {

    pub fn subdata(&mut self, data: T) {
        unsafe {
            if needs_drop::<T>() {
                //we need to make sure that the destructor on the data is run if it is a Drop type
                drop(self.replace(data));
            } else {
                //else, we can just overwrite the data without dropping
                //in fact, we can even invalidate the buffer region since we don't need the data either
                self.invalidate_subdata_raw();
                self.subdata_raw(&data)
            }
        }
    }

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

impl<'a,T:Sized,A:Dynamic> SliceMut<'a,[T],A> {

    pub fn subdata(&mut self, mut data: Box<[T]>) {
        unsafe {
            if needs_drop::<T>() {
                //we need to make sure that the destructor on the data is run if it is a Drop type
                drop(self.replace(&mut *data));
            } else {
                //else, we can just overwrite the data without dropping
                //in fact, we can even invalidate the buffer region since we don't need the data either
                self.invalidate_subdata_raw();
                self.subdata_raw(&*data)
            }
        }
    }

    pub fn replace(&mut self, data: &mut [T]) {
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
