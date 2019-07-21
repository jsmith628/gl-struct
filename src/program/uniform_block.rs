
use super::*;

use std::marker::PhantomData;
use std::any::Any;
use std::mem::*;

pub struct UniformBlock<T:?Sized> {
    pub(super) program: GLuint,
    pub(super) index: GLuint,
    pub(super) ty: PhantomData<Box<T>>
}

impl UniformBlock<dyn Any> {

    unsafe fn get_uniform_block_iv(&self, pname:GLenum) -> GLint {
        let mut dest = MaybeUninit::uninit();
        gl::GetActiveUniformBlockiv(self.program, self.index, pname, dest.as_mut_ptr());
        dest.assume_init()
    }

    pub fn name_length(&self) -> GLuint { unsafe {self.get_uniform_block_iv(gl::UNIFORM_BLOCK_NAME_LENGTH) as GLuint} }
    pub fn binding(&self) -> GLuint { unsafe {self.get_uniform_block_iv(gl::UNIFORM_BLOCK_BINDING) as GLuint} }
    pub fn data_size(&self) -> GLuint { unsafe {self.get_uniform_block_iv(gl::UNIFORM_BLOCK_DATA_SIZE) as GLuint} }
    pub fn active_uniforms(&self) -> GLuint { unsafe {self.get_uniform_block_iv(gl::UNIFORM_BLOCK_ACTIVE_UNIFORMS) as GLuint} }

    pub fn referenced_by(&self, shader:ShaderType) -> bool {
        use ShaderType::*;
        unsafe {
            self.get_uniform_block_iv(
                match shader {
                    Vertex => gl::UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER,
                    TessControl => gl::UNIFORM_BLOCK_REFERENCED_BY_TESS_CONTROL_SHADER,
                    TessEval => gl::UNIFORM_BLOCK_REFERENCED_BY_TESS_EVALUATION_SHADER,
                    Geometry => gl::UNIFORM_BLOCK_REFERENCED_BY_GEOMETRY_SHADER,
                    Fragment => gl::UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER,
                    Compute => gl::UNIFORM_BLOCK_REFERENCED_BY_COMPUTE_SHADER,
                }
            ) != 0
        }
    }

    pub fn get_name(&self) -> String {
        unsafe {
            get_resource_string(self.program, self.index, self.name_length(), gl::GetActiveUniformBlockName, "Malformed Uniform Name")
        }
    }

    pub fn get_uniform_indices(&self) -> Box<[Uniform<dyn Any>]> {
        unsafe {
            let len = self.active_uniforms();
            let mut list = Vec::with_capacity(len as usize);
            list.set_len(len as usize);

            gl::GetActiveUniformBlockiv(
                self.program, self.index,
                gl::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES,
                &mut list[0]
            );

            list.into_iter().map(|i| Uniform {program: self.program, index: i as GLuint, ty: PhantomData} ).collect()
        }
    }

}

impl<T:?Sized> !Send for UniformBlock<T> {}
impl<T:?Sized> !Sync for UniformBlock<T> {}
