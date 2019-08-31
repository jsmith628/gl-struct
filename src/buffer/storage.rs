use super::*;


//
//Wrappers for glBufferStorage and glBufferData
//(ie Buffer creation methods)
//

impl<T:?Sized, A:BufferAccess> Buffer<T,A> {

    pub unsafe fn storage_raw(_gl: &GL44, raw: RawBuffer, data: *const T, hint:StorageHint) -> Self {

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
        if gl::NamedBufferStorage::is_loaded() {
            gl::NamedBufferStorage(raw.id(), size as GLsizeiptr, ptr, flags)
        } else {
            let mut target = BufferTarget::CopyWriteBuffer.as_loc();
            gl::BufferStorage(target.bind(&raw).target_id(), size as GLsizeiptr, ptr, flags);
        }


        //make sure we don't delete the buffer by accident
        forget(raw);

        //now, constuct a buffer with that pointer, where the leading half is the buffer id and the
        //latter half is any object metadata
        Buffer {
            ptr: conv.rust_mut,
            access: PhantomData
        }

    }

    pub unsafe fn data_raw(raw: RawBuffer, data: *const T, usage: DataHint) -> Self where A:NonPersistentAccess {
        //get the size of the object
        let size = size_of_val(&*data);

        //swap out the first half of the data pointer with the buffer id in order to get the void ptr
        //half and to construct the pointer for the buffer object
        let mut conv = BufPtr{ rust: data };
        let ptr = conv.gl;
        conv.buf = raw.id();

        //upload the data
        if gl::NamedBufferData::is_loaded() {
            gl::NamedBufferData(raw.id(), size as GLsizeiptr, ptr, usage.unwrap_or(Default::default()) as GLenum);
        } else {
            let mut target = BufferTarget::CopyWriteBuffer.as_loc();
            let tar = target.bind(&raw).target_id();
            gl::BufferData(tar, size as GLsizeiptr, ptr, usage.unwrap_or(Default::default()) as GLenum);
        }

        //make sure we don't delete the buffer by accident
        forget(raw);

        //now, constuct a buffer with that pointer, where the leading half is the buffer id and the
        //latter half is any object metadata
        Buffer {
            ptr: conv.rust_mut,
            access: PhantomData
        }
    }

    pub unsafe fn from_raw(gl: &GL15, data: *const T, hint: CreationHint) -> Self where A:NonPersistentAccess {
        let raw = RawBuffer::gen(gl);
        if let Ok(gl4) = gl.try_as_gl44() {
            Self::storage_raw(&gl4, raw, data, hint.map(|h| h.1))
        } else {
            Self::data_raw(raw, data, hint.map(|h| h.0))
        }
    }

}

impl<T:?Sized, A:BufferAccess> Buffer<T,A> {

    pub fn storage(gl: &GL44, raw: RawBuffer, data: Box<T>, hint: StorageHint) -> Self {
        map_dealloc(data, |ptr| unsafe{Self::storage_raw(gl, raw, ptr, hint)})
    }

    pub fn data(raw: RawBuffer, data: Box<T>, usage: DataHint) -> Self where A:NonPersistentAccess {
        map_dealloc(data, |ptr| unsafe{Self::data_raw(raw, ptr, usage)})
    }

    pub fn from_box(gl: &GL15, data: Box<T>, hint: CreationHint) -> Self where A:NonPersistentAccess {
        map_dealloc(data, |ptr| unsafe{Self::from_raw(gl, ptr, hint)})
    }

}

impl<T:?Sized+GPUCopy, A:BufferAccess> Buffer<T,A> {

    pub fn storage_ref(gl: &GL44, raw: RawBuffer, data: &T, hint: StorageHint) -> Self {
        unsafe { Self::storage_raw(gl, raw, data as *const T, hint) }
    }

    pub fn data_ref(uninit: RawBuffer, data: &T, usage: DataHint) -> Self where A:NonPersistentAccess {
        unsafe { Self::data_raw(uninit, data as *const T, usage) }
    }

    pub fn from_ref(gl: &GL15, data: &T, hint: CreationHint) -> Self where A:NonPersistentAccess {
        unsafe { Self::from_raw(gl, data as *const T, hint) }
    }
}

impl<T:Sized, A:BufferAccess> Buffer<T,A> {

    pub fn storage_uninit(gl: &GL44, raw: RawBuffer, hint: StorageHint) -> Self {
        unsafe { Self::storage_raw(gl, raw, null::<T>(), hint) }
    }

    pub unsafe fn data_uninit(raw: RawBuffer, usage: DataHint) -> Self where A:NonPersistentAccess {
        Self::data_raw(raw, null::<T>(), usage)
    }

    pub unsafe fn uninit(gl: &GL15, hint: CreationHint) -> Self where A:NonPersistentAccess {
        Self::from_raw(gl, null::<T>(), hint)
    }

}

impl<T:Sized, A:BufferAccess> Buffer<[T],A> {

    pub unsafe fn storage_uninit_count(gl: &GL44, raw:RawBuffer, count: usize, hint: StorageHint) -> Self {
        Self::storage_raw(gl, raw, from_raw_parts(null::<T>(), count), hint)
    }

    pub unsafe fn data_uninit_count(raw: RawBuffer, count: usize, usage: DataHint) -> Self where A:NonPersistentAccess {
        Self::data_raw(raw, from_raw_parts(null::<T>(), count), usage)
    }

    pub unsafe fn uninit_count(gl: &GL15, count: usize, hint: CreationHint) -> Self where A:NonPersistentAccess {
        Self::from_raw(gl, from_raw_parts(null::<T>(), count), hint)
    }

}
