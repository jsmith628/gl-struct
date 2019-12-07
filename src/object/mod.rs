use super::*;
use context::*;

use std::marker::PhantomData;
use std::convert::TryInto;

macro_rules! gl_resource{

    (@obj gen=$gl:ident) => {};
    (@fun gen=$gl:ident) => {
        #[inline]
        fn gen(_gl: &<Self as Object>::GL) -> Self {
            unsafe {
                let mut obj = ::std::mem::MaybeUninit::<Self>::uninit();
                gl::$gl(1, obj.get_mut().0 as *mut gl::types::GLuint);
                obj.assume_init()
            }
        }

        #[inline]
        fn gen_resources(_gl: &<Self as Object>::GL, count: gl::types::GLuint) -> Box<[Self]> {
            unsafe {
                let mut obj = Vec::<Self>::with_capacity(count as usize);
                obj.set_len(count as usize);
                gl::$gl(count as gl::types::GLsizei, &mut obj[0].0 as *mut gl::types::GLuint);
                obj.into_boxed_slice()
            }
        }
    };


    (@fun is=$gl:ident) => {};
    (@obj is=$gl:ident) => {
        #[inline] fn is(id: Self::Raw) -> bool { unsafe { gl::$gl(id) != gl::FALSE } }
    };

    (@obj bind=$gl:ident) => {};
    (@fun bind=$gl:ident) => {};

    (@obj gl=$GL:ident) => { type GL = $GL; };
    (@fun gl=$GL:ident) => { };

    (@obj target=$Target:ident) => { };
    (@obj target=!) => { };
    (@fun target=$Target:tt) => { type BindingTarget = $Target; };

    (@obj ident=$ident:ident) => { };
    (@fun ident=$ident:ident) => {
        const IDENTIFIER: crate::object::ResourceIdentifier = crate::object::ResourceIdentifier::$ident;
    };

    (@obj delete=$gl:ident) => {
        #[inline]
        fn delete(self) {
            unsafe { gl::$gl(1, &self.into_raw() as *const gl::types::GLuint); }
        }
    };
    (@fun delete=$gl:ident) => {
        #[inline]
        fn delete_resources(resources: Box<[Self]>) {
            unsafe {
                //the transmutation makes sure that we don't double-free
                let ids = ::std::mem::transmute::<Box<[Self]>, Box<[gl::types::GLuint]>>(resources);
                gl::$gl(ids.len() as gl::types::GLsizei, &ids[0] as *const gl::types::GLuint);
            }
        }
    };

    (@bind $ty:ident {$($tt:tt)*} bind=$gl:ident $($rest:tt)*) => { gl_resource!(@bind $ty {$($tt)* bind=$gl} $($rest)*); };
    (@bind $ty:ident {$($tt:tt)*} target=$Target:tt $($rest:tt)*) => { gl_resource!(@bind $ty {target=$Target $($tt)*} $($rest)*); };
    (@bind $ty:ident {$($tt:tt)*} $param:ident=$gl:ident $($rest:tt)*) => { gl_resource!(@bind $ty {$($tt)*} $($rest)*); };
    (@bind $ty:ident {target=$Target:ident bind=$gl:ident}) => {
        impl Target<$ty> for $Target {
            #[inline] fn target_id(self) -> GLenum {self.into()}
            #[inline] unsafe fn bind(self, obj:&$ty) {gl::$gl(self.into(), $crate::object::Resource::id(obj))}
            #[inline] unsafe fn unbind(self) {gl::$gl(self.into(), 0)}
        }
    };
    (@bind $ty:ident {target=!}) => {
        impl Target<$ty> for ! {
            #[inline] fn target_id(self) -> GLenum {self}
            #[inline] unsafe fn bind(self, _:&$ty) {self}
            #[inline] unsafe fn unbind(self) {self}
        }
    };

    ({$($mod:tt)*} struct $name:ident {$($fun:ident=$gl:tt),*}) => {

        gl_resource!(@bind $name {} $($fun=$gl)*);

        #[repr(C)] $($mod)* struct $name(GLuint);

        unsafe impl $crate::object::Object for $name {

            type Raw = gl::types::GLuint;

            $(gl_resource!(@obj $fun=$gl);)*

            #[inline]
            unsafe fn from_raw(id: gl::types::GLuint) -> Option<Self> {
                if Self::is(id) { Some($name(id)) } else {None}
            }

            #[inline]
            fn into_raw(self) -> gl::types::GLuint {
                let id = self.id();
                ::std::mem::forget(self);
                id
            }

            fn label(&mut self, label: &str) -> Result<(),GLError> {
                object::object_label(self, label)
            }

            fn get_label(&self) -> Option<String> { object::get_object_label(self) }
        }

        unsafe impl $crate::object::Resource for $name {
            $(gl_resource!(@fun $fun=$gl);)*

            #[inline] fn id(&self) -> gl::types::GLuint { self.0 }

        }

        impl Drop for $name {
            #[inline] fn drop(&mut self) { $crate::object::Object::delete($name(self.0)); }
        }


    };

    ({$($mod:tt)*} #[$attr:meta] $($tt:tt)*) => {gl_resource!({$($mod)* #[$attr]} $($tt)*);};
    ({$($mod:tt)*} $kw:ident($($args:tt)*) $($tt:tt)*) => {gl_resource!({$($mod)* $kw($($args)*)} $($tt)*);};
    ({$($mod:tt)*} $kw:ident $($tt:tt)*) => {gl_resource!({$($mod)* $kw} $($tt)*);};

    ($kw:ident $($tt:tt)*) => {gl_resource!({} $kw $($tt)*);};
    (#[$attr:meta] $($tt:tt)*) => {gl_resource!({} #[$attr] $($tt)*);};
}

pub use self::buffer::*;
pub use self::texture::*;
pub use self::renderbuffer::*;
pub use self::vertex_array::*;
pub use self::sampler::*;
pub use self::query::*;
// pub use self::sync::*;

pub mod buffer;
pub mod texture;
pub mod renderbuffer;
pub mod vertex_array;
pub mod sampler;
pub mod query;
pub mod sync;


glenum! {
    pub enum ResourceIdentifier {
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

pub unsafe trait Object: Sized {
    ///The OpenGL version type that guarrantees that the functions required for initialization are loaded
    type GL: GLVersion;
    type Raw: Copy;

    ///
    ///Consumes this object and leaks its id
    ///
    ///As such, it is up to the caller to delete the resource or remake the object with
    ///[from_raw](Object::from_raw) to avoid a memory leak. Despite this however, this method is not
    ///considered unsafe for much the same reason [Box::into_raw] is not
    fn into_raw(self) -> Self::Raw;


    ///
    ///Constructs an object from a raw GL object id or [Option::None] if the id is not a name of
    ///
    ///
    ///This id should be one from a previous call to [into_raw](Object::into_raw) or an unowned object
    ///created manually otherwise there almost certainly will be a double-free. However, it is _not_
    ///unsafe to provide an invalid id or an id from a different type as OpenGL and the implementor
    ///should catch them and return a None
    ///
    ///# Unsafety
    ///
    ///Calling this on the same id twice will almost certainly cause a double-free and/or other memory
    ///issues from double-ownership
    ///
    unsafe fn from_raw(raw: Self::Raw) -> Option<Self>;

    ///Determines if a given id is the name of an OpenGL object of this type
    fn is(raw: Self::Raw) -> bool;

    ///
    ///Consumes this object and deletes its OpenGL resources.
    ///
    ///Do note though that if this object is queued for an operation (unless synchronization is
    ///overrided by various (unsafe) means) the object will continue to exist until those operations
    ///have completed. Furthermore, certain objects will continue to exist in memory if bound
    ///to certain targets similar to a reference counted smart-pointer.
    ///
    fn delete(self);

    fn label(&mut self, label: &str) -> Result<(), GLError>;

    fn get_label(&self) -> Option<String>;

}

pub(self) fn object_label<R:Resource>(this:&mut R, label:&str) -> Result<(),GLError> {
    use std::mem::MaybeUninit;

    unsafe {
        if gl::ObjectLabel::is_loaded() {
            let mut max_length = MaybeUninit::uninit();
            gl::GetIntegerv(gl::MAX_LABEL_LENGTH, max_length.as_mut_ptr());
            if max_length.assume_init() >= label.len() as GLint {
                gl::ObjectLabel(
                    R::IDENTIFIER as GLenum,
                    this.id(), label.len() as GLsizei, label.as_ptr() as *const GLchar
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

pub(self) fn get_object_label<R:Resource>(this:&R) -> Option<String>{
    use std::mem::MaybeUninit;
    use std::ptr::*;

    unsafe {
        if gl::GetObjectLabel::is_loaded() {
            //get the length of the label
            let mut length = MaybeUninit::uninit();
            gl::GetObjectLabel(
                R::IDENTIFIER as GLenum, this.id(), 0, length.as_mut_ptr(), null_mut()
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
                    R::IDENTIFIER as GLenum,
                    this.id(), length as GLsizei, null_mut(), bytes.as_mut_ptr() as *mut GLchar
                );

                //since label() loads from a &str, we can assume the returned bytes are valid utf8
                Some(String::from_utf8_unchecked(bytes))
            }

        } else {
            None
        }
    }
}

///
///An OpenGL resource object that follows the standard [glGen*](Resource::gen),
///[glIs*](Resource::is), and [glDelete*](Resource::delete) pattern
///
///# Unsafety
///
///It is up to the caller to implement [Drop] individually in order to call [delete](Self::delete)
///when leaving scope, and to guarrantee that the [GL](Resource::GL) object properly loads all necessary
///functions
///
pub unsafe trait Resource: Object<Raw=GLuint> {

    type BindingTarget: Target<Self>;

    const IDENTIFIER: ResourceIdentifier;

    ///
    ///The identification of the object used internally by OpenGL that is returned by the gen method.
    ///
    ///This identification number is unique to this object for as long as it exists, and thus,
    ///barring unsafe shenanigans, no other object will share it. In fact, the id is unique even
    ///outside its type and for all OpenGL resources of this particular form. Of course though, once
    ///the object has been deleted, the id can be reused for another object or even another type.
    ///
    ///Note that this is intended to be analogous to creating a raw pointer from a reference. As such,
    ///this method is actually _not_ unsafe, as it does not inherently make any unsafe memory operations
    ///or even leak data (much like pointer creation). Rather, any functions operating the returned a
    ///raw id should be marked unsafe themselves since they can make no guarrantees of memory safety.
    ///
    fn id(&self) -> GLuint;

    ///Creates a new OpenGL resource of this type
    fn gen(gl: &<Self as Object>::GL) -> Self;

    ///Creates an array of new OpenGL resources
    fn gen_resources(gl: &<Self as Object>::GL, count: GLuint) -> Box<[Self]>;

    ///Returns true if the two OpenGL objects are the same _object_ without checking value-equivalence
    #[inline] fn obj_eq<R:Resource+?Sized>(&self, rhs:&R) -> bool {self.id()==rhs.id()}

    ///
    ///Consumes an array of objects and deletes their OpenGL resources.
    ///
    ///Do note though that if this object is queued for an operation (unless synchronization is
    ///overrided by various (unsafe) means) the object will continue to exist until those operations
    ///have completed. Furthermore, certain objects will continue to exist in memory if bound
    ///to certain targets similar to a reference counted smart-pointer.
    ///
    fn delete_resources(resouces: Box<[Self]>);

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
pub struct BindingLocation<R,T:Target<R>>(pub(crate) T, PhantomData<R>);

///An object that owns a binding of a [Resource] to a particular [BindingLocation] and unbinds it when leaving scope
pub struct Binding<'a,R,T:Target<R>>(pub(crate) &'a BindingLocation<R,T>, pub(crate) &'a R);

impl<'a,R,T:Target<R>> Binding<'a,R,T> {
    #[inline] pub fn target(&self) -> T { self.0.target() }
    #[inline] pub fn resource(&self) -> &R { self.1 }
    #[inline] pub fn target_id(&self) -> GLenum { self.0.target_id() }
}

impl<'a,R,T:Target<R>> !Sync for Binding<'a,R,T> {}
impl<'a,R,T:Target<R>> !Send for Binding<'a,R,T> {}
impl<'a,R,T:Target<R>> Drop for Binding<'a,R,T> {
    #[inline] fn drop(&mut self) { unsafe { self.target().unbind() } }
}

impl<R,T:Target<R>> !Sync for BindingLocation<R,T> {}
impl<R,T:Target<R>> !Send for BindingLocation<R,T> {}

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
