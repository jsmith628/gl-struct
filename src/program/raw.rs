
use super::*;

use std::rc::*;
use std::mem::forget;

glenum! {
    pub enum ProgramParameter {
        [DeleteStatus DELETE_STATUS "Delete Status"],
        [LinkStatus LINK_STATUS "Delete Status"],
        [ValidateStatus VALIDATE_STATUS "Validate Status"],
        [InfoLogLength INFO_LOG_LENGTH "Info Log Length"],
        [AttachedShaders ATTACHED_SHADERS "Attached Shaders"],
        [ActiveAttributes ACTIVE_ATTRIBUTES "Active Attributes"],
        [ActiveAttributeMaxLength ACTIVE_ATTRIBUTE_MAX_LENGTH "Active Attribute Max Length"],
        [ActiveUniforms ACTIVE_UNIFORMS "Active Attributes"],
        [ActiveUniformMaxLength ACTIVE_UNIFORM_MAX_LENGTH "Active Uniform Max Length"],
        [TransformFeedbackBufferMode TRANSFORM_FEEDBACK_BUFFER_MODE "Transform Feedback Buffer Mode"],
        [TransformFeedbackVaryings TRANSFORM_FEEDBACK_VARYINGS "Transform Feedback Varyings"],
        [TransformFeedbackVaryingMaxLength TRANSFORM_FEEDBACK_VARYING_MAX_LENGTH "Transform Feedback Varying Max Length"],
        [ActiveUniformBlocks ACTIVE_UNIFORM_BLOCKS "Active Uniform Blocks"],
        [ActiveUniformBlockMaxNameLength ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH "Active Uniform Block Max Name Length"],
        [GeometryVerticesOut GEOMETRY_VERTICES_OUT "Geometry Vertices Out"],
        [GeometryInputType GEOMETRY_INPUT_TYPE "Geometry Input Type"],
        [GeometryOutputType GEOMETRY_OUTPUT_TYPE "Geometry Output Type"],
        [GeometryShaderInvocations GEOMETRY_SHADER_INVOCATIONS "Geometry Shader Invocations"],
        [TessControlOutputVertices TESS_CONTROL_OUTPUT_VERTICES "Tessellation Control Output Vertices"],
        [TessGenMode TESS_GEN_MODE "Tessellation Gen Mode"],
        [TessGenSpacing TESS_GEN_SPACING "Tessellation Gen Spacing"],
        [TessGenVertexOrder TESS_GEN_VERTEX_ORDER "Tessellation Gen Vertex Order"],
        [TessGenPointMode TESS_GEN_POINT_MODE "Tessellation Gen Point Mode"],
        // [ComputeWorkGroupSize COMPUTE_WORK_GROUP_SIZE "Compute Work Group Size"],
        [ProgramSeparable PROGRAM_SEPARABLE "Program Separable"],
        [ProgramBinaryRetrievableHint PROGRAM_BINARY_RETRIEVABLE_HINT "Program Binary Retrievable Hint"],
        [ActiveAtomicCounterBuffers ACTIVE_ATOMIC_COUNTER_BUFFERS "Active Atomic Counter Buffers"]
    }
}

pub struct RawProgram {
    id: GLuint,
    attached_shaders: Vec<Rc<Shader>>
}

impl RawProgram {

    pub fn create(_gl: &GL20) -> Self {
        let id = unsafe {gl::CreateProgram()};
        if id == 0 {
            //this is a panic rather than a Result because it's not really _supposed_ to happen
            //and shouldn't be contigent on anything the user has done.
            //I mean...
            //  "If as error occurs, zero will be returned"
            //isn't exactly very specific so I'm assuming
            //it's not really going to happen unless something crazy goes on
            panic!("Unknown error when creating program object");
        } else {
            RawProgram { id: id, attached_shaders: Vec::new() }
        }
    }

    pub fn id(&self) -> GLuint {self.id}

    pub fn get_program_int(&self, p: ProgramParameter) -> GLint {
        let mut val:GLint = 0;
        unsafe { gl::GetProgramiv(self.id, p.into(), &mut val as *mut GLint); }
        val
    }

    pub fn delete_status(&self) -> bool {self.get_program_int(ProgramParameter::DeleteStatus) != 0}
    pub fn link_status(&self) -> bool {self.get_program_int(ProgramParameter::LinkStatus) != 0}
    pub fn validate_status(&self) -> bool {self.get_program_int(ProgramParameter::ValidateStatus) != 0}

    pub fn info_log_length(&self) -> GLuint {self.get_program_int(ProgramParameter::InfoLogLength) as GLuint}
    pub fn attached_shaders(&self) -> GLuint {self.get_program_int(ProgramParameter::AttachedShaders) as GLuint}

    pub fn leak(mut self) -> GLuint {
        let id = self.id();
        self.attached_shaders.clear();
        forget(self);
        id
    }

    pub fn is(id: GLuint) -> bool { unsafe {gl::IsProgram::is_loaded() && gl::IsProgram(id)!=0} }

    pub unsafe fn from_raw(id: GLuint) -> Option<Self> {
        if Self::is(id) { Some(RawProgram{id:id, attached_shaders:Vec::new()}) } else { None }
    }

    pub fn attach_shader(&mut self, shader: Rc<Shader>) -> bool {
        unsafe {
            for s in self.attached_shaders.iter() {
                if s.id() == shader.id() { return true; }
            }
            gl::AttachShader(self.id(), shader.id());
            self.attached_shaders.push(shader);
            return false;
        }
    }

    pub fn detach_shader(&mut self, shader: &Shader) -> bool {
        unsafe {
            for i in 0..self.attached_shaders.len() {
                let s = &self.attached_shaders[i];
                if s.id() == shader.id() {
                    gl::DetachShader(self.id(), shader.id());
                    self.attached_shaders.remove(i);
                    return true;
                }
            }
            return false;
        }
    }

    pub fn get_attached_shaders(&self) -> Box<[Rc<Shader>]> {
        self.attached_shaders.clone().into_boxed_slice()
    }

    pub fn get_attached_shaders_ids(&self) -> Box<[GLuint]> {
        let len = self.attached_shaders() as usize;
        if len > 0 {
            unsafe {
                let mut ids = Vec::with_capacity(len);
                let mut actual = 0;
                ids.set_len(len);
                gl::GetAttachedShaders(self.id(), len as GLsizei, &mut actual, &mut ids[0]);
                ids.set_len(actual as usize);
                ids.into_boxed_slice()
            }
        } else {
            Vec::new().into_boxed_slice()
        }

    }

    pub fn info_log(&mut self) -> String {
        unsafe {
            get_resource_string(self.id(), self.info_log_length(), gl::GetProgramInfoLog, "Malformatted program info log")
        }
    }

    pub fn link(&mut self) -> Result<(),GLError> {
        unsafe {
            gl::LinkProgram(self.id());
            if !self.link_status() {
                Err(GLError::ProgramLinking(self.id(), self.info_log()))
            } else {
                Ok(())
            }
        }
    }

    pub fn validate(&mut self) -> Result<(), GLError> {
        unsafe {
            gl::ValidateProgram(self.id());
            if !self.validate_status() {
                Err(GLError::ProgramValidation(self.id(), self.info_log()))
            } else {
                Ok(())
            }
        }
    }

    pub fn into_program(mut self) -> Result<Program, GLError> {
        if !self.link_status() || !self.validate_status() {
            self.link()?;
            self.validate()?;
        }
        Ok(Program{raw:self})
    }

    pub fn delete(self) -> bool {
        unsafe {
            gl::DeleteProgram(self.id());
            let status = self.delete_status();
            forget(self);
            status
        }
    }

}

impl Drop for RawProgram {
    fn drop(&mut self) { unsafe { gl::DeleteProgram(self.id()); } }
}
