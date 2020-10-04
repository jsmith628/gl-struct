#![feature(trait_alias)]
#![feature(specialization)]
#![feature(marker_trait_attr)]
#![feature(never_type)]
#![feature(untagged_unions)]
#![feature(const_fn)]
#![feature(concat_idents)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(allocator_api)]
#![feature(arbitrary_enum_discriminant)]
#![feature(get_mut_unchecked)]
#![feature(new_uninit)]
#![feature(maybe_uninit_ref)]
#![feature(maybe_uninit_slice)]

//these will probably be removed at some point,
//but for now, I don't want to be swamped with clippy warnings
#![allow(clippy::type_complexity)]
#![allow(clippy::missing_safety_doc)]

//this is probably really bad, but I don't really care
#![recursion_limit="32768"]

pub extern crate gl;
pub extern crate num_traits;
#[cfg(feature = "glfw-context")] extern crate glfw;
#[cfg(feature = "glutin-context")] extern crate glutin;

#[macro_use] extern crate bitflags;
#[macro_use] extern crate derivative;

use gl::types::*;
use std::convert::*;
use std::fmt;
use std::fmt::{Display, Debug, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;

#[macro_use] mod macros;
#[macro_use] pub mod glsl;

pub mod version;
pub mod context;

pub mod format;
pub mod pixels;
pub mod image;

pub mod buffer;
pub mod texture;
pub mod renderbuffer;
pub mod vertex_array;
pub mod framebuffer;
pub mod sampler;
// pub mod query;
pub mod sync;

pub trait Bit { const VALUE:bool; }
pub struct High;
pub struct Low;

impl Bit for High { const VALUE:bool = true; }
impl Bit for Low { const VALUE:bool = false; }

#[marker] pub trait BitMasks<B:Bit>: Bit {}
impl<B:Bit> BitMasks<Low> for B {}
impl<B:Bit> BitMasks<B> for High {}

pub trait GLEnum: Sized + Copy + Eq + Hash + Debug + Display + Into<GLenum> + TryFrom<GLenum, Error=GLError> {}

glenum! {
    pub enum IntType {
        [Byte BYTE "Byte"],
        [UByte UNSIGNED_BYTE "UByte"],
        [Short SHORT "Short"],
        [UShort UNSIGNED_SHORT "UShort"],
        [Int INT "Int"],
        [UInt UNSIGNED_INT "UInt"]
    }
}

impl IntType {
    #[inline]
    pub fn size_of(self) -> usize {
        match self {
            IntType::Byte | IntType::UByte => 1,
            IntType::Short |IntType::UShort => 2,
            IntType::Int | IntType::UInt => 4
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GLError {
    InvalidEnum(GLenum, String),
    InvalidOperation(String),
    InvalidValue(String),
    InvalidBits(GLbitfield, String),
    BufferCopySizeError(usize, usize),
    FunctionNotLoaded(&'static str),
    Version(GLuint, GLuint)
}

display_from_debug!(GLError);
impl Debug for GLError {

    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GLError::InvalidEnum(id, ty) => write!(f, "Invalid enum: #{} is not a valid {}", id, ty),
            GLError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            GLError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            GLError::InvalidBits(id, ty) => write!(f, "Invalid bitfield: {:b} are not valid flags for {}", id, ty),
            GLError::FunctionNotLoaded(name) => write!(f, "{} not loaded", name),
            GLError::Version(maj, min) => write!(f, "OpenGL version {}.{} not supported", maj, min),
            GLError::BufferCopySizeError(s, cap) =>
                write!(f, "Invalid Buffer Copy: Source size {} smaller than Destination capacity {}.
                (If you are using an array, try slicing first.)", s, cap),
        }
    }

}

glenum! {
    #[non_exhaustive]
    pub(self) enum ResourceIdentifier {
        [Buffer BUFFER "Buffer"],
        [Framebuffer FRAMEBUFFER "Framebuffer"],
        [ProgramPipeline PROGRAM_PIPELINE "Program Pipeline"],
        [Program PROGRAM "Program"],
        [Query QUERY "Query"],
        [Renderbuffer RENDERBUFFER "Renderbuffer"],
        [Sampler SAMPLER "Sampler"],
        [Shader SHADER "Shader"],
        [Texture TEXTURE "Texture"],
        [TransformFeedback TRANSFORM_FEEDBACK "Transform Feedback"],
        [VertexArray VERTEX_ARRAY "Vertex Array"]
    }
}

pub(self) fn object_label(identifier: ResourceIdentifier, name: GLuint, label:&str) -> Result<(),GLError> {
    use std::mem::MaybeUninit;

    unsafe {
        if gl::ObjectLabel::is_loaded() {
            let mut max_length = MaybeUninit::uninit();
            gl::GetIntegerv(gl::MAX_LABEL_LENGTH, max_length.as_mut_ptr());
            if max_length.assume_init() >= label.len() as GLint {
                gl::ObjectLabel(
                    identifier as GLenum, name,
                    label.len() as GLsizei, label.as_ptr() as *const GLchar
                );
                Ok(())
            } else {
                Err(GLError::InvalidValue("object label longer than maximum allowed length".to_string()))
            }
        } else {
            Err(GLError::FunctionNotLoaded("ObjectLabel"))
        }
    }
}

pub(self) fn get_object_label(identifier: ResourceIdentifier, name: GLuint) -> Option<String>{
    use std::mem::MaybeUninit;
    use std::ptr::*;

    unsafe {
        if gl::GetObjectLabel::is_loaded() {
            //get the length of the label
            let mut length = MaybeUninit::uninit();
            gl::GetObjectLabel(
                identifier as GLenum, name, 0, length.as_mut_ptr(), null_mut()
            );

            let length = length.assume_init();
            if length==0 { //if there is no label
                None
            } else {
                //allocate the space for the label
                let mut bytes = Vec::with_capacity(length as usize);
                bytes.set_len(length as usize);

                //copy the label
                gl::GetObjectLabel(
                    identifier as GLenum, name,
                    length as GLsizei, null_mut(), bytes.as_mut_ptr() as *mut GLchar
                );

                //since label() loads from a &str, we can assume the returned bytes are valid utf8
                Some(String::from_utf8_unchecked(bytes))
            }

        } else {
            None
        }
    }
}

pub(crate) trait Target<R>: Copy + Eq + Hash + Debug + Display {
    fn target_id(self) -> GLenum;
    unsafe fn bind(self, obj: &R);
    unsafe fn unbind(self);
}

///An object that owns a [Target] to a glBind* function for a resource `R`
#[derive(PartialEq, Eq, Hash)]
pub(crate) struct BindingLocation<T>(pub(crate) T, PhantomData<*const ()>);

///An object that owns a binding of a [Resource] to a particular [BindingLocation] and unbinds it when leaving scope
pub(crate) struct Binding<'a,R,T:Target<R>>(pub(crate) &'a BindingLocation<T>, pub(crate) &'a R);

impl<'a,R,T:Target<R>> Binding<'a,R,T> {
    #[inline] pub fn target(&self) -> T { *self.0.target() }
    #[inline] pub fn resource(&self) -> &R { self.1 }
    #[inline] pub fn target_id(&self) -> GLenum { self.target().target_id() }
}

impl<'a,R,T:Target<R>> Drop for Binding<'a,R,T> {
    #[inline] fn drop(&mut self) { unsafe { self.target().unbind() } }
}

impl<T> BindingLocation<T> {

    ///The [target](Target) of this location
    pub fn target(&self) -> &T where T:Copy { &self.0 }

    ///
    ///Constructs a new binding location with the given target
    ///
    ///# Unsafety
    ///It is up to the caller to guarrantee that this is the only location with the given
    ///[binding target](Target) at the given time
    ///
    #[inline]
    pub const unsafe fn new(target: T) -> Self {BindingLocation(target, PhantomData)}

    ///
    ///A wrapper of glBind* for `R` using an owned resource
    ///
    ///The fact that it is owned means that we can bind without checking validity of the resource
    ///id. Furthermore, for the same reasons as [bind_raw](BindingLocation::bind_raw), this method is actually safe
    ///
    #[inline]
    pub fn bind<'a,R>(&'a mut self, resource: &'a R) -> Binding<'a,R,T> where T:Target<R> {
        unsafe { self.target().bind(resource); }
        Binding(self, resource)
    }

    #[inline]
    pub fn map_bind<'a,R,U,F>(&'a mut self, resource: &'a R, f:F) -> U
    where T: Target<R>, F: FnOnce(Binding<'a,R,T>) -> U
    {
        f(self.bind(resource))
    }



}


#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub enum GLRef<'a,T:?Sized,A:buffer::BufferStorage> {
    Ref(&'a T),
    Buf(buffer::Slice<'a,T,A>)
}

impl<'a,'b:'a,T:?Sized,A:buffer::BufferStorage> From<&'a GLRef<'b,T,A>> for GLRef<'b,T,A> {
    fn from(r:&'a GLRef<'b,T,A>) -> Self {
        match r {
            GLRef::Ref(ptr) => GLRef::Ref(ptr),
            GLRef::Buf(ptr) => GLRef::Buf(*ptr),
        }
    }
}

impl<'a,'b:'a,T:?Sized,A:buffer::BufferStorage> From<&'a GLMut<'b,T,A>> for GLRef<'a,T,A> {
    fn from(r:&'a GLMut<'b,T,A>) -> Self {
        match r {
            GLMut::Mut(ptr) => GLRef::Ref(ptr),
            GLMut::Buf(ptr) => GLRef::Buf(ptr.as_slice()),
        }
    }
}

impl<'a,T:?Sized,A:buffer::BufferStorage> From<GLMut<'a,T,A>> for GLRef<'a,T,A> {
    fn from(r:GLMut<'a,T,A>) -> Self {
        match r {
            GLMut::Mut(ptr) => GLRef::Ref(ptr),
            GLMut::Buf(ptr) => GLRef::Buf(ptr.into()),
        }
    }
}

impl<'a,T:?Sized,A:buffer::BufferStorage> GLRef<'a,T,A> {
    pub fn size(&self) -> usize {
        match self {
            Self::Ref(ptr) => ::std::mem::size_of_val(ptr),
            Self::Buf(ptr) => ptr.size(),
        }
    }
}

impl<'a,T,A:buffer::BufferStorage> GLRef<'a,[T],A> {
    pub fn is_empty(&self) -> bool { self.len()==0 }
    pub fn len(&self) -> usize {
        match self {
            Self::Ref(ptr) => ptr.len(),
            Self::Buf(ptr) => ptr.len(),
        }
    }
}

impl<'a,F:format::SpecificCompressed,A:buffer::BufferStorage> GLRef<'a,pixels::CompressedPixels<F>,A> {
    pub fn is_empty(&self) -> bool { self.len()==0 }
    pub fn len(&self) -> usize {
        match self {
            Self::Ref(ptr) => ptr.len(),
            Self::Buf(ptr) => ptr.len(),
        }
    }
}

pub enum GLMut<'a,T:?Sized,A:buffer::BufferStorage> {
    Mut(&'a mut T),
    Buf(buffer::SliceMut<'a,T,A>)
}

impl<'a,'b:'a,T:?Sized,A:buffer::BufferStorage> From<&'a mut GLMut<'b,T,A>> for GLMut<'a,T,A> {
    fn from(r:&'a mut GLMut<'b,T,A>) -> Self {
        match r {
            GLMut::Mut(ptr) => GLMut::Mut(ptr),
            GLMut::Buf(ptr) => GLMut::Buf(ptr.as_mut_slice()),
        }
    }
}

impl<'a,T:?Sized,A:buffer::BufferStorage> GLMut<'a,T,A> {
    pub fn size(&self) -> usize {
        match self {
            Self::Mut(ptr) => ::std::mem::size_of_val(ptr),
            Self::Buf(ptr) => ptr.size(),
        }
    }
}

impl<'a,T,A:buffer::BufferStorage> GLMut<'a,[T],A> {
    pub fn is_empty(&self) -> bool { self.len()==0 }
    pub fn len(&self) -> usize {
        match self {
            Self::Mut(ptr) => ptr.len(),
            Self::Buf(ptr) => ptr.len(),
        }
    }
}

impl<'a,F:format::SpecificCompressed,A:buffer::BufferStorage> GLMut<'a,pixels::CompressedPixels<F>,A> {
    pub fn is_empty(&self) -> bool { self.len()==0 }
    pub fn len(&self) -> usize {
        match self {
            Self::Mut(ptr) => ptr.len(),
            Self::Buf(ptr) => ptr.len(),
        }
    }
}
