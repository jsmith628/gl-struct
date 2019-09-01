
use gl::types::*;
use super::*;
use context::*;

use std::mem::transmute;
use std::ops::{Deref};
use std::convert::TryInto;
use std::any::Any;
use std::marker::PhantomData;
use std::ffi::*;

pub use self::shader::*;
pub use self::raw::*;
pub use self::uniform::*;
pub use self::uniform_block::*;

mod shader;
mod raw;
mod uniform;
mod uniform_block;

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

    pub enum GLSLTypeToken {
        [Float FLOAT "float"],
        [Vec2 FLOAT_VEC2 "vec2"],
        [Vec3 FLOAT_VEC3 "vec3"],
        [Vec4 FLOAT_VEC4 "vec4"],

        [Double DOUBLE "double"],
        [DVec2 DOUBLE_VEC2 "dvec2"],
        [DVec3 DOUBLE_VEC3 "dvec3"],
        [DVec4 DOUBLE_VEC4 "dvec4"],

        [Int INT "int"],
        [IVec2 INT_VEC2 "ivec2"],
        [IVec3 INT_VEC3 "ivec3"],
        [IVec4 INT_VEC4 "ivec4"],

        [UInt UNSIGNED_INT "uint"],
        [UVec2 UNSIGNED_INT_VEC2 "uvec2"],
        [UVec3 UNSIGNED_INT_VEC3 "uvec3"],
        [UVec4 UNSIGNED_INT_VEC4 "uvec4"],

        [Bool BOOL "bool"],
        [BVec2 BOOL_VEC2 "bvec2"],
        [BVec3 BOOL_VEC3 "bvec3"],
        [BVec4 BOOL_VEC4 "bvec4"],

        [Mat2 FLOAT_MAT2 "mat2"],
        [Mat3 FLOAT_MAT3 "mat3"],
        [Mat4 FLOAT_MAT4 "mat4"],
        [Mat2x3 FLOAT_MAT2x3 "mat2x3"],
        [Mat2x4 FLOAT_MAT2x4 "mat2x4"],
        [Mat3x2 FLOAT_MAT3x2 "mat3x2"],
        [Mat3x4 FLOAT_MAT3x4 "mat3x4"],
        [Mat4x2 FLOAT_MAT4x2 "mat4x2"],
        [Mat4x3 FLOAT_MAT4x3 "mat4x3"],

        [DMat2 DOUBLE_MAT2 "dmat2"],
        [DMat3 DOUBLE_MAT3 "dmat3"],
        [DMat4 DOUBLE_MAT4 "dmat4"],
        [DMat2x3 DOUBLE_MAT2x3 "dmat2x3"],
        [DMat2x4 DOUBLE_MAT2x4 "dmat2x4"],
        [DMat3x2 DOUBLE_MAT3x2 "dmat3x2"],
        [DMat3x4 DOUBLE_MAT3x4 "dmat3x4"],
        [DMat4x2 DOUBLE_MAT4x2 "dmat4x2"],
        [DMat4x3 DOUBLE_MAT4x3 "dmat4x3"],

        [Sampler1D SAMPLER_1D "sampler1D"],
        [Sampler2D SAMPLER_2D "sampler2D"],
        [Sampler3D SAMPLER_3D "sampler3D"],
        [SamplerCube SAMPLER_CUBE "samplerCube"],
        [Sampler1DArray SAMPLER_1D_ARRAY "sampler1DArray"],
        [Sampler2DArray SAMPLER_2D_ARRAY "sampler2DArray"],
        [SamplerCubeMapArray SAMPLER_CUBE_MAP_ARRAY "samplerCubeArray"],
        [Sampler2DMultisample SAMPLER_2D_MULTISAMPLE "sampler2DMS"],
        [Sampler2DMultisampleArray SAMPLER_2D_MULTISAMPLE_ARRAY "sampler2DMSArray"],
        [SamplerBuffer SAMPLER_BUFFER "samplerBuffer"],
        [Sampler2DRect SAMPLER_2D_RECT "sampler2DRect"],

        [Sampler1DShadow SAMPLER_1D_SHADOW "sampler1DShadow"],
        [Sampler2DShadow SAMPLER_2D_SHADOW "sampler2DShadow"],
        [Sampler1DArrayShadow SAMPLER_1D_ARRAY_SHADOW "sampler1DArrayShadow"],
        [Sampler2DArrayShadow SAMPLER_2D_ARRAY_SHADOW "sampler2DArrayShadow"],
        [SamplerCubeShadow SAMPLER_CUBE_SHADOW "samplerCubeShadow"],
        [SamplerCubeMapArrayShadow SAMPLER_CUBE_MAP_ARRAY_SHADOW "samplerCubeArrayShadow"],
        [Sampler2DRectShadow SAMPLER_2D_RECT_SHADOW "sampler2DRectShadow"],

        [IntSampler1D INT_SAMPLER_1D "isampler1D"],
        [IntSampler2D INT_SAMPLER_2D "isampler2D"],
        [IntSampler3D INT_SAMPLER_3D "isampler3D"],
        [IntSamplerCube INT_SAMPLER_CUBE "isamplerCube"],
        [IntSampler1DArray INT_SAMPLER_1D_ARRAY "isampler1DArray"],
        [IntSampler2DArray INT_SAMPLER_2D_ARRAY "isampler2DArray"],
        [IntSamplerCubeMapArray INT_SAMPLER_CUBE_MAP_ARRAY "isamplerCubeArray"],
        [IntSampler2DMultisample INT_SAMPLER_2D_MULTISAMPLE "isampler2DMS"],
        [IntSampler2DMultisampleArray INT_SAMPLER_2D_MULTISAMPLE_ARRAY "isampler2DMSArray"],
        [IntSamplerBuffer INT_SAMPLER_BUFFER "isamplerBuffer"],
        [IntSampler2DRect INT_SAMPLER_2D_RECT "isampler2DRect"],

        [UnsignedIntSampler1D UNSIGNED_INT_SAMPLER_1D "usampler1D"],
        [UnsignedIntSampler2D UNSIGNED_INT_SAMPLER_2D "usampler2D"],
        [UnsignedIntSampler3D UNSIGNED_INT_SAMPLER_3D "usampler3D"],
        [UnsignedIntSamplerCube UNSIGNED_INT_SAMPLER_CUBE "usamplerCube"],
        [UnsignedIntSampler1DArray UNSIGNED_INT_SAMPLER_1D_ARRAY "usampler1DArray"],
        [UnsignedIntSampler2DArray UNSIGNED_INT_SAMPLER_2D_ARRAY "usampler2DArray"],
        [UnsignedIntSamplerCubeMapArray UNSIGNED_INT_SAMPLER_CUBE_MAP_ARRAY "usamplerCubeArray"],
        [UnsignedIntSampler2DMultisample UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE "usampler2DMS"],
        [UnsignedIntSampler2DMultisampleArray UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE_ARRAY "usampler2DMSArray"],
        [UnsignedIntSamplerBuffer UNSIGNED_INT_SAMPLER_BUFFER "usamplerBuffer"],
        [UnsignedIntSampler2DRect UNSIGNED_INT_SAMPLER_2D_RECT "usampler2DRect"],

        [Image1D IMAGE_1D "image1D"],
        [Image2D IMAGE_2D "image2D"],
        [Image3D IMAGE_3D "image3D"],
        [Image2DRect IMAGE_2D_RECT "image2DRect"],
        [ImageCube IMAGE_CUBE "imageCube"],
        [ImageBuffer IMAGE_BUFFER "imageBuffer"],
        [Image1DArray IMAGE_1D_ARRAY "image1DArray"],
        [Image2DArray IMAGE_2D_ARRAY "image2DArray"],
        [ImageCubeMapArray IMAGE_CUBE_MAP_ARRAY "imageCubeArray"],
        [Image2DMutlisample IMAGE_2D_MULTISAMPLE "image2DMS"],
        [Image2DMultisampleArray IMAGE_2D_MULTISAMPLE_ARRAY "image2DMSArray"],

        [IntImage1D INT_IMAGE_1D "iimage1D"],
        [IntImage2D INT_IMAGE_2D "iimage2D"],
        [IntImage3D INT_IMAGE_3D "iimage3D"],
        [IntImage2DRect INT_IMAGE_2D_RECT "iimage2DRect"],
        [IntImageCube INT_IMAGE_CUBE "iimageCube"],
        [IntImageBuffer INT_IMAGE_BUFFER "iimageBuffer"],
        [IntImage1DArray INT_IMAGE_1D_ARRAY "iimage1DArray"],
        [IntImage2DArray INT_IMAGE_2D_ARRAY "iimage2DArray"],
        [IntImageCubeMapArray INT_IMAGE_CUBE_MAP_ARRAY "iimageCubeArray"],
        [IntImage2DMutlisample INT_IMAGE_2D_MULTISAMPLE "iimage2DMS"],
        [IntImage2DMultisampleArray INT_IMAGE_2D_MULTISAMPLE_ARRAY "iimage2DMSArray"],

        [UnsignedIntImage1D UNSIGNED_INT_IMAGE_1D "uimage1D"],
        [UnsignedIntImage2D UNSIGNED_INT_IMAGE_2D "uimage2D"],
        [UnsignedIntImage3D UNSIGNED_INT_IMAGE_3D "uimage3D"],
        [UnsignedIntImage2DRect UNSIGNED_INT_IMAGE_2D_RECT "uimage2DRect"],
        [UnsignedIntImageCube UNSIGNED_INT_IMAGE_CUBE "uimageCube"],
        [UnsignedIntImageBuffer UNSIGNED_INT_IMAGE_BUFFER "uimageBuffer"],
        [UnsignedIntImage1DArray UNSIGNED_INT_IMAGE_1D_ARRAY "uimage1DArray"],
        [UnsignedIntImage2DArray UNSIGNED_INT_IMAGE_2D_ARRAY "uimage2DArray"],
        [UnsignedIntImageCubeMapArray UNSIGNED_INT_IMAGE_CUBE_MAP_ARRAY "uimageCubeArray"],
        [UnsignedIntImage2DMutlisample UNSIGNED_INT_IMAGE_2D_MULTISAMPLE "uimage2DMS"],
        [UnsignedIntImage2DMultisampleArray UNSIGNED_INT_IMAGE_2D_MULTISAMPLE_ARRAY "uimage2DMSArray"],

        [UnsignedIntAtomicCounter UNSIGNED_INT_ATOMIC_COUNTER "atomic_uint"]
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

    pub fn get_uniform_location(&self, name: &CStr) -> Option<Uniform<dyn Any>> {
        unsafe {
            let index = gl::GetUniformLocation(self.id(), name.as_ptr());
            if index >= 0 {
                Some(Uniform {program: self.id(), index: index as GLuint, ty: PhantomData} )
            } else {
                None
            }
        }
    }

}

impl Program {
    pub fn active_attributes(&self) -> GLuint { unsafe {self.get_program_int(gl::ACTIVE_ATTRIBUTES) as GLuint} }
    pub fn active_attribute_max_length(&self) -> GLuint {
        unsafe {self.get_program_int(gl::ACTIVE_ATTRIBUTE_MAX_LENGTH) as GLuint}
    }

    pub fn active_uniforms(&self) -> GLuint { unsafe {self.get_program_int(gl::ACTIVE_UNIFORMS) as GLuint} }
    pub fn active_uniform_max_length(&self) -> GLuint {
        unsafe {self.get_program_int(gl::ACTIVE_UNIFORM_MAX_LENGTH) as GLuint}
    }

    pub fn transform_feedback_varyings(&self) -> GLuint {
        unsafe {self.get_program_int(gl::TRANSFORM_FEEDBACK_VARYINGS) as GLuint}
    }
    pub fn transform_feedback_varying_max_length(&self) -> GLuint {
        unsafe {self.get_program_int(gl::TRANSFORM_FEEDBACK_VARYING_MAX_LENGTH) as GLuint}
    }
    pub fn transform_feedback_buffer_mode(&self) -> TransformFeedbackBufferMode {
        unsafe {self.get_program_glenum(gl::TRANSFORM_FEEDBACK_BUFFER_MODE)}
    }

    pub fn active_uniform_blocks(&self) -> GLuint { unsafe {self.get_program_int(gl::ACTIVE_UNIFORM_BLOCKS) as GLuint} }
    pub fn active_uniform_block_max_name_length(&self) -> GLuint {
        unsafe { self.get_program_int(gl::ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH) as GLuint }
    }

    pub fn geometry_vertices_out(&self) -> GLuint { unsafe {self.get_program_int(gl::GEOMETRY_VERTICES_OUT) as GLuint} }
    pub fn geometry_shader_invocations(&self) -> GLuint { unsafe {self.get_program_int(gl::GEOMETRY_SHADER_INVOCATIONS) as GLuint} }
    pub fn geometry_input_type(&self) -> DrawPrimitive { unsafe {self.get_program_glenum(gl::GEOMETRY_INPUT_TYPE)} }
    pub fn geometry_output_type(&self) -> DrawPrimitive { unsafe {self.get_program_glenum(gl::GEOMETRY_OUTPUT_TYPE)} }

    pub fn tess_control_output_vertices(&self) -> GLuint {
        unsafe {self.get_program_int(gl::TESS_CONTROL_OUTPUT_VERTICES) as GLuint}
    }
    pub fn tess_gen_point_mode(&self) -> bool { unsafe {self.get_program_int(gl::TESS_GEN_POINT_MODE)!=0} }
    pub fn tess_gen_mode(&self) -> TessGenMode { unsafe {self.get_program_glenum(gl::TESS_GEN_MODE)} }
    pub fn tess_gen_spacing(&self) -> TessGenSpacing { unsafe {self.get_program_glenum(gl::TESS_GEN_SPACING)} }
    pub fn tess_gen_vertex_order(&self) -> VertexWinding { unsafe {self.get_program_glenum(gl::TESS_GEN_VERTEX_ORDER)} }

    pub fn program_separable(&self) -> bool { unsafe {self.get_program_int(gl::PROGRAM_SEPARABLE)!= 0} }
    pub fn program_binary_retrievable_hint(&self) -> bool {
        unsafe { self.get_program_int(gl::PROGRAM_BINARY_RETRIEVABLE_HINT) != 0 }
    }

    pub fn active_atomic_counter_buffers(&self) -> GLuint {
        unsafe { self.get_program_int(gl::ACTIVE_ATOMIC_COUNTER_BUFFERS) as GLuint }
    }

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
