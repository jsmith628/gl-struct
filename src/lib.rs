#![feature(core_intrinsics)]
// #![feature(optin_builtin_traits)]
#![feature(ptr_offset_from)]
#![feature(untagged_unions)]
#![feature(concat_idents)]
#![feature(specialization)]
#![feature(allocator_api)]
#![feature(result_map_or_else)]
#![feature(trace_macros)]
#![feature(unsize)]
#![feature(coerce_unsized)]
// #![feature(const_fn)]
#![allow(deprecated)]
#![recursion_limit="8192"]

pub extern crate gl;

#[macro_use] extern crate bitflags;

use gl::types::*;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Debug, Formatter};
use std::hash::Hash;

pub use program::*;
pub use glsl::*;
pub use buffer::*;

macro_rules! display_from_debug {
    ($name:ty) => {
        impl ::std::fmt::Display for $name {
            #[inline]
            fn fmt(&self,f:  &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                ::std::fmt::Debug::fmt(self, f)
            }
        }
    }
}

macro_rules! glenum {

    ({$($kw:tt)*} enum $name:ident {$($(#[$attr:meta])* $item:ident),*} $($tt:tt)*) => {
        glenum!({#[allow(non_camel_case_types)] $($kw)*} enum $name {$($(#[$attr])* [$item $item stringify!($item)]),*} $($tt)*);
    };

    ({$($kw:tt)*} enum $name:ident {$($(#[$attr:meta])* [$item:ident $gl:ident $pretty:expr] ),*} $($tt:tt)*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        $($kw)* enum $name {
            $(
                $(#[$attr])*
                $item = ::gl::$gl as isize
            ),*
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match self {
                    $($name::$item => write!(f, $pretty)),*
                }
            }
        }

        impl From<$name> for gl::types::GLenum {
            fn from(e: $name) -> ::gl::types::GLenum {
                match e {
                    $($name::$item => ::gl::$gl),*
                }
            }
        }

        impl ::std::convert::TryFrom<::gl::types::GLenum> for $name {
            type Error = GLError;
            fn try_from(e: ::gl::types::GLenum) -> Result<$name, GLError>{
                match e {
                    $(::gl::$gl => Ok($name::$item),)*
                    _ => Err(::GLError::InvalidEnum(e, stringify!($name).to_string()))
                }
            }
        }

        impl $crate::GLEnum for $name {}

        glenum!($($tt)*);

    };

    ({$($kws:tt)*} #[$attr:meta] $($tt:tt)*) => { glenum!({$($kws)* #[$attr]} $($tt)*); };
    ({$($kws:tt)*} $kw:ident($($path:tt)*) $($tt:tt)*) => { glenum!({$($kws)* $kw($($path)*)} $($tt)*); };
    ({$($kws:tt)*} $kw:ident $($tt:tt)*) => { glenum!({$($kws)* $kw} $($tt)*); };

    () => {};
    (# $($tt:tt)*) => {glenum!({} # $($tt)*);};
    ($kw:ident $($tt:tt)*) => {glenum!({} $kw $($tt)*);};

}

///a helper macro for looping over generic tuples
macro_rules! impl_tuple {

    //the start of the loop
    ($callback:ident) => {impl_tuple!({A:a B:b C:c D:d E:e F:f G:g H:h I:i K:k J:j} L:l $callback);};
    ($callback:ident @with_last) => {
        impl_tuple!({A:a B:b C:c D:d E:e F:f G:g H:h I:i K:k J:j} L:l $callback @with_last);
    };

    //the end of the loop
    ({} $callback:ident) => {};
    ({} $T0:ident:$t0:ident $callback:ident ) => {};
    ({} $T0:ident:$t0:ident $callback:ident @$($options:tt)*) => {};

    ({$($A:ident:$a:ident)*} $T0:ident:$t0:ident $callback:ident) => {
        $callback!($($A:$a)* $T0:$t0);
        impl_tuple!({} $($A:$a)* $callback);
    };

    ({$($A:ident:$a:ident)*} $T0:ident:$t0:ident $callback:ident @with_last) => {
        $callback!({$($A:$a)*} $T0:$t0);
        impl_tuple!({} $($A:$a)* $callback @with_last);
    };

    //find the last generic in order to remove it from the list
    ({$($A:ident:$a:ident)*} $T0:ident:$t0:ident $T1:ident:$t1:ident $($rest:tt)*) => {
        impl_tuple!({$($A:$a)* $T0:$t0} $T1:$t1 $($rest)*);
    };
}

macro_rules! check_loaded {
    ($gl_fun0:ident, $($gl_fun:ident),+; $expr:expr) => {
        check_loaded!($gl_fun0; check_loaded!($($gl_fun),+; $expr)).map_or_else(|e| Err(e), |ok| ok)
    };

    ($gl_fun:ident; $expr:expr) => {
        if $crate::gl::$gl_fun::is_loaded() {
            Ok($expr)
        } else {
            Err($crate::GLError::FunctionNotLoaded(concat!("gl", stringify!($gl_fun))))
        }
    }
}


macro_rules! gl_resource{

    (@fun gen=$gl:ident) => {
        #[inline]
        fn gen(_gl: &Self::GL) -> Self {
            unsafe {
                let mut obj = ::std::mem::uninitialized::<Self>();
                gl::$gl(1, &mut obj.0 as *mut gl::types::GLuint);
                obj
            }
        }

        #[inline]
        fn gen_resources(_gl: &Self::GL, count: gl::types::GLuint) -> Box<[Self]> {
            unsafe {
                let mut obj = Vec::<Self>::with_capacity(count as usize);
                obj.set_len(count as usize);
                gl::$gl(count as gl::types::GLsizei, &mut obj[0].0 as *mut gl::types::GLuint);
                obj.into_boxed_slice()
            }
        }
    };

    (@fun is=$gl:ident) => {
        #[inline] fn is(id: gl::types::GLuint) -> bool { unsafe { gl::$gl(id) != gl::FALSE } }
    };

    (@fun gl=$GL:ident) => { type GL = $GL; };
    (@fun target=$Target:ident) => { type BindingTarget = $Target; };

    (@fun delete=$gl:ident) => {
        #[inline]
        fn delete(self) {
            unsafe { gl::$gl(1, &self.into_raw() as *const gl::types::GLuint); }
        }

        #[inline]
        fn delete_resources(resources: Box<[Self]>) {
            unsafe {
                //the transmutation makes sure that we don't double-free
                let ids = ::std::mem::transmute::<Box<[Self]>, Box<[gl::types::GLuint]>>(resources);
                gl::$gl(ids.len() as gl::types::GLsizei, &ids[0] as *const gl::types::GLuint);
            }
        }
    };

    ({$($mod:tt)*} struct $name:ident {$($fun:ident=$gl:ident),*}) => {
        #[repr(C)] $($mod)* struct $name(GLuint);

        unsafe impl $crate::Resource for $name {
            $(gl_resource!(@fun $fun=$gl);)*

            #[inline] fn id(&self) -> gl::types::GLuint { self.0 }

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

        }

        impl Drop for $name {
            #[inline] fn drop(&mut self) { $crate::Resource::delete($name(self.0)); }
        }


    };

    ({$($mod:tt)*} #[$attr:meta] $($tt:tt)*) => {gl_resource!({$($mod)* #[$attr]} $($tt)*);};
    ({$($mod:tt)*} $kw:ident($($args:tt)*) $($tt:tt)*) => {gl_resource!({$($mod)* $kw($($args)*)} $($tt)*);};
    ({$($mod:tt)*} $kw:ident $($tt:tt)*) => {gl_resource!({$($mod)* $kw} $($tt)*);};

    ($kw:ident $($tt:tt)*) => {gl_resource!({} $kw $($tt)*);};
    (#[$attr:meta] $($tt:tt)*) => {gl_resource!({} #[$attr] $($tt)*);};
}

#[macro_use]
pub mod glsl;
pub mod program;
pub mod buffer;
// pub mod buffer_new;
// pub mod texture;

pub trait Surface: {
    fn is_active(&self) -> bool;
    fn make_current(&mut self) -> &mut Context;
}

pub struct GLProvider { _private: () }
pub struct GL2 { _private: () }
pub struct GL3 { _private: () }
pub struct GL4 { _private: () }

impl GLProvider {

    // #[inline] pub(crate) fn unchecked() -> &'static Self { &GLProvider { _private: () }  }

    pub fn get_current() -> Result<GLProvider, ()> {
        //if glFinish isn't loaded, we can pretty safely assume nothing has
        if gl::Finish::is_loaded() {
            Ok(GLProvider{ _private: () })
        } else {
            Err(())
        }
    }

    pub unsafe fn load<F: FnMut(&'static str) -> *const GLvoid>(proc_addr: F) -> GLProvider {
        gl::load_with(proc_addr);
        GLProvider{ _private: () }
    }

    #[inline] pub fn upgrade(&self) -> Result<&GL2, GLError> {
        check_loaded!(
            GenBuffers, BindBuffer, DeleteBuffers, GetBufferParameteriv,
            BufferData, BufferSubData, GetBufferSubData, CopyBufferSubData,
            MapBuffer, UnmapBuffer;
            &GL2{_private:()}
        )
    }

}

impl GL2 {
    #[inline] pub fn upgrade(&self) -> Result<&GL3, GLError> {
        check_loaded!(MapBuffer, UnmapBuffer; &GL3{_private:()} )
    }
}

impl GL3 {
    #[inline] pub fn as_gl2(&self) -> &GL2 {&GL2{_private:()}}
    #[inline] pub fn upgrade(&self) -> Result<&GL4, GLError> {
        check_loaded!(BufferStorage, MapBufferRange; &GL4{_private:()} )
    }
}

impl GL4 {
    #[inline] pub fn as_gl2(&self) -> &GL2 {&GL2{_private:()}}
    #[inline] pub fn as_gl3(&self) -> &GL3 {&GL3{_private:()}}
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
pub unsafe trait Resource:Sized {

    ///The OpenGL version type that guarrantees that the functions required for initialization are loaded
    type GL;
    type BindingTarget: Target<Resource=Self>;

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

    ///
    ///Consumes this object and leaks its id
    ///
    ///As such, it is up to the caller to delete the resource or remake the object with
    ///[from_raw](Resource::from_raw) to avoid a memroy leak. Despite this however, this method is not
    ///considered unsafe for much the same reason [Box::into_raw] is not
    fn into_raw(self) -> GLuint;

    ///
    ///Constructs an object from a raw GL object id or [Option::None] if the id is not a name of
    ///
    ///
    ///This id should be one from a previous call to [into_raw](Resource::into_raw) or an unowned object
    ///created manually otherwise there almost certainly will be a double-free. However, it is _not_
    ///unsafe to provide an invalid id or an id from a different type as OpenGL and the implementor
    ///should catch them and return a None
    ///
    ///# Unsafety
    ///
    ///Calling this on the same id twice will almost certainly cause a double-free and/or other memory
    ///issues from double-ownership
    ///
    unsafe fn from_raw(id:GLuint) -> Option<Self>;


    ///Creates a new OpenGL resource of this type
    fn gen(gl: &Self::GL) -> Self;

    ///Creates an array of new OpenGL resources
    fn gen_resources(gl: &Self::GL, count: GLuint) -> Box<[Self]>;

    ///Determines if a given id is the name of an OpenGL resource of this type
    fn is(GLuint) -> bool;

    ///Returns true if the two OpenGL objects are the same _object_ without checking value-equivalence
    #[inline] fn obj_eq<R:Resource+?Sized>(&self, rhs:&R) -> bool {self.id()==rhs.id()}

    ///
    ///Consumes this object and deletes its OpenGL resource.
    ///
    ///Do note though that if this object is queued for an operation (unless synchronization is
    ///overrided by various (unsafe) means) the object will continue to exist until those operations
    ///have completed. Furthermore, certain objects will continue to exist in memory if bound
    ///to certain targets similar to a reference counted smart-pointer.
    ///
    fn delete(self);

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

///
///An OpenGL Enum that corresponds to target arguments in the glBind* functions
///
///# Unsafety
///
///It is up to the implementor to make sure that the possible enum values are valid arguments to
///whichever glBind* function is being called
///
pub unsafe trait Target: GLEnum {

    type Resource: Resource<BindingTarget=Self>;

    ///
    ///Binds the given resource id to this target
    ///
    ///# Unsafety
    ///
    ///The caller must make sure that the id corresponds to a valid resource id of a type valid for
    ///this particlular glBind* function. Furthermore, the caller must guarrantee that this target is
    ///valid for the configuration of the resource
    ///
    unsafe fn bind(self, id: GLuint);

    ///
    ///Constructs a new binding location with the given target
    ///
    ///# Unsafety
    ///It is up to the caller to guarrantee that this is the only location with the given
    ///[binding target](Target) at the given time
    #[inline]
    unsafe fn as_loc(self) -> BindingLocation<Self::Resource> {
        BindingLocation(self)
    }
}

///An object that owns a [Target] to a glBind* function for a resource `R`
#[derive(PartialEq, Eq, Hash)]
pub struct BindingLocation<R:Resource>(R::BindingTarget);

///An object that owns a binding of a [Resource] to a particular [BindingLocation] and unbinds it when leaving scope
pub struct Binding<'a,R:Resource>(&'a BindingLocation<R>, GLuint);

impl<'a,R:Resource> Binding<'a,R> {
    #[inline] pub fn target(&self) -> R::BindingTarget { self.0.target() }
    #[inline] pub fn target_id(&self) -> GLenum { self.0.target_id() }
    #[inline] pub fn resource_id(&self) -> GLuint { self.1 }
}

// impl<'a,R:Resource> !Sync for Binding<'a,R> {}
// impl<'a,R:Resource> !Send for Binding<'a,R> {}
impl<'a,R:Resource> Drop for Binding<'a,R> {
    #[inline] fn drop(&mut self) { unsafe { self.target().bind(0) } }
}

// impl<R:Resource> !Sync for BindingLocation<R> {}
// impl<R:Resource> !Send for BindingLocation<R> {}
impl<R:Resource> BindingLocation<R> {

    ///The [target](Target) of this location
    pub fn target(&self) -> R::BindingTarget { self.0 }

    ///The the [GLenum] corresponding to this location's target
    pub fn target_id(&self) -> GLenum { self.0.into() }

    ///
    ///Constructs a new binding location with the given target
    ///
    ///# Unsafety
    ///It is up to the caller to guarrantee that this is the only location with the given
    ///[binding target](Target) at the given time
    #[inline]
    pub unsafe fn new(target: R::BindingTarget) -> Self {BindingLocation(target)}

    ///
    ///A wrapper of glBind* for `R` using a raw resource id
    ///
    ///# Safety
    ///
    ///Do note that This method is actually _safe_. While it certainly appears as if it wouldn't be,
    ///all possible unsafe sources are already accounted for:
    /// * We already know that this is the only [BindingLocation] for its [Target] as its construction
    ///   is marked as unsafe
    /// * This does not violate memory safety as any object modification must happen from an unsafe
    ///   OpenGL call
    /// * Even if the `id` is not a valid resource name for `R`, we can easily check with
    ///   a glIs* function
    /// * While not really unsafe _per se_, the resource will never remain bound outside its lifetime
    ///   due to the implementation of [Drop] on [Binding]
    /// * The buffer cannot be deleted before being unbound without running unsafe fuctions
    ///
    ///# Errors
    ///
    ///A [GLError::InvalidOperation] is returned if `id` is not a name of a resource of type `R`
    ///
    #[inline]
    pub fn bind_raw<'a>(&'a mut self, id: GLuint) -> Result<Binding<'a,R>, GLError> {
        if R::is(id) {
            unsafe { self.target().bind(id); }
            Ok(Binding(self, id))
        } else {
            Err(GLError::InvalidOperation("Cannot bind resource to the given target".to_string()))
        }
    }

    ///
    ///A wrapper of glBind* for `R` using an owned resource
    ///
    ///The fact that it is owned means that we can bind without checking validity of the resource
    ///id. Furthermore, for the same reasons as [bind_raw](BindingLocation::bind_raw), this method is actually safe
    ///
    #[inline]
    pub fn bind<'a>(&'a mut self, resource: &'a R) -> Binding<'a,R> {
        unsafe { self.target().bind(resource.id()); }
        Binding(self, resource.id())
    }

}






///
///A struct for keeping track of global GL state while
///enforcing rust-like borrow rules on things like gl settings
///and bind points
///
pub struct Context {
    _private: ::std::marker::PhantomData<*const ()>
}

impl Context {
    pub fn init(_gl: &GLProvider) -> Context {
        Context { _private: ::std::marker::PhantomData }
    }
}

// impl !Send for Context {}
// impl !Sync for Context {}

glenum! {
    pub enum IntType {
        [Byte BYTE "Byte"],
        [UByte UNSIGNED_BYTE "UByte"],
        [Short SHORT "Short"],
        [UShort UNSIGNED_SHORT "UShort"],
        [Int INT "Int"],
        [UInt UNSIGNED_INT "UInt"]
    }

    pub enum FloatType {
        [Half HALF_FLOAT "Half"],
        [Float FLOAT "FLoat"]
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

impl FloatType {
    #[inline]
    pub fn size_of(self) -> usize {
        match self {
            FloatType::Half => 2,
            FloatType::Float => 4,
        }
    }
}


pub trait GLEnum: Sized + Copy + Eq + Hash + Debug + Display + Into<GLenum> + TryFrom<GLenum, Error=GLError> {}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum GLError {
    ShaderCompilation(GLenum, ShaderType, String),
    ProgramLinking(GLenum, String),
    ProgramValidation(GLenum, String),
    InvalidEnum(GLenum, String),
    InvalidOperation(String),
    InvalidBits(GLbitfield, String),
    BufferCopySizeError(usize, usize),
    FunctionNotLoaded(&'static str)
}

display_from_debug!(GLError);
impl Debug for GLError {

    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GLError::ShaderCompilation(id, ty, log) => write!(f, "{} #{} compilation error: {}", ty, id, log),
            GLError::ProgramLinking(id, log) => write!(f, "Program #{} link error with Program: {}", id, log),
            GLError::ProgramValidation(id, log) => write!(f, "Program #{} validation error: {}", id, log),
            GLError::InvalidEnum(id, ty) => write!(f, "Invalid enum: #{} is not a valid {}", id, ty),
            GLError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            GLError::InvalidBits(id, ty) => write!(f, "Invalid bitfield: {:b} are not valid flags for {}", id, ty),
            GLError::FunctionNotLoaded(name) => write!(f, "{} not loaded", name),
            GLError::BufferCopySizeError(s, cap) =>
                write!(f, "Invalid Buffer Copy: Source size {} smaller than Destination capacity {}.
                (If you are using an array, try slicing first.)", s, cap),
        }
    }

}

/// A trait for type-level bools
pub trait Boolean {
    type Not: Boolean<Not=Self>;
    const VALUE: bool;
}

/// A type representing a `true` value
pub struct True;

/// A type representing a `false` value
pub struct False;

impl Boolean for True {
    type Not = False;
    const VALUE: bool = true;
}

impl Boolean for False {
    type Not = True;
    const VALUE: bool = false;
}
