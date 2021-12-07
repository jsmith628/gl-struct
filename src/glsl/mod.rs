
use super::*;
use std::slice::*;

pub use self::attrib_format::*;

#[macro_use]
mod glsl;
mod attrib_format;
pub mod glsl_type;

pub unsafe trait GLSLType: Sized + Copy + Debug {
    type AttributeFormat: AttribFormat;

    #[inline] fn uniform_locations() -> GLuint {1}
    #[inline] fn first_element_name(var: String) -> String { var }

    unsafe fn load_uniform(id: GLint, data: &Self) { Self::load_uniforms(id, from_ref(data)); }
    unsafe fn load_uniforms(id: GLint, data: &[Self]);
    unsafe fn get_uniform(p: GLuint, id:GLint) -> Self;

    #[inline]
    unsafe fn bind_attribute(attr: GLuint, format: Self::AttributeFormat, stride: usize, offset: usize) {
        format.bind_attribute(attr, stride, offset);
    }

    #[inline]
    unsafe fn set_attribute(attr: GLuint, format: Self::AttributeFormat, data: *const GLvoid) {
        format.set_attribute(attr, data);
    }

}

pub unsafe trait GLSLStruct { const SRC: &'static str; }
pub unsafe trait GLSLFunction<ReturnType, Params> { const SRC: &'static str; }

pub unsafe trait BlockLayout: Sized + Copy {}
pub unsafe trait Layout<B:BlockLayout> {}
pub unsafe trait AlignedVec4 {}

#[derive(Clone, Copy, Debug)] #[allow(non_camel_case_types)] pub struct std140;
#[derive(Clone, Copy, Debug)] #[allow(non_camel_case_types)] pub struct std430;
#[derive(Clone, Copy, Debug)] #[allow(non_camel_case_types)] pub struct shared;

unsafe impl BlockLayout for std140 {}
unsafe impl BlockLayout for std430 {}
unsafe impl BlockLayout for shared {}

pub trait GLSLData<T:GLSLType>: From<T> + Into<T> + AttributeData<T> {}
impl<T:GLSLType, G> GLSLData<T> for G where G: From<T> + Into<T> + AttributeData<T> {}

pub unsafe trait AttribFormat: Sized + Clone + Copy + PartialEq + Eq + Hash + Debug {
    fn size(self) -> usize;
    fn attrib_count(self) -> usize {1}
    unsafe fn bind_attribute(self, attr_id: GLuint, stride: usize, offset: usize);
    unsafe fn set_attribute(self, attr_id: GLuint, data: *const GLvoid);
}

pub trait AttributeData<T:GLSLType>: Sized + Copy {
    fn format() -> T::AttributeFormat;
}

pub trait AttributeValue<T:GLSLType>: GPUCopy { fn format(&self) -> T::AttributeFormat; }
impl<A:AttributeData<T>, T:GLSLType> AttributeValue<T> for A {
    #[inline] fn format(&self) -> T::AttributeFormat {A::format()}
}

pub trait GLSLSubroutine: Copy + Eq {
    fn function_name(&self) -> &'static ::std::ffi::CStr;
}
