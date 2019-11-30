use super::*;


pub type UninitBuf = Buffer<(), !>;

impl UninitBuf {

    unsafe fn from_id(id:GLuint) -> Self { Buffer { ptr: BufPtr::new(id, null_mut()), access: PhantomData } }

    pub fn gen(#[allow(unused_variables)] gl: &GL15) -> UninitBuf {
        let mut id = MaybeUninit::uninit();
        unsafe {
            gl::GenBuffers(1, id.as_mut_ptr());
            Self::from_id(id.assume_init())
        }
    }

    pub fn gen_buffers(#[allow(unused_variables)] gl: &GL15, n:GLuint) -> Box<[UninitBuf]> {
        if n==0 { return Box::new([]); }
        let mut ids = Box::new_uninit_slice(n as usize);
        unsafe {
            gl::GenBuffers(n as GLsizei, ids[0].as_mut_ptr());
            ids.into_iter().map(|id| Self::from_id(id.assume_init())).collect()
        }
    }

    pub fn create(#[allow(unused_variables)] gl: &GL15) -> UninitBuf {
        let mut id = MaybeUninit::uninit();
        unsafe {
            if gl::CreateBuffers::is_loaded() {
                gl::CreateBuffers(1, id.as_mut_ptr());
            } else {
                gl::GenBuffers(1, id.as_mut_ptr());
                gl::BindBuffer(gl::COPY_WRITE_BUFFER, id.assume_init());
                gl::BindBuffer(gl::COPY_WRITE_BUFFER, 0);
            }
            Self::from_id(id.assume_init())
        }
    }

    pub fn create_buffers(gl: &GL15, n:GLuint) -> Box<[UninitBuf]> {
        if n==0 { return Box::new([]); }
        let mut ids = Box::new_uninit_slice(n as usize);
        unsafe {
            if gl::CreateBuffers::is_loaded() {
                gl::CreateBuffers(n as GLsizei, ids[0].as_mut_ptr());
                ids.into_iter().map(|id| Self::from_id(id.assume_init())).collect()
            } else {
                let mut bufs = Self::gen_buffers(gl, n);
                for b in bufs.iter_mut() { gl::BindBuffer(gl::COPY_WRITE_BUFFER, b.id()); }
                gl::BindBuffer(gl::COPY_WRITE_BUFFER, 0);
                bufs
            }
        }
    }

    pub unsafe fn storage_raw<T:?Sized,A:Initialized>(
        self,
        #[allow(unused_variables)] gl: &GL44,
        data: *const T,
        hint:StorageHint
    ) -> Buffer<T,A> {

        //get the size and pointer of the object
        let size = size_of_val(&*data) as GLsizeiptr;
        let ptr = data as *const GLvoid;

        //get the creation flags
        let mut flags = 0;
        if <A::MapRead as Bit>::VALUE { flags |= gl::MAP_READ_BIT };
        if <A::MapWrite as Bit>::VALUE { flags |= gl::MAP_WRITE_BIT };
        if <A::MapPersistent as Bit>::VALUE { flags |= gl::MAP_PERSISTENT_BIT };
        if <A::DynamicStorage as Bit>::VALUE { flags |= gl::DYNAMIC_STORAGE_BIT };
        if let Some(hints) = hint {
            flags |= gl::CLIENT_STORAGE_BIT & hints.bits()
        }

        //upload the data
        if size!=0 {
            if gl::NamedBufferStorage::is_loaded() {
                gl::NamedBufferStorage(self.id(), size, ptr, flags)
            } else {
                BufferTarget::CopyWriteBuffer.as_loc().map_bind(&self,
                    |b| gl::BufferStorage(b.target_id(), size, ptr, flags)
                );
            }
        }

        //construct the inner representation for the buffer
        let inner = BufPtr::new(self.id(), data as *mut T);

        //make sure we don't delete the buffer by accident
        forget(self);

        //now, constuct a buffer with that pointer, where the leading half is the buffer id and the
        //latter half is any object metadata
        Buffer {
            ptr: inner,
            access: PhantomData
        }
    }

    pub fn storage_box<T:?Sized,A:Initialized>(self, gl: &GL44, data: Box<T>, hint: StorageHint) -> Buffer<T,A> {
        map_dealloc(data, |ptr| unsafe{self.storage_raw(gl, ptr, hint)})
    }

    pub fn storage<T:Sized,A:Initialized>(self, gl: &GL44, data: T, hint: StorageHint) -> Buffer<T,A> {
        unsafe {
            let buf = self.storage_raw(gl, &data, hint);
            forget(data);
            buf
        }
    }

    pub fn storage_uninit<T:Sized,A:Initialized>(self, gl: &GL44, hint: StorageHint) -> Buffer<MaybeUninit<T>,A> {
        unsafe { self.storage_raw(gl, null(), hint) }
    }

    pub fn storage_uninit_slice<T:Sized,A:Initialized>(
        self, gl: &GL44, count: usize, hint: StorageHint
    ) -> Buffer<[MaybeUninit<T>],A> {
        unsafe { self.storage_raw(gl, slice_from_raw_parts(null(), count), hint) }
    }

    pub unsafe fn data_raw<T:?Sized>(self, data: *const T, hint:DataHint) -> Buffer<T,ReadWrite> {

        //get the size and pointer of the object
        let size = size_of_val(&*data) as GLsizeiptr;
        let ptr = data as *const GLvoid;

        //get the usage
        let usage = hint.unwrap_or(Default::default()) as GLenum;

        //upload the data
        if size!=0 {
            if gl::NamedBufferData::is_loaded() {
                gl::NamedBufferData(self.id(), size, ptr, usage)
            } else {
                BufferTarget::CopyWriteBuffer.as_loc().map_bind(&self,
                    |b| gl::BufferData(b.target_id(), size, ptr, usage)
                );
            }
        }

        //construct the inner representation for the buffer
        let inner = BufPtr::new(self.id(), data as *mut T);

        //make sure we don't delete the buffer by accident
        forget(self);

        //now, constuct a buffer with that pointer, where the leading half is the buffer id and the
        //latter half is any object metadata
        Buffer {
            ptr: inner,
            access: PhantomData
        }

    }

    pub fn data_box<T:?Sized>(self, data: Box<T>, usage: DataHint) -> Buffer<T,ReadWrite> {
        map_dealloc(data, |ptr| unsafe{self.data_raw(ptr, usage)})
    }

    pub fn data<T:Sized>(self, data: T, usage: DataHint) -> Buffer<T,ReadWrite> {
        unsafe {
            let buf = self.data_raw(&data, usage);
            forget(data);
            buf
        }
    }

    pub fn data_uninit<T:Sized>(self, usage: DataHint) -> Buffer<MaybeUninit<T>,ReadWrite> {
        unsafe { self.data_raw(null(), usage) }
    }

    pub fn data_uninit_slice<T:Sized>(self, count: usize, usage: DataHint) -> Buffer<[MaybeUninit<T>],ReadWrite> {
        unsafe { self.data_raw(slice_from_raw_parts(null(), count), usage) }
    }

    pub unsafe fn with_raw<T:?Sized,A:NonPersistent>(
        self, data: *const T, hint:CreationHint
    ) -> Buffer<T,A> {
        if let Ok(gl) = self.gl().try_as_gl44() {
            self.storage_raw(&gl, data, hint.map(|h| h.1))
        } else {
            self.data_raw(data, hint.map(|h| h.0)).downgrade::<A>()
        }
    }

    pub fn with_box<T:?Sized,A:NonPersistent>(self, data: Box<T>, hint: CreationHint) -> Buffer<T,A> {
        map_dealloc(data, |ptr| unsafe{self.with_raw(ptr, hint)})
    }

    pub fn with<T:Sized,A:NonPersistent>(self, data: T, hint: CreationHint) -> Buffer<T,A> {
        unsafe {
            let buf = self.with_raw(&data, hint);
            forget(data);
            buf
        }
    }

    pub fn uninit<T:Sized,A:NonPersistent>(self, hint: CreationHint) -> Buffer<MaybeUninit<T>,A> {
        unsafe { self.with_raw(null(), hint) }
    }

    pub fn uninit_slice<T:Sized,A:NonPersistent>(
        self, count: usize, hint: CreationHint
    ) -> Buffer<[MaybeUninit<T>],A> {
        unsafe { self.with_raw(slice_from_raw_parts(null(), count), hint) }
    }

}
