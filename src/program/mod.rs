
use gl::types::*;
use ::*;

use std::mem::transmute;
use std::ops::{Deref};
use std::convert::TryInto;

pub use self::shader::*;
pub use self::raw::*;
// pub use self::uniform::*;

mod shader;
mod raw;
// mod uniform;

glenum! {

    pub enum DrawPrimitive {
        [Points POINTS "Points"],
        [Lines LINES "Lines"],
        [LineStrip LINE_STRIP "Line Strip"],
        [LineLoop LINE_LOOP "Line Loop"],
        [LinesAdjacency LINES_ADJACENCY "Lines Adjacency"],
        [LineStripAdjacency LINE_STRIP_ADJACENCY "Line Strip Adjacency"],
        [Triangles TRIANGLES "Triangles"],
        [TriangleStrip TRIANGLE_STRIP "Triangle Strip"],
        [TriangleFan TRIANGLE_FAN "Triangle Fan"],
        [TrianglesAdjacency TRIANGLES_ADJACENCY "Triangles Adjacency"],
        [TriangleStripAdjacency TRIANGLE_STRIP_ADJACENCY "Triangle Strip Adjacency"]
    }

    pub enum TransformFeedbackBufferMode {
        [SeparateAttribs SEPARATE_ATTRIBS "Separate Attributes"],
        [InterleavedAttribs INTERLEAVED_ATTRIBS "Interleaved Attributes"]
    }

    pub enum TessGenMode {
        [Quads QUADS "Quads"],
        [Triangles TRIANGLES "Triangles"],
        [Isolines ISOLINES "Isolines"]
    }

    pub enum TessGenSpacing {
        [Equal EQUAL "Equal"],
        [FractionalEven FRACTIONAL_EVEN "Fractional-Even"],
        [FractionalOdd FRACTIONAL_ODD "Fractional-Odd"]
    }
}

impl DrawPrimitive {
    pub fn valid_array_size(self, s: usize) -> bool {
        match self {
            DrawPrimitive::Points => true,
            DrawPrimitive::Lines => (s&1) == 0,
            DrawPrimitive::LinesAdjacency => (s&3) == 0,
            DrawPrimitive::LineStripAdjacency => s > 2,
            DrawPrimitive::Triangles => (s%3) == 0,
            DrawPrimitive::TrianglesAdjacency => (s%6) == 0,
            DrawPrimitive::TriangleStripAdjacency => s > 4 && (s&1) == 0,
            _ => s > 1
        }
    }
}

pub(self) unsafe fn get_program_string(
    id:GLuint, len:GLuint, f: unsafe fn(GLuint,GLsizei,*mut GLsizei,*mut GLchar), msg:&'static str
) -> String {
    let len = len as usize;
    let mut actual: GLint = 0;
    if len > 0 {
        let mut log:Vec<u8> = Vec::with_capacity(len);
        log.set_len(len);
        f(id, len as GLsizei, &mut actual as *mut GLint, transmute(&mut log[0]));
        log.set_len(len.min(actual as usize));
        String::from_utf8(log).expect(msg)
    } else {
        "".to_owned()
    }
}

pub(self) unsafe fn get_resource_string(
    prog:GLuint, id:GLuint, len:GLuint, f: unsafe fn(GLuint,GLuint,GLsizei,*mut GLsizei,*mut GLchar), msg:&'static str
) -> String {
    let len = len as usize;
    let mut actual: GLint = 0;
    if len > 0 {
        let mut log:Vec<u8> = Vec::with_capacity(len);
        log.set_len(len);
        f(prog, id, len as GLsizei, &mut actual as *mut GLint, transmute(&mut log[0]));
        log.set_len(len.min(actual as usize));
        String::from_utf8(log).expect(msg)
    } else {
        "".to_owned()
    }
}

pub struct Program {
    raw: RawProgram
}

impl Deref for Program {
    type Target = RawProgram;
    fn deref(&self) -> &RawProgram {&self.raw}
}

impl Program {

    fn from_raw(raw: RawProgram) -> Self {
        Program {raw: raw}
    }

    pub fn from_source(gl:&GL20, src: &[(ShaderType, &[&str])]) -> Result<Self, GLError> {

        //compile the shader source code
        let mut shaders = Vec::with_capacity(src.len());
        for (ty, src) in src { shaders.push(Shader::from_source(gl, *ty, src)?); }

        //convert to references
        Self::from_shaders(gl, &shaders.iter().collect::<Vec<_>>())
    }

    pub fn from_shaders(gl:&GL20, shaders: &[&Shader]) -> Result<Self, GLError> {

        let mut raw = RawProgram::create(gl);

        for s in shaders.iter() { unsafe { gl::AttachShader(raw.id(), s.id()); } }

        let result = {
            raw.link()?;
            raw.validate()?;
            Ok(())
        };

        result.map(
            |_| {
                for s in shaders.iter() { unsafe { gl::DetachShader(raw.id(), s.id()); } }
                Self::from_raw(raw)
            }
        )
    }

}

use ProgramParameter::*;

impl Program {
    pub fn active_attributes(&self) -> GLuint { self.get_program_int(ActiveAttributes) as GLuint }
    pub fn active_attribute_max_length(&self) -> GLuint { self.get_program_int(ActiveAttributeMaxLength) as GLuint }

    pub fn active_uniforms(&self) -> GLuint { self.get_program_int(ActiveUniforms) as GLuint }
    pub fn active_uniform_max_length(&self) -> GLuint { self.get_program_int(ActiveUniformMaxLength) as GLuint }

    pub fn transform_feedback_varyings(&self) -> GLuint { self.get_program_int(TransformFeedbackVaryings) as GLuint }
    pub fn transform_feedback_varying_max_length(&self) -> GLuint { self.get_program_int(TransformFeedbackVaryingMaxLength) as GLuint }
    pub fn transform_feedback_buffer_mode(&self) -> TransformFeedbackBufferMode {
        (self.get_program_int(TransformFeedbackBufferMode) as GLenum).try_into().unwrap()
    }

    pub fn active_uniform_blocks(&self) -> GLuint { self.get_program_int(ActiveUniformBlocks) as GLuint }
    pub fn active_uniform_block_max_name_length(&self) -> GLuint { self.get_program_int(ActiveUniformBlockMaxNameLength) as GLuint }

    pub fn geometry_vertices_out(&self) -> GLuint { self.get_program_int(GeometryVerticesOut) as GLuint }
    pub fn geometry_shader_invocations(&self) -> GLuint { self.get_program_int(GeometryShaderInvocations) as GLuint }
    pub fn geometry_input_type(&self) -> DrawPrimitive {
        (self.get_program_int(GeometryInputType) as GLenum).try_into().unwrap()
    }
    pub fn geometry_output_type(&self) -> DrawPrimitive {
        (self.get_program_int(GeometryOutputType) as GLenum).try_into().unwrap()
    }

    pub fn tess_control_output_vertices(&self) -> GLuint { self.get_program_int(TessControlOutputVertices) as GLuint }
    pub fn tess_gen_point_mode(&self) -> bool { self.get_program_int(TessGenPointMode) != 0 }
    pub fn tess_gen_mode(&self) -> TessGenMode {
        (self.get_program_int(TessGenMode) as GLenum).try_into().unwrap()
    }
    pub fn tess_gen_spacing(&self) -> TessGenSpacing {
        (self.get_program_int(TessGenSpacing) as GLenum).try_into().unwrap()
    }
    pub fn tess_gen_vertex_order(&self) -> VertexWinding {
        (self.get_program_int(TessGenVertexOrder) as GLenum).try_into().unwrap()
    }

    pub fn program_separable(&self) -> bool { self.get_program_int(ProgramSeparable) != 0 }
    pub fn program_binary_retrievable_hint(&self) -> bool { self.get_program_int(ProgramBinaryRetrievableHint) != 0 }

    pub fn active_atomic_counter_buffers(&self) -> GLuint { self.get_program_int(ActiveAtomicCounterBuffers) as GLuint }

    pub fn compute_work_group_size(&self) -> [GLuint; 3] {
        unsafe {
            let mut dest = [0;3];
            gl::GetProgramiv(self.id(), gl::COMPUTE_WORK_GROUP_SIZE, &mut dest[0] as *mut GLuint as *mut GLint);
            dest
        }
    }

}

impl !Send for Program {}
impl !Sync for Program {}
