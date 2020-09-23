use super::*;

union BufUnion<T:?Sized> {
    gl: *const GLvoid,
    gl_mut: *mut GLvoid,
    rust: *const T,
    rust_mut: *mut T,
    buf: usize,
}

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""), PartialEq(bound=""), Eq(bound=""))]
pub(crate) struct BufPtr<T:?Sized>{
    ptr: *mut T
}

impl<U:?Sized, T:?Sized+Unsize<U>> CoerceUnsized<BufPtr<U>> for BufPtr<T> {}

fn check_alignment(ptr:*const GLvoid, align: usize) {
    assert_eq!((ptr as usize) % align, 0, "Invalid map alignment for type");
}

impl<T:?Sized> BufPtr<T> {

    #[inline]
    pub fn new(id: GLuint, ptr: *mut T) -> Self {
        let mut union = BufUnion {rust_mut: ptr};
        union.buf = id as usize;
        BufPtr { ptr: unsafe { union.rust_mut } }
    }

    #[inline] pub fn id(self) -> GLuint { unsafe {BufUnion{rust_mut: self.ptr}.buf as GLuint} }
    #[inline] pub fn size(self) -> usize { unsafe {size_of_val(&*self.ptr)} }
    #[inline] pub fn align(self) -> usize { unsafe {align_of_val(&*self.ptr)} }
    #[inline] pub fn needs_drop(self) -> bool { unsafe { (&*self.ptr).needs_drop_val() } }


    #[inline]
    #[allow(dead_code)]
    pub fn dangling(self) -> *const T { unsafe {self.swap_ptr_unchecked(NonNull::dangling().as_ptr())} }

    #[inline]
    pub fn dangling_mut(self) -> *mut T { unsafe {self.swap_mut_ptr_unchecked(NonNull::dangling().as_ptr())} }

    #[inline]
    pub fn swap_offset(self, offset: usize) -> *const T {
        self.swap_ptr(offset as *const GLvoid)
    }

    #[inline]
    pub fn swap_offset_mut(self, offset: usize) -> *mut T {
        self.swap_mut_ptr(offset as *mut GLvoid)
    }

    #[inline]
    pub fn swap_ptr(self, ptr: *const GLvoid) -> *const T {
        check_alignment(ptr, self.align());
        unsafe { self.swap_ptr_unchecked(ptr) }
    }

    #[inline]
    pub fn swap_mut_ptr(self, ptr: *mut GLvoid) -> *mut T {
        check_alignment(ptr, self.align());
        unsafe { self.swap_mut_ptr_unchecked(ptr) }
    }

    #[inline]
    pub unsafe fn swap_ptr_unchecked(self, ptr: *const GLvoid) -> *const T {
        let mut union = BufUnion {rust_mut: self.ptr};
        union.gl = ptr;
        union.rust
    }

    #[inline]
    pub unsafe fn swap_mut_ptr_unchecked(self, ptr: *mut GLvoid) -> *mut T {
        let mut union = BufUnion {rust_mut: self.ptr};
        union.gl_mut = ptr;
        union.rust_mut
    }

    unsafe fn get_parameter_iv(&self, value:GLenum) -> GLint {
        let mut dest = MaybeUninit::uninit();
        if gl::GetNamedBufferParameteriv::is_loaded() {
            gl::GetNamedBufferParameteriv(self.id(), value, dest.as_mut_ptr());
        } else {
            ARRAY_BUFFER.map_bind(self,
                |binding| gl::GetBufferParameteriv(binding.target_id(), value, dest.as_mut_ptr())
            );
        }
        dest.assume_init()
    }

    unsafe fn get_parameter_i64v(&self, value:GLenum) -> GLint64 {
        let mut dest = MaybeUninit::uninit();
        if gl::GetNamedBufferParameteri64v::is_loaded() {
            gl::GetNamedBufferParameteri64v(self.id(), value, dest.as_mut_ptr());
        } else {
            ARRAY_BUFFER.map_bind(self,
                |binding| gl::GetBufferParameteri64v(binding.target_id(), value, dest.as_mut_ptr())
            );
        }
        dest.assume_init()
    }

    pub unsafe fn buffer_size(&self) -> usize {
        if gl::GetBufferParameteri64v::is_loaded() {
            self.get_parameter_i64v(gl::BUFFER_SIZE) as usize
        } else {
            self.get_parameter_iv(gl::BUFFER_SIZE) as usize
        }
    }

    pub unsafe fn immutable_storage(&self) -> bool {
        if gl::BufferStorage::is_loaded() {
            self.get_parameter_iv(gl::BUFFER_IMMUTABLE_STORAGE) != 0
        } else {
            false
        }
    }

    pub unsafe fn storage_flags(&self) -> StorageFlags {
        if gl::BufferStorage::is_loaded() {
            StorageFlags::from_bits(self.get_parameter_iv(gl::BUFFER_STORAGE_FLAGS) as GLbitfield).unwrap()
        } else {
            StorageFlags::MAP_READ_BIT | StorageFlags::MAP_WRITE_BIT | StorageFlags::DYNAMIC_STORAGE_BIT
        }
    }

    pub unsafe fn usage(&self) -> BufferUsage {
        (self.get_parameter_iv(gl::BUFFER_USAGE) as GLenum).try_into().unwrap()
    }

    pub unsafe fn creation_flags(&self) -> BufferCreationFlags {
        BufferCreationFlags(self.usage(), self.storage_flags())
    }

}

impl<T> BufPtr<[T]> {
    #[inline] pub fn len(self) -> usize { unsafe {(&*self.ptr).len()} }

    #[inline]
    pub unsafe fn from_raw_parts(id: GLuint, len: usize) -> Self {
        Self::new(id, std::slice::from_raw_parts_mut(NonNull::dangling().as_mut(), len))
    }

}

impl<F:SpecificCompressed> BufPtr<CompressedPixels<F>> {
    #[inline] pub fn is_empty(self) -> bool { unsafe {(&*self.ptr).is_empty()} }
    #[inline] pub fn blocks(self) -> usize { unsafe {(&*self.ptr).blocks()} }
    #[inline] pub fn len(self) -> usize { unsafe {(&*self.ptr).len()} }
}

///
///Any type that can be cloned within a [buffer](super::Buffer) by simple byte-wise copies of its data.
///
#[marker] pub trait GPUCopy {}

impl<T:Copy> GPUCopy for T {}
impl<T:Copy> GPUCopy for [T] {}
impl GPUCopy for str {}
impl GPUCopy for std::ffi::CStr {}
impl GPUCopy for std::ffi::OsStr {}
impl GPUCopy for std::path::Path {}

macro_rules! impl_tuple_gpucopy {
    ({$($T:ident:$t:ident)*} $Last:ident:$l:ident) => {
        impl<$($T:GPUCopy,)* $Last: GPUCopy+?Sized> GPUCopy for ($($T,)* $Last,) {}
    };
}
impl_tuple!(impl_tuple_gpucopy @with_last);

///Gives a hint as to if the given value needs its destructor run
unsafe trait NeedsDropVal { fn needs_drop_val(&self) -> bool; }

unsafe impl<T:?Sized> NeedsDropVal for T { #[inline] default fn needs_drop_val(&self) -> bool {true} }
unsafe impl<T:Sized> NeedsDropVal for T { #[inline] fn needs_drop_val(&self) -> bool {needs_drop::<T>()} }
unsafe impl<T:Sized> NeedsDropVal for [T] {
    #[inline] fn needs_drop_val(&self) -> bool {!self.is_empty() && needs_drop::<T>()}
}
unsafe impl<F:SpecificCompressed> NeedsDropVal for CompressedPixels<F> {
    #[inline] fn needs_drop_val(&self) -> bool {self.len()>0 && needs_drop::<F::Block>()}
}

macro_rules! impl_tuple_needs_drop {
    ({$($T:ident:$t:ident)*} $Last:ident:$l:ident) => {
        unsafe impl<$($T,)* $Last> NeedsDropVal for ($($T,)* [$Last],) {
            #[inline] fn needs_drop_val(&self) -> bool {
                let (.., $l) = self;
                $(needs_drop::<$T>() || )* $l.needs_drop_val()
            }
        }
    };
}
impl_tuple!(impl_tuple_needs_drop @with_last);
