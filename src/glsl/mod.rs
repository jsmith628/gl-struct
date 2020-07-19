
use ::*;
use gl::*;

use std::ops::*;
use std::slice::*;
use std::mem::*;

use num_traits::{Zero,One};

use crate::vertex_array::*;

pub use self::c_bool::*;
pub use self::conv::*;
pub use self::layout::*;
pub use self::ops::*;
pub use self::functions::*;

#[macro_use]
mod glsl;

mod c_bool;
mod conv;
mod layout;
mod ops;
mod glsl_type;
mod functions;

pub unsafe trait GLSLType: Sized + Copy + 'static {
    type AttribFormat: AttribFormat;

    #[inline] fn uniform_locations() -> GLuint {1}
    // #[inline] fn first_element_name(var: String) -> String { var }

    unsafe fn load_uniform(id: GLint, data: &Self) { Self::load_uniforms(id, from_ref(data)); }
    unsafe fn load_uniforms(id: GLint, data: &[Self]);
    unsafe fn get_uniform(p: GLuint, id:GLint) -> Self;

    // #[inline]
    // unsafe fn bind_attribute(attr: GLuint, format: Self::AttributeFormat, stride: usize, offset: usize) {
    //     format.bind_attribute(attr, stride, offset);
    // }

    // #[inline]
    // unsafe fn set_attribute(attr: GLuint, format: Self::AttributeFormat, data: *const GLvoid) {
    //     format.set_attribute(attr, data);
    // }

}

pub unsafe trait GLSLStruct { const SRC: &'static str; }
pub unsafe trait GLSLFunction<ReturnType, Params> { const SRC: &'static str; }

#[marker] pub unsafe trait BlockLayout: Sized + Copy {}
#[marker] pub unsafe trait Layout<B:BlockLayout> {}
#[marker] pub unsafe trait AlignedVec4 {}

#[derive(Clone, Copy, Debug)] #[allow(non_camel_case_types)] pub struct std140;
#[derive(Clone, Copy, Debug)] #[allow(non_camel_case_types)] pub struct std430;
#[derive(Clone, Copy, Debug)] #[allow(non_camel_case_types)] pub struct shared;

unsafe impl BlockLayout for std140 {}
unsafe impl BlockLayout for std430 {}
unsafe impl BlockLayout for shared {}

pub trait GLSLSubroutine: Copy + Eq {
    fn function_name(&self) -> &'static ::std::ffi::CStr;
}


macro_rules! glsl_type {

    () => {};

    ($(#[$attr:meta])* type $name:ident = $prim:ty; $($rest:tt)*) => {
        $(#[$attr])* #[allow(non_camel_case_types)] pub type $name = $prim;
        glsl_type!($($rest)*);
    };

    ( $(#[$attr:meta])* $name:ident = $prim:ty; $($rest:tt)*) => {
        $(#[$attr])*
        #[repr(C)]
        #[derive(Clone, Copy, PartialEq, Debug, Default)]
        #[allow(non_camel_case_types)]
        pub struct $name { value: $prim }

        // impl From<$prim> for $name { #[inline] fn from(v: $prim) -> Self { $name{value: v} } }
        // impl From<$name> for $prim { #[inline] fn from(v: $name) -> Self { v.value } }

        glsl_type!($($rest)*);
    };

    (#$align:literal $($rest:tt)*) => {glsl_type!(#[repr(align($align))] $($rest)*);};

}

glsl_type!{
    type void = ();
    type gl_bool = c_bool::c_bool;
    #8 bvec2 = [gl_bool; 2];
    #16 bvec3 = [gl_bool; 3];
    #16 bvec4 = [gl_bool; 4];

    type uint = GLuint;
    #8 uvec2 = [uint; 2];
    #16 uvec3 = [uint; 3];
    #16 uvec4 = [uint; 4];

    type int = GLint;
    #8 ivec2 = [int; 2];
    #16 ivec3 = [int; 3];
    #16 ivec4 = [int; 4];

    type float = GLfloat;
    #8 vec2 = [float; 2];
    #16 vec3 = [float; 3];
    #16 vec4 = [float; 4];
    mat2x2 = [vec2; 2];
    mat3x2 = [vec2; 3];
    mat4x2 = [vec2; 4];
    mat2x3 = [vec3; 2];
    mat3x3 = [vec3; 3];
    mat4x3 = [vec3; 4];
    mat2x4 = [vec4; 2];
    mat3x4 = [vec4; 3];
    mat4x4 = [vec4; 4];

    type double = GLdouble;
    #16 dvec2 = [double; 2];
    #32 dvec3 = [double; 3];
    #32 dvec4 = [double; 4];
    dmat2x2 = [dvec2; 2];
    dmat3x2 = [dvec2; 3];
    dmat4x2 = [dvec2; 4];
    dmat2x3 = [dvec3; 2];
    dmat3x3 = [dvec3; 3];
    dmat4x3 = [dvec3; 4];
    dmat2x4 = [dvec4; 2];
    dmat3x4 = [dvec4; 3];
    dmat4x4 = [dvec4; 4];

    type mat2 = mat2x2;
    type mat3 = mat3x3;
    type mat4 = mat4x4;
    type dmat2 = dmat2x2;
    type dmat3 = dmat3x3;
    type dmat4 = dmat4x4;

}
