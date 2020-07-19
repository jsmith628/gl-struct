use super::*;
use context::*;

use std::marker::PhantomData;
use std::convert::TryInto;

pub use self::buffer::*;
pub use self::texture::*;
pub use self::renderbuffer::*;
pub use self::vertex_array::*;
pub use self::framebuffer::*;
pub use self::sampler::*;
// pub use self::query::*;
// pub use self::sync::*;

pub mod buffer;
pub mod texture;
pub mod renderbuffer;
pub mod vertex_array;
pub mod framebuffer;
pub mod sampler;
// pub mod query;
pub mod sync;

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

pub trait Target<R>: Copy + Eq + Hash + Debug + Display {

    fn target_id(self) -> GLenum;

    unsafe fn bind(self, obj: &R);
    unsafe fn unbind(self);

    ///
    ///Constructs a new binding location with the given target
    ///
    ///# Unsafety
    ///It is up to the caller to guarrantee that this is the only location with the given
    ///[binding target](Target) at the given time
    #[inline] unsafe fn as_loc(self) -> BindingLocation<R,Self> { BindingLocation(self, PhantomData) }
}

///An object that owns a [Target] to a glBind* function for a resource `R`
#[derive(PartialEq, Eq, Hash)]
pub struct BindingLocation<R,T:Target<R>>(pub(crate) T, PhantomData<*const R>);

///An object that owns a binding of a [Resource] to a particular [BindingLocation] and unbinds it when leaving scope
pub struct Binding<'a,R,T:Target<R>>(pub(crate) &'a BindingLocation<R,T>, pub(crate) &'a R);

impl<'a,R,T:Target<R>> Binding<'a,R,T> {
    #[inline] pub fn target(&self) -> T { self.0.target() }
    #[inline] pub fn resource(&self) -> &R { self.1 }
    #[inline] pub fn target_id(&self) -> GLenum { self.0.target_id() }
}

impl<'a,R,T:Target<R>> Drop for Binding<'a,R,T> {
    #[inline] fn drop(&mut self) { unsafe { self.target().unbind() } }
}

impl<R,T:Target<R>> BindingLocation<R,T> {

    ///The [target](Target) of this location
    pub fn target(&self) -> T { self.0 }

    ///The the [GLenum] corresponding to this location's target
    pub fn target_id(&self) -> GLenum { self.0.target_id() }

    ///
    ///Constructs a new binding location with the given target
    ///
    ///# Unsafety
    ///It is up to the caller to guarrantee that this is the only location with the given
    ///[binding target](Target) at the given time
    ///
    #[inline]
    pub unsafe fn new(target: T) -> Self {BindingLocation(target, PhantomData)}

    ///
    ///A wrapper of glBind* for `R` using an owned resource
    ///
    ///The fact that it is owned means that we can bind without checking validity of the resource
    ///id. Furthermore, for the same reasons as [bind_raw](BindingLocation::bind_raw), this method is actually safe
    ///
    #[inline]
    pub fn bind<'a>(&'a mut self, resource: &'a R) -> Binding<'a,R,T> {
        unsafe { self.target().bind(resource); }
        Binding(self, resource)
    }

    #[inline]
    pub fn map_bind<'a, U, F:FnOnce(Binding<'a,R,T>)->U>(&'a mut self, resource: &'a R, f:F) -> U {
        f(self.bind(resource))
    }



}
