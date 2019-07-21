
use super::*;

use std::marker::PhantomData;
use std::any::Any;
use std::mem::*;

pub struct Uniform<T:?Sized> {
    pub(super) program: GLuint,
    pub(super) index: GLuint,
    pub(super) ty: PhantomData<Box<T>>
}

impl Uniform<dyn Any> {

    unsafe fn get_uniform_iv(&self, pname:GLenum) -> GLint {
        let mut dest = MaybeUninit::uninit();
        gl::GetActiveUniformsiv(self.program, 1, &self.index, pname, dest.as_mut_ptr());
        dest.assume_init()
    }

    unsafe fn get_uniform_iv_option(&self, pname:GLenum) -> Option<GLuint> {
        let param = self.get_uniform_iv(pname);
        if param < 0 { None } else { Some(param as GLuint) }
    }

    pub fn type_token(&self) -> GLSLTypeToken { unsafe {(self.get_uniform_iv(gl::UNIFORM_TYPE) as GLuint).try_into().unwrap()} }
    pub fn array_size(&self) -> GLuint { unsafe {self.get_uniform_iv(gl::UNIFORM_SIZE) as GLuint} }
    pub fn name_length(&self) -> GLuint { unsafe {self.get_uniform_iv(gl::UNIFORM_NAME_LENGTH) as GLuint} }
    pub fn array_stride(&self) -> Option<GLuint> { unsafe {self.get_uniform_iv_option(gl::ARRAY_STRIDE)} }
    pub fn matrix_stride(&self) -> Option<GLuint> { unsafe {self.get_uniform_iv_option(gl::MATRIX_STRIDE)} }
    pub fn is_row_major(&self) -> bool { unsafe {self.get_uniform_iv(gl::IS_ROW_MAJOR) != 0} }
    pub fn offset(&self) -> Option<GLuint> { unsafe {self.get_uniform_iv_option(gl::OFFSET)} }
    pub fn block_index(&self) -> Option<GLuint> { unsafe {self.get_uniform_iv_option(gl::BLOCK_INDEX)}  }
    pub fn atomic_counter_buffer_index(&self) -> Option<GLuint> { unsafe {self.get_uniform_iv_option(gl::ATOMIC_COUNTER_BUFFER_INDEX)} }

    pub fn get_name(&self) -> String {
        unsafe {
            get_resource_string(self.program, self.index, self.name_length(), gl::GetActiveUniformName, "Malformed Uniform Name")
        }
    }

}

impl<T:?Sized> !Send for Uniform<T> {}
impl<T:?Sized> !Sync for Uniform<T> {}
