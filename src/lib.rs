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
#![feature(maybe_uninit_ref)]
#![feature(non_exhaustive)]
#![feature(never_type)]
#![recursion_limit="8192"]

pub extern crate gl;
#[cfg(feature = "glfw-context")] pub extern crate glfw;
#[cfg(feature = "glutin-context")] pub extern crate glutin;

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

pub use gl_enum::*;

#[macro_use] mod gl_enum;

pub use program::*;
pub use glsl::*;

#[macro_use] pub mod glsl;

pub mod context;
pub mod object;
pub mod program;

pub mod format;

#[derive(Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GLError {
    ShaderCompilation(GLuint, ShaderType, String),
    ProgramLinking(GLuint, String),
    ProgramValidation(GLuint, String),
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
            GLError::ShaderCompilation(id, ty, log) => write!(f, "{} #{} compilation error: {}", ty, id, log),
            GLError::ProgramLinking(id, log) => write!(f, "Program #{} link error with Program: {}", id, log),
            GLError::ProgramValidation(id, log) => write!(f, "Program #{} validation error: {}", id, log),
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
