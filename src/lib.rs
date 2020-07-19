#![feature(core_intrinsics)]
#![feature(optin_builtin_traits)]
#![feature(ptr_offset_from)]
#![feature(untagged_unions)]
#![feature(concat_idents)]
#![feature(specialization)]
#![feature(allocator_api)]
#![feature(box_into_raw_non_null)]
#![feature(trace_macros)]
#![feature(unsize)]
#![feature(coerce_unsized)]
#![feature(const_fn)]
#![feature(maybe_uninit_ref)]
#![feature(trait_alias)]
#![feature(marker_trait_attr)]
#![feature(new_uninit)]
#![feature(get_mut_unchecked)]
#![feature(arbitrary_enum_discriminant)]
#![feature(never_type)]
#![feature(maybe_uninit_slice)]
#![recursion_limit="32768"]

pub extern crate gl;
pub extern crate num_traits;
#[cfg(feature = "glfw-context")] extern crate glfw;
#[cfg(feature = "glutin-context")] extern crate glutin;

#[macro_use] extern crate bitflags;
#[macro_use] extern crate derivative;

use gl::types::*;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Debug, Formatter};
use std::hash::Hash;

pub use gl_enum::*;

#[macro_use] mod gl_enum;
#[macro_use] mod macros;
#[macro_use] pub mod glsl;

pub mod context;
pub mod object;

pub mod image;
pub mod pixel;

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
        [Float FLOAT "Float"]
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

pub trait Bit { const VALUE:bool; }
pub struct High;
pub struct Low;

impl Bit for High { const VALUE:bool = true; }
impl Bit for Low { const VALUE:bool = false; }

#[marker] pub trait BitMasks<B:Bit>: Bit {}
impl<B:Bit> BitMasks<Low> for B {}
impl<B:Bit> BitMasks<B> for High {}

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
