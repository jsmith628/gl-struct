#![feature(core_intrinsics)]
#![feature(optin_builtin_traits)]
#![feature(ptr_offset_from)]
#![feature(untagged_unions)]
#![feature(concat_idents)]
#![feature(specialization)]
#![feature(allocator_api)]
#![feature(box_into_raw_non_null)]
#![feature(result_map_or_else)]
#![feature(trace_macros)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(const_fn)]
#![feature(type_alias_enum_variants)]
#![recursion_limit="8192"]

pub extern crate gl;

#[macro_use] extern crate macro_program;
#[macro_use] extern crate bitflags;

extern crate trait_arith;

use gl::types::*;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Debug, Formatter};
use std::hash::Hash;

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

pub use resources::*;
pub use gl_enum::*;

#[macro_use] mod resources;
#[macro_use] mod gl_enum;

pub use program::*;
pub use glsl::*;
pub use buffer::*;

#[macro_use] pub mod glsl;

pub mod program;
pub mod buffer;
pub mod buffer_new;
pub mod image_format;
pub mod texture;
pub mod renderbuffer;
pub mod sampler;

pub trait Surface: {
    fn is_active(&self) -> bool;
    fn make_current(&mut self) -> &mut Context;
}

pub trait GLProvider {
    fn try_as_gl1(&self) -> Result<&GL1,GLError>;
    fn try_as_gl2(&self) -> Result<&GL2,GLError>;
    fn try_as_gl3(&self) -> Result<&GL3,GLError>;
    fn try_as_gl4(&self) -> Result<&GL4,GLError>;
}

pub struct GL1 { _private: () }
pub struct GL2 { _private: () }
pub struct GL3 { _private: () }
pub struct GL4 { _private: () }

impl GL1 {

    pub fn get_current() -> Result<Self, ()> {
        //if glFinish isn't loaded, we can pretty safely assume nothing has
        if gl::Finish::is_loaded() {
            Ok(GL1{ _private: () })
        } else {
            Err(())
        }
    }

    pub unsafe fn load<F: FnMut(&'static str) -> *const GLvoid>(proc_addr: F) -> GL1 {
        gl::load_with(proc_addr);
        GL1{ _private: () }
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
    #[inline] pub fn as_gl1(&self) -> &GL1 {&GL1{_private:()}}
    #[inline] pub fn upgrade(&self) -> Result<&GL3, GLError> {
        check_loaded!(MapBuffer, UnmapBuffer; &GL3{_private:()} )
    }
}

impl GL3 {
    #[inline] pub fn as_gl1(&self) -> &GL1 {&GL1{_private:()}}
    #[inline] pub fn as_gl2(&self) -> &GL2 {&GL2{_private:()}}
    #[inline] pub fn upgrade(&self) -> Result<&GL4, GLError> {
        check_loaded!(BufferStorage, MapBufferRange; &GL4{_private:()} )
    }
}

impl GL4 {
    #[inline] pub fn as_gl1(&self) -> &GL1 {&GL1{_private:()}}
    #[inline] pub fn as_gl2(&self) -> &GL2 {&GL2{_private:()}}
    #[inline] pub fn as_gl3(&self) -> &GL3 {&GL3{_private:()}}
}

impl GLProvider for GL1 {
    #[inline] fn try_as_gl1(&self) -> Result<&GL1,GLError> {Ok(self)}
    #[inline] fn try_as_gl2(&self) -> Result<&GL2,GLError> {self.upgrade()}
    #[inline] fn try_as_gl3(&self) -> Result<&GL3,GLError> {self.try_as_gl2().and_then(|gl| gl.upgrade())}
    #[inline] fn try_as_gl4(&self) -> Result<&GL4,GLError> {self.try_as_gl3().and_then(|gl| gl.upgrade())}
}

impl GLProvider for GL2 {
    #[inline] fn try_as_gl1(&self) -> Result<&GL1,GLError> {Ok(self.as_gl1())}
    #[inline] fn try_as_gl2(&self) -> Result<&GL2,GLError> {Ok(self)}
    #[inline] fn try_as_gl3(&self) -> Result<&GL3,GLError> {self.upgrade()}
    #[inline] fn try_as_gl4(&self) -> Result<&GL4,GLError> {self.try_as_gl3().and_then(|gl| gl.upgrade())}
}

impl GLProvider for GL3 {
    #[inline] fn try_as_gl1(&self) -> Result<&GL1,GLError> {Ok(self.as_gl1())}
    #[inline] fn try_as_gl2(&self) -> Result<&GL2,GLError> {Ok(self.as_gl2())}
    #[inline] fn try_as_gl3(&self) -> Result<&GL3,GLError> {Ok(self)}
    #[inline] fn try_as_gl4(&self) -> Result<&GL4,GLError> {self.upgrade()}
}

impl GLProvider for GL4 {
    #[inline] fn try_as_gl1(&self) -> Result<&GL1,GLError> {Ok(self.as_gl1())}
    #[inline] fn try_as_gl2(&self) -> Result<&GL2,GLError> {Ok(self.as_gl2())}
    #[inline] fn try_as_gl3(&self) -> Result<&GL3,GLError> {Ok(self.as_gl3())}
    #[inline] fn try_as_gl4(&self) -> Result<&GL4,GLError> {Ok(self)}
}

///
///A struct for keeping track of global GL state while
///enforcing rust-like borrow rules on things like gl settings
///and bind points
///
pub struct Context {

}

impl Context {
    pub fn init(_gl: &GL1) -> Context {
        Context {}
    }
}

impl !Send for Context {}
impl !Sync for Context {}

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
