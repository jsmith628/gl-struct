
use super::*;

use std::convert::TryInto;
use std::mem::forget;

glenum! {
    pub enum ShaderType {
        [Vertex VERTEX_SHADER "Vertex Shader"],
        [TessControl TESS_CONTROL_SHADER "Tesselation Control Shader"],
        [TessEval TESS_EVALUATION_SHADER "Tesselation Evaluation Shader"],
        [Geometry GEOMETRY_SHADER "Geometry Shader"],
        [Fragment FRAGMENT_SHADER "Fragment Shader"],
        [Compute COMPUTE_SHADER "Compute Shader"]
    }

    pub enum ShaderParameter {
        [ShaderType SHADER_TYPE "Shader Type"],
        [DeleteStatus DELETE_STATUS "Delete Status"],
        [CompileStatus COMPILE_STATUS "Compile Status"],
        [InfoLogLength INFO_LOG_LENGTH "Info Log Length"],
        [ShaderSourceLength SHADER_SOURCE_LENGTH "Shader Source Length"]
        // [SPIRVBinary SPIR_V_BINARY "SPIR-V Binary"]
    }
}

pub struct Shader(GLuint);

impl Shader {

//TODO something for ReleaseShaderCompiler as GetShaderPrecisionFormat

    pub fn create(_gl: &GL20, ty: ShaderType) -> Result<Self, GLError> {
        unsafe {
            gl::GetError();//flush the current errors
            let id = gl::CreateShader(ty.into());
            if (gl::GetError() | gl::INVALID_ENUM) != 0 {
                Err(GLError::InvalidEnum(ty.into(), "Shader Type in this GL version".to_owned()))
            } else {
                Ok(Shader(id))
            }
        }
    }

    pub fn id(&self) -> GLuint {self.0}

    pub fn leak(self) -> GLuint {
        let id = self.id();
        forget(self);
        id
    }

    pub fn is(id: GLuint) -> bool { unsafe {gl::IsShader::is_loaded() && gl::IsShader(id)!=0} }

    pub unsafe fn from_raw(id: GLuint) -> Option<Self> {
        if Self::is(id) { Some(Shader(id)) } else { None }
    }

    pub fn get_shader_int(&self, p: ShaderParameter) -> GLint {
        unsafe {
            let mut val:GLint = 0;
            gl::GetShaderiv(self.0, p.into(), &mut val as *mut GLint);
            val
        }
    }

    pub fn shader_type(&self) -> ShaderType {
        (self.get_shader_int(ShaderParameter::ShaderType) as GLenum).try_into().unwrap()
    }

    pub fn compile_status(&self) -> bool { self.get_shader_int(ShaderParameter::CompileStatus) != 0 }
    pub fn delete_status(&self) -> bool { self.get_shader_int(ShaderParameter::DeleteStatus) != 0 }
    pub fn info_log_length(&self) -> GLuint { self.get_shader_int(ShaderParameter::InfoLogLength) as GLuint }
    pub fn shader_source_length(&self) -> GLuint { self.get_shader_int(ShaderParameter::ShaderSourceLength) as GLuint }

    pub fn info_log(&mut self) -> String {
        unsafe {
            get_resource_string(self.0, self.info_log_length(), gl::GetShaderInfoLog, "Malformatted shader info log")
        }
    }

    pub fn get_source(&self) -> String {
        unsafe {
            get_resource_string(self.0, self.shader_source_length(), gl::GetShaderSource, "Malformatted shader source")
        }
    }

    pub fn source(&mut self, src: &[&str]) {
        if src.len() == 0 {return;}
        let lengths = src.iter().map(|s| s.len() as GLint).collect::<Vec<_>>();
        let ptrs = src.iter().map(|s| s.as_ptr() as *const GLchar).collect::<Vec<_>>();
        unsafe {
            gl::ShaderSource(
                self.0,
                src.len() as GLsizei,
                &(ptrs[0]) as *const *const GLchar,
                &lengths[0] as *const GLint
            )
        }
    }

    pub fn compile(&mut self) -> Result<(),GLError> {
        unsafe { gl::CompileShader(self.0); }
        if !self.compile_status() {
            Err(GLError::ShaderCompilation(self.0, self.shader_type(), self.info_log()))
        } else {
            Ok(())
        }
    }

    pub fn from_source(gl: &GL20, ty: ShaderType, src: &[&str]) -> Result<Self, GLError> {
        //create the shader
        let mut s = Shader::create(gl, ty)?;

        //give the source code to the GL
        s.source(src);

        //compile, test, and return
        let result = s.compile();
        result.map(|_| s)
    }

    pub fn delete(self) -> bool {
        unsafe { gl::DeleteShader(self.0); }
        let deleted = self.delete_status();
        forget(self);
        deleted
    }

}

impl Drop for Shader {
    fn drop(&mut self) { unsafe { gl::DeleteShader(self.0); } }
}

impl !Send for Shader {}
impl !Sync for Shader {}
