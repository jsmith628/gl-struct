
use gl::types::*;
use ::*;

use std::mem::transmute;
use std::ops::{Deref};
use std::convert::TryInto;

pub use self::shader::*;
pub use self::raw::*;

mod shader;
mod raw;

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

pub(self) unsafe fn get_resource_string(
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


// pub unsafe trait Program: Sized {
//     fn init(context: &GL1) -> Result<Self, GLError>;
// }
//
// pub unsafe trait ShaderProgram: Program {}
// pub unsafe trait ComputeProgram: Program {}

// pub struct Uniform<T: GLSLType> {
//     value: Box<T>,
//     location: Cell<(GLint, GLuint)>,
//     loaded: Cell<bool>,
// }
//
// impl<T: GLSLType> Deref for Uniform<T> {
//     type Target = T;
//     fn deref(&self) -> &T {&*self.value}
// }
//
// impl<T: GLSLType> DerefMut for Uniform<T> {
//     fn deref_mut(&mut self) -> &mut T { self.loaded.set(false); &mut *self.value}
// }
//
// impl<T: GLSLType> Uniform<T> {
//     #[inline]
//     pub fn set<U: Into<T>>(&mut self, data: U) {
//         **self = data.into();
//     }
//
//     #[inline]
//     pub fn get<U: From<T>>(&self) -> U {
//         (**self).into()
//     }
//
// }
//
// pub struct UniformLocation {
//     id: GLint,
//     pid: GLuint
// }
//
// impl UniformLocation {
//
//     pub fn get(p: &ProgramID, name: &str) -> Result<UniformLocation, UniformLocation> {
//         let id = unsafe {
//             gl::GetUniformLocation(p.id, CString::new(name).unwrap().into_raw())
//         };
//
//         let loc = UniformLocation { id: id, pid: p.id };
//
//         if id<0 {Err(loc)} else {Ok(loc)}
//
//     }
//
//     pub unsafe fn get_uniform<T:GLSLType>(&self) -> Uniform<T> {
//
//         let value = T::get_uniform(self.pid, self.id);
//
//         Uniform {
//             value: Box::new(value),
//             location: Cell::new((self.id, self.pid)),
//             loaded: Cell::new(true)
//         }
//     }
//
//     pub unsafe fn load<T:GLSLType>(&self, value: &Uniform<T>) {
//         if !self.is_loaded(value) {
//             T::load_uniform(self.id, &**value);
//             value.loaded.set(true);
//             value.location.set((self.id, self.pid));
//         }
//     }
//
//     #[inline] fn is_loaded<T:GLSLType>(&self, value: &Uniform<T>) -> bool {
//         let (id, pid) = value.location.get();
//         value.loaded.get() && id == self.id && pid == self.pid
//     }
//
// }
//
// #[derive(Clone, Copy)]
// pub enum Attribute<'a, A:GLSLType> {
//     Value(&'a dyn AttributeValue<A>),
//     Array(AttribArray<'a, A>)
// }
//
// #[derive(Clone, Copy, PartialEq, Eq, Hash)]
// pub struct AttributeLocation {
//     id: GLint
// }
//
// impl AttributeLocation {
//
//     pub fn get(p: &ProgramID, name: &str) -> Result<Self, Self> {
//         let id = unsafe { gl::GetAttribLocation(p.id, CString::new(name).unwrap().into_raw()) };
//         let loc = AttributeLocation {id: id};
//         if id<0 {Err(loc)} else {Ok(loc)}
//     }
//
//     #[inline]
//     pub unsafe fn load<'a, A:GLSLType>(&self, a: &Attribute<'a, A>) {
//         if self.id < 0 {return};
//         match a {
//             Attribute::Value(val) => {
//                 debug_assert_eq!(val.format().size(), ::std::mem::size_of_val(val), "Invalid value size for given attribute format!");
//
//                 union Repr<'b, T> {
//                     rust: &'b T,
//                     void: &'b GLvoid
//                 }
//
//                 gl::DisableVertexAttribArray(self.id as GLuint);
//                 A::set_attribute(self.id as GLuint, val.format(), Repr{rust: val}.void);
//             },
//             Attribute::Array(arr) => {
//                 gl::EnableVertexAttribArray(self.id as GLuint);
//                 arr.bind();
//                 A::bind_attribute(self.id as GLuint, arr.format(), arr.stride(), arr.offset());
//                 AttribArray::<'a, A>::unbind();
//             }
//         }
//     }
//
// }
//
// pub trait InterfaceBlock<L:BlockLayout, T:Layout<L>+?Sized> {
//     fn buffer_target() -> IndexedBufferTarget;
//     fn binding(&self) -> GLuint;
//
//     #[inline]
//     unsafe fn bind_buffer_range<A:BufferAccess>(&self, buffer: &Buffer<T, A>) {
//         Self::buffer_target().bind_range(buffer, self.binding());
//     }
//
//     #[inline] unsafe fn unbind(&self) {Self::buffer_target().unbind(self.binding())}
// }
//
// pub struct UniformBlock<L:BlockLayout, T:Layout<L>+Sized> {
//     id: GLuint,
//     pid: GLuint,
//     binding: GLuint,
//     p: PhantomData<(Box<T>, L)>
// }
//
// impl<L:BlockLayout, T:Layout<L>+Sized> UniformBlock<L, T> {
//
//     pub unsafe fn get(p: &ProgramID, name: &str) -> Self {
//         let mut block = UniformBlock {
//             id: gl::GetUniformBlockIndex(p.id, CString::new(name).unwrap().into_raw()),
//             pid: p.id,
//             binding: 0,
//             p: PhantomData
//         };
//
//         if block.id!=gl::INVALID_INDEX {
//             gl::GetActiveUniformBlockiv(block.pid, block.id, gl::UNIFORM_BLOCK_BINDING, transmute::<&mut GLuint, *mut GLint>(&mut block.binding));
//         }
//
//         block
//     }
//
//     pub unsafe fn set_binding(&mut self, binding: GLuint) {
//         debug_assert!(binding < gl::MAX_UNIFORM_BUFFER_BINDINGS, "UBO Binding higher than maximum!");
//         self.binding = binding;
//         gl::UniformBlockBinding(self.pid, self.id, self.binding);
//     }
//
// }
//
// impl<L:BlockLayout, T:Layout<L>+Sized> InterfaceBlock<L,T> for UniformBlock<L, T> {
//     #[inline] fn buffer_target() -> IndexedBufferTarget {IndexedBufferTarget::UniformBuffer}
//     #[inline] fn binding(&self) -> GLuint {self.binding}
// }
//
//
// pub struct ShaderStorageBlock<L:BlockLayout, T:Layout<L>+?Sized> {
//     id: GLuint,
//     pid: GLuint,
//     binding: GLuint,
//     p: PhantomData<(Box<T>, L)>
// }
//
// impl<L:BlockLayout, T:Layout<L>+?Sized> ShaderStorageBlock<L, T> {
//
//     pub unsafe fn get(p: &ProgramID, name: &str) -> Self {
//         let mut block = ShaderStorageBlock {
//             id: gl::GetProgramResourceIndex(p.id, gl::SHADER_STORAGE_BLOCK, CString::new(name).unwrap().into_raw()),
//             pid: p.id,
//             binding: 0,
//             p: PhantomData
//         };
//
//         if block.id!=gl::INVALID_INDEX {
//             //aaaand this is officially the worst gl call ever made...
//             let props = [gl::BUFFER_BINDING];
//             gl::GetProgramResourceiv(
//                 block.pid, gl::SHADER_STORAGE_BLOCK, block.id,
//                 1, &props[0] as *const GLenum, 1, ::std::ptr::null_mut(),
//                 transmute::<&mut GLuint, *mut GLint>(&mut block.binding)
//             );
//         }
//
//         block
//     }
//
//     pub unsafe fn set_binding(&mut self, binding: GLuint) {
//         debug_assert!(binding < gl::MAX_VERTEX_ATTRIB_BINDINGS, "UBO Binding higher than maximum!");
//         self.binding = binding;
//         gl::ShaderStorageBlockBinding(self.pid, self.id, self.binding);
//     }
//
// }
//
// impl<L:BlockLayout, T:Layout<L>+?Sized> InterfaceBlock<L,T> for ShaderStorageBlock<L, T> {
//     #[inline] fn buffer_target() -> IndexedBufferTarget {IndexedBufferTarget::ShaderStorageBuffer}
//     #[inline] fn binding(&self) -> GLuint {self.binding}
// }
//
// #[derive(Clone, Copy, PartialEq, Eq, Hash)]
// pub struct SubroutineLocation {
//     id: GLint,
//     stage: ShaderType,
//     pid: GLuint
// }
//
// impl SubroutineLocation {
//
// }
