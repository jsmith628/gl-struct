use super::*;
use crate::gl;

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub struct Slice<'a, T:?Sized, A:BufferStorage> {
    pub(super) ptr: BufPtr<T>,
    pub(super) offset: usize,
    pub(super) buf: PhantomData<&'a Buffer<T, A>>
}

pub struct SliceMut<'a, T:?Sized, A:BufferStorage> {
    pub(super) ptr: BufPtr<T>,
    pub(super) offset: usize,
    pub(super) buf: PhantomData<&'a mut Buffer<T, A>>
}

impl<'a,U:?Sized,T:?Sized+Unsize<U>,A:BufferStorage> CoerceUnsized<Slice<'a,U,A>> for Slice<'a,T,A> {}
impl<'a,U:?Sized,T:?Sized+Unsize<U>,A:BufferStorage> CoerceUnsized<SliceMut<'a,U,A>> for SliceMut<'a,T,A> {}

//
//Constructing slices
//

impl<'a, 'b:'a, T:?Sized, A:BufferStorage> From<&'a Slice<'b,T,A>> for Slice<'b,T,A> {
    #[inline] fn from(bmut: &'a Slice<'b,T,A>) -> Self { *bmut }
}

impl<'a, T:?Sized, A:BufferStorage> From<SliceMut<'a,T,A>> for Slice<'a,T,A> {
    #[inline] fn from(bmut: SliceMut<'a,T,A>) -> Self {Slice{ptr: bmut.ptr, offset: bmut.offset, buf:PhantomData}}
}

impl<'a, 'b:'a, T:?Sized, A:BufferStorage> From<&'a SliceMut<'b,T,A>> for Slice<'a,T,A> {
    #[inline] fn from(bmut: &'a SliceMut<'b,T,A>) -> Self {
        Slice{ptr: bmut.ptr, offset: bmut.offset, buf:PhantomData}
    }
}

impl<'a, 'b:'a, T:?Sized, A:BufferStorage> From<&'a mut SliceMut<'b,T,A>> for SliceMut<'a,T,A> {
    #[inline] fn from(bmut: &'a mut SliceMut<'b,T,A>) -> Self {
        SliceMut{ptr: bmut.ptr, offset: bmut.offset, buf:PhantomData}
    }
}

impl<'a, T:?Sized, A:BufferStorage> From<&'a Buffer<T,A>> for Slice<'a,T,A> {
    #[inline] fn from(bref: &'a Buffer<T,A>) -> Self {Slice{ptr: bref.ptr, offset: 0, buf:PhantomData}}
}

impl<'a, T:?Sized, A:BufferStorage> From<&'a mut Buffer<T,A>> for SliceMut<'a,T,A> {
    #[inline] fn from(bref: &'a mut Buffer<T,A>) -> Self {SliceMut{ptr: bref.ptr, offset: 0, buf:PhantomData}}
}

impl<'a,T:?Sized,A:BufferStorage> Slice<'a,T,A> {

    #[inline]
    unsafe fn into_mut(self) -> SliceMut<'a,T,A> {
        SliceMut{ ptr:self.ptr, offset:self.offset, buf:PhantomData }
    }

    #[inline] pub fn id(&self) -> GLuint {self.ptr.id()}
    #[inline] pub fn size(&self) -> usize {self.ptr.size()}
    #[inline] pub fn align(&self) -> usize {self.ptr.align()}
    #[inline] pub fn offset(&self) -> usize {self.offset}

    #[inline] pub(crate) fn offset_ptr(&self) -> *const T {
        self.ptr.swap_offset(self.offset)
    }

    #[inline] pub fn as_slice(&self) -> Slice<T,A> { Slice::from(self) }

    #[inline] pub unsafe fn downgrade_unchecked<B:BufferStorage>(self) -> Slice<'a,T,B> {
        Slice{ptr: self.ptr, offset: self.offset, buf: PhantomData}
    }

    #[inline]
    pub fn downgrade<B:BufferStorage>(self) -> Slice<'a,T,B> where A:DowngradesTo<B> {
        unsafe { self.downgrade_unchecked() }
    }

    pub unsafe fn get_subdata_raw(&self, data: *mut T) {

        //for convenience
        let size = size_of_val(&*data) as GLintptr;

        //if we have a ZST, we can just return
        //also, since OpenGL buffers can't be zero-sized anyway, GetBufferData could error if we didn't
        if size==0 { return; }

        if gl::GetNamedBufferSubData::is_loaded() {
            gl::GetNamedBufferSubData(
                self.id(), self.offset() as GLintptr, size as GLintptr, data as *mut GLvoid
            );
        } else {
            ARRAY_BUFFER.map_bind(self, |b|
                gl::GetBufferSubData(
                    b.target_id(), self.offset() as GLintptr, size, data as *mut GLvoid
                )
            );
        }
    }

    pub fn get_subdata(&self) -> T where T:Copy {
        unsafe {
            let mut data = MaybeUninit::uninit();
            self.get_subdata_raw(data.as_mut_ptr());
            data.assume_init()
        }
    }

    pub fn get_subdata_box(&self) -> Box<T> where T:GPUCopy  {
        unsafe {
            //allocates a box of the proper size and then downloads into its pointer
            map_alloc(self.ptr, |ptr| self.get_subdata_raw(ptr))
        }
    }

    pub fn get_subdata_ref<Q:BorrowMut<T>>(&self, dest: &mut Q) where T:GPUCopy  {
        //borrow
        let dest = dest.borrow_mut();

        //check the bounds (get_subdata_raw does NOT do this for us)
        assert_eq!(size_of_val(dest), self.size(), "destination and source have different sizes");

        //since T is GPUCopy, we can just do a normal direct download
        unsafe { self.get_subdata_raw(dest.borrow_mut()) }
    }

    pub unsafe fn copy_subdata_raw<'b, GL:Supports<GL_ARB_copy_buffer>, B:BufferStorage>(
        &self, #[allow(unused_variables)] gl: &GL, dest: &mut SliceMut<'b,T,B>
    ) {
        if self.size()==0 || dest.size()==0 { return; }
        COPY_READ_BUFFER.map_bind(self, |b1|
            COPY_WRITE_BUFFER.map_bind(dest, |b2|
                gl::CopyBufferSubData(
                    b1.target_id(), b2.target_id(),
                    self.offset as GLintptr, dest.offset as GLintptr,
                    self.size() as GLsizeiptr
                )
            )
        )
    }

    pub fn copy_subdata<'b, GL:Supports<GL_ARB_copy_buffer>, B:BufferStorage>(
        &self, gl: &GL, dest: &mut SliceMut<'b,T,B>
    ) where T: GPUCopy+'a {
        assert_eq!(dest.size(), self.size(), "destination and source buffers have different sizes");
        unsafe{ self.copy_subdata_raw(gl, dest) }
    }

}

impl<'a,T:Sized,A:BufferStorage> Slice<'a,[T],A> {
    #[inline] pub fn len(&self) -> usize {self.ptr.len()}
    #[inline] pub fn is_empty(&self) -> bool { self.len()==0 }

    #[inline]
    pub unsafe fn from_raw_parts(id:GLuint, len:usize, offset:usize) -> Self {
        Slice{ptr: BufPtr::from_raw_parts(id, len), offset, buf:PhantomData}
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
        match self.is_empty() {
            true => None,
            _ => {
                let (first, rest) = self.split_at(1);
                Some((first.index(0), rest))
            }
        }
    }

    pub fn split_last(self) -> Option<(Slice<'a,T,A>, Slice<'a,[T],A>)> {
        match self.is_empty() {
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

}

//
//Vertex Attributes
//
impl<'a,T:AttribData, A:BufferStorage> Slice<'a,[T],A> {

    pub fn into_attrib_array(self) -> AttribArray<'a,T::GLSL> {
        self.into()
    }

    pub fn split_attribs(self) -> <T::GLSL as SplitAttribs<'a>>::Split
    where T::GLSL: SplitAttribs<'a>
    {
        <T::GLSL as SplitAttribs<'a>>::split_array(self.into())
    }
}


impl<'a,F:SpecificCompressed, A:BufferStorage> Slice<'a,CompressedPixels<F>,A> {
    #[inline] pub fn blocks(&self) -> usize { self.ptr.blocks() }
    #[inline] pub fn pixel_count(&self) -> usize { self.ptr.pixel_count() }
}

impl<'a,T:?Sized,A:BufferStorage> SliceMut<'a,T,A> {

    #[inline] pub fn id(&self) -> GLuint {self.ptr.id()}
    #[inline] pub fn size(&self) -> usize {self.ptr.size()}
    #[inline] pub fn align(&self) -> usize {self.ptr.align()}
    #[inline] pub fn offset(&self) -> usize {self.offset}

    #[inline] pub(crate) fn offset_ptr(&self) -> *const T {
        self.ptr.swap_offset(self.offset)
    }

    #[inline] pub(crate) fn offset_ptr_mut(&mut self) -> *mut T {
        self.ptr.swap_offset_mut(self.offset)
    }

    #[inline] pub fn as_slice(&self) -> Slice<T,A> { Slice::from(self) }
    #[inline] pub fn as_mut_slice(&mut self) -> SliceMut<T,A> { SliceMut::from(self) }

    #[inline] pub unsafe fn downgrade_unchecked<B:BufferStorage>(self) -> SliceMut<'a,T,B> {
        SliceMut{ptr: self.ptr, offset: self.offset, buf: PhantomData}
    }

    #[inline]
    pub fn downgrade<B:BufferStorage>(self) -> SliceMut<'a,T,B> where A:DowngradesTo<B> {
        unsafe { self.downgrade_unchecked() }
    }

    #[inline] pub unsafe fn get_subdata_raw(&self, dest: *mut T) { self.as_slice().get_subdata_raw(dest) }

    #[inline] pub fn get_subdata(&self) -> T where T:Copy {self.as_slice().get_subdata()}
    #[inline] pub fn get_subdata_box(&self) -> Box<T> where T:GPUCopy {self.as_slice().get_subdata_box()}
    #[inline] pub fn get_subdata_ref<Q:BorrowMut<T>>(&self, dest: &mut Q) where T:GPUCopy {
        self.as_slice().get_subdata_ref(dest)
    }

    #[inline]
    pub unsafe fn copy_subdata_raw<'b, GL:Supports<GL_ARB_copy_buffer>, B:BufferStorage>(
        &self, gl: &GL, dest: &mut SliceMut<'b,T,B>
    ) {
        self.as_slice().copy_subdata_raw(gl, dest)
    }

    #[inline]
    pub fn copy_subdata<'b, GL:Supports<GL_ARB_copy_buffer>, B:BufferStorage>(
        &self, gl: &GL, dest: &mut SliceMut<'b,T,B>
    ) where T: GPUCopy+'a {
        self.as_slice().copy_subdata(gl, dest)
    }

    pub unsafe fn invalidate_subdata_raw(&mut self) {
        if self.size()==0 { return; }
        if gl::InvalidateBufferSubData::is_loaded() {
            gl::InvalidateBufferSubData(self.id(), self.offset() as GLintptr, self.size() as GLsizeiptr)
        }
    }

}

impl<'a,T:Sized,A:BufferStorage> SliceMut<'a,[T],A> {
    #[inline] pub fn len(&self) -> usize {self.as_slice().len()}
    #[inline] pub fn is_empty(&self) -> bool { self.len()==0 }

    #[inline]
    pub unsafe fn from_raw_parts(id:GLuint, len:usize, offset:usize) -> Self {
        SliceMut{ptr: BufPtr::from_raw_parts(id, len), offset, buf:PhantomData}
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

}

//
//Vertex Attributes
//
impl<'a,T:AttribData, A:BufferStorage> SliceMut<'a,[T],A> {

    pub fn into_attrib_array(self) -> AttribArray<'a,T::GLSL> {
        Slice::from(self).into()
    }

    pub fn split_attribs(self) -> <T::GLSL as SplitAttribs<'a>>::Split
    where T::GLSL: SplitAttribs<'a>
    {
        <T::GLSL as SplitAttribs<'a>>::split_array(self.into_attrib_array())
    }
}

impl<'a,F:SpecificCompressed, A:BufferStorage> SliceMut<'a,CompressedPixels<F>,A> {
    #[inline] pub fn blocks(&self) -> usize { self.ptr.blocks() }
    #[inline] pub fn pixel_count(&self) -> usize { self.ptr.pixel_count() }
}

//
//Writing subdata: glBufferSubData
//

impl<'a,T:?Sized,A:Dynamic> SliceMut<'a,T,A> {

    pub unsafe fn subdata_raw(&mut self, data: *const T) {

        //convenience vars
        let void = data as *const GLvoid;
        let size = self.size().min(size_of_val(&*data)) as GLsizeiptr;

        //if we are copying over nothing, we don't need to call anything (and it might even throw an error)
        if size==0 { return; }

        if gl::NamedBufferSubData::is_loaded() {
            gl::NamedBufferSubData(self.id(), self.offset as GLintptr, size, void);
        } else {
            ARRAY_BUFFER.map_bind(self,
                |b| gl::BufferSubData( b.target_id(), self.offset as GLintptr, size, void)
            );
        }
    }

    pub fn subdata(&mut self, data: T) where T: Copy {
        //just use subdata_raw since data implements Copy
        unsafe { self.subdata_raw(&data) }
    }

    pub fn subdata_ref<Q:Borrow<T>>(&mut self, data: &Q) where T: GPUCopy {
        //borrow
        let data = data.borrow();

        //check bounds
        assert_eq!(size_of_val(data), self.size(), "destination and source have different sizes");

        //since T is GPUCopy, we can just directly upload
        unsafe { self.subdata_raw(data) }
    }

    pub fn replace(&mut self, data: T) -> T where T:Sized {
        unsafe {
            //read the buffer data into a temporary variable
            let mut old_data = MaybeUninit::<T>::uninit();
            self.get_subdata_raw(old_data.as_mut_ptr());

            //modify the buffer
            self.subdata_raw(&data);
            forget(data); //we need to make sure the destructor of data is NOT run

            old_data.assume_init()
        }
    }

    pub fn replace_ref<Q:BorrowMut<T>>(&mut self, data: &mut Q) where T:Sized {
        unsafe {
            //read the buffer data into a temporary variable
            let mut old_data = MaybeUninit::<T>::uninit();
            self.get_subdata_raw(old_data.as_mut_ptr());

            //swap the memory
            std::ptr::swap(data.borrow_mut(), old_data.as_mut_ptr());

            //modify the buffer
            self.subdata_raw(old_data.as_ptr());
        }
    }

}
