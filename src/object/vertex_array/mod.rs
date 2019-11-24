use super::*;

use std::marker::PhantomData;
use std::mem::MaybeUninit;

use crate::object::buffer::AttribArray;
use glsl::GLSLType;


pub use self::attrib_format::*;
mod attrib_format;

pub trait Vertex: GLSLType {}


pub struct VertexArray<'a,V:Vertex> {
    id: GLuint,
    buffers: PhantomData<(&'a Buffer<GLuint, ReadOnly>, AttribArray<'a,V>)>
}


pub struct VertexAttrib<'a, T:GLSLType+Vertex> {
    vaobj: GLuint,
    index: GLuint,
    reference: PhantomData<&'a mut VertexArray<'a,T>>
}

impl<'a,T:GLSLType+Vertex> VertexAttrib<'a,T> {
    #[inline] pub fn index(&self) -> GLuint { self.index }

    unsafe fn get(&self, pname:GLenum) -> GLint {
        let mut dest = MaybeUninit::uninit();
        if gl::GetVertexArrayIndexediv::is_loaded() {
            gl::GetVertexArrayIndexediv(self.vaobj, self.index, pname, dest.as_mut_ptr());
        } else {
            gl::BindVertexArray(self.vaobj);
            gl::GetVertexAttribiv(self.index, pname, dest.as_mut_ptr());
            gl::BindVertexArray(0);
        }
        dest.assume_init()
    }

    unsafe fn get_64(&self, pname:GLenum) -> GLint64 {
        let mut dest = MaybeUninit::uninit();
        if gl::GetVertexArrayIndexed64iv::is_loaded() {
            gl::GetVertexArrayIndexed64iv(self.vaobj, self.index, pname, dest.as_mut_ptr());
            dest.assume_init()
        } else {
            self.get(pname) as GLint64
        }
    }

    pub fn array_enabled(&self) -> bool { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_ENABLED) != 0 } }
    pub fn array_size(&self) -> usize { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_SIZE) as usize } }
    pub fn array_stride(&self) -> usize { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_STRIDE) as usize } }
    //TODO type
    pub fn array_normalized(&self) -> bool { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_NORMALIZED) != 0 } }
    pub fn array_integer(&self) -> bool { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_INTEGER) != 0 } }
    pub fn array_long(&self) -> bool { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_LONG) != 0 } }
    pub fn array_divisor(&self) -> usize { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_DIVISOR) as usize } }
    pub fn relative_offset(&self) -> usize { unsafe { self.get_64(gl::VERTEX_ATTRIB_RELATIVE_OFFSET) as usize } }

    pub fn enable_array(&mut self) {
        unsafe {
            if gl::EnableVertexArrayAttrib::is_loaded() {
                gl::EnableVertexArrayAttrib(self.vaobj, self.index);
            } else {
                gl::BindVertexArray(self.vaobj);
                gl::EnableVertexAttribArray(self.index);
                gl::BindVertexArray(0);
            }
        }
    }

    pub fn disable_array(&mut self) {
        unsafe {
            if gl::DisableVertexArrayAttrib::is_loaded() {
                gl::DisableVertexArrayAttrib(self.vaobj, self.index);
            } else {
                gl::BindVertexArray(self.vaobj);
                gl::DisableVertexAttribArray(self.index);
                gl::BindVertexArray(0);
            }
        }
    }

    pub fn pointer(&mut self, pointer: AttribArray<'a,T>) {
        unsafe {
            gl::BindVertexArray(self.vaobj);



            gl::BindVertexArray(0);
        }
    }

    pub fn divisor(&mut self, divisor: GLuint) {
        unsafe {
            gl::BindVertexArray(self.vaobj);
            gl::VertexAttribDivisor(self.index, divisor);
            gl::BindVertexArray(0);
        }
    }

}
