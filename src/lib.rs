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


///A helper macro for contructing by-value stack arrays from a list comprehension
macro_rules! arr {
    (for $i:ident in 0..$n:literal { $expr:expr }) => { arr![for $i in 0..($n) { $expr }]};
    (for $i:ident in 0..=$n:literal { $expr:expr }) => { arr![for $i in 0..($n+1) { $expr }]};
    (for $i:ident in 0..=($n:expr) { $expr:expr }) => { arr![for $i in 0..($n+1) { $expr }]};
    (for $i:ident in 0..($n:expr) { $expr:expr }) => {
        {
            //create a MaybeUninit containint the array
            let mut arr = ::std::mem::MaybeUninit::<[_;$n]>::uninit();

            //loop over the array and assign each entry according to the index
            for $i in 0..$n {

                //compute the value here because we don't want the unsafe block to transfer
                let val = $expr;

                //we use write() here because we don't want to drop the previous value
                #[allow(unused_unsafe)]
                unsafe { ::std::ptr::write(&mut (*arr.as_mut_ptr())[$i], val); }

            }

            #[allow(unused_unsafe)]
            unsafe { arr.assume_init() }
        }
    }
}

///a helper macro for looping over generic tuples
macro_rules! impl_tuple {

    //the start of the loop
    ($callback:ident) => {impl_tuple!({A:a B:b C:c D:d E:e F:f G:g H:h I:i J:j K:k} L:l $callback);};
    ($callback:ident @with_last) => {
        impl_tuple!({A:a B:b C:c D:d E:e F:f G:g H:h I:i K:k J:j} L:l $callback @with_last);
    };

    //the end of the loop
    ({} $callback:ident) => {};
    // ({} $T0:ident:$t0:ident $callback:ident ) => {};
    ({} $callback:ident @$($options:tt)*) => {};

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

#[macro_use] pub mod glsl;

pub mod context;
pub mod object;

pub mod format;

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
