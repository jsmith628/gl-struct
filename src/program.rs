
use gl::types::*;
use ::*;

use std::cell::Cell;
use std::mem::transmute;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ffi::CString;

glenum! {
    pub enum ShaderType {
        [Vertex VERTEX_SHADER "Vertex Shader"],
        [TessControl TESS_CONTROL_SHADER "Tesselation Control Shader"],
        [TessEval TESS_EVALUATION_SHADER "Tesselation Evaluation Shader"],
        [Geometry GEOMETRY_SHADER "Geometry Shader"],
        [Fragment FRAGMENT_SHADER "Fragment Shader"],
        [Compute COMPUTE_SHADER "Compute Shader"]
    }

    pub enum DrawMode {
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
}

impl DrawMode {
    pub fn valid_array_size(self, s: usize) -> bool {
        match self {
            DrawMode::Points => true,
            DrawMode::Lines => (s&1) == 0,
            DrawMode::LinesAdjacency => (s&3) == 0,
            DrawMode::LineStripAdjacency => s > 2,
            DrawMode::Triangles => (s%3) == 0,
            DrawMode::TrianglesAdjacency => (s%6) == 0,
            DrawMode::TriangleStripAdjacency => s > 4 && (s&1) == 0,
            _ => s > 1
        }
    }
}

pub struct Shader {
    id: GLuint,
    ty: ShaderType
}

impl Shader {

    pub fn create(_gl: &GLProvider, src: &str, ty: ShaderType) -> Result<Self, GLError> {
        unsafe {
            //create the shader
            let s = Shader {id: gl::CreateShader(ty.into()), ty: ty};

            let len = src.len() as GLint;
            let src_array = &src.as_bytes()[0];

            //do pointer magic to give GL the source code of the shader and compile
            gl::ShaderSource(s.id, 1, transmute(&src_array), &len as *const GLint);
            gl::CompileShader(s.id);

            //error check
            if s.get_shader_int(gl::COMPILE_STATUS) == gl::FALSE as GLint {
                Err(GLError::ShaderCompilation(s.id, ty, s.shader_info_log()))
            } else {
                Ok(s)
            }
        }

    }

    #[inline] pub fn shader_type(&self) -> ShaderType { self.ty }

    unsafe fn get_shader_int(&self, p: GLenum) -> GLint {
        let mut val:GLint = 0;
        gl::GetShaderiv(self.id, p, &mut val as *mut GLint);
        val
    }

    fn shader_info_log(&self) -> String {
        unsafe {
            let len = self.get_shader_int(gl::INFO_LOG_LENGTH);
            let mut actual: GLint = 0;
            if len > 0 {
                let mut log:Vec<u8> = vec![0; (len) as usize];
                gl::GetShaderInfoLog(self.id, len, &mut actual as *mut GLint, transmute(&mut log[0]));
                String::from_utf8(log).expect("Malformatted shader log")
            } else {
                "".to_owned()
            }
        }
    }

}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id); }
    }
}

pub struct ProgramID {
    id: GLuint
}

impl ProgramID {

    pub fn from_source(_gl: &GLProvider, shaders: Vec<(&str, ShaderType)>) -> Result<Self, GLError> {
        let mut list = Vec::with_capacity(shaders.len());
        for (src, ty) in shaders.iter() {
            list.push(Shader::create(_gl, src, *ty)?);
        }
        Self::from_shaders(_gl, list)
    }

    pub fn from_shaders(_gl: &GLProvider, shaders: Vec<Shader>) -> Result<Self, GLError> {

        unsafe {
            //create the program
            let id = gl::CreateProgram();
            let program = ProgramID{id: id};

            //attach the shaders
            for shader in shaders.iter() {
                gl::AttachShader(id, shader.id);
            }

            //we want to error check the link and validation, but since we also need to detatch the
            //shaders, we need to store the result temporarily
            let res = {
                //link and error check
                gl::LinkProgram(id);
                if program.get_program_int(gl::LINK_STATUS) == gl::FALSE as GLint {
                    Err(GLError::ProgramLinking(id, program.program_info_log()))
                } else {
                    //validate and error error check
                    gl::ValidateProgram(id);
                    if program.get_program_int(gl::VALIDATE_STATUS) == gl::FALSE as GLint {
                        Err(GLError::ProgramValidation(id, program.program_info_log()))
                    } else {
                        Ok(program)
                    }
                }
            };

            //detach the shaders
            for shader in shaders.iter() {
                gl::DetachShader(id, shader.id);
            }

            res
        }
    }

    #[inline] pub unsafe fn use_program(&self) { gl::UseProgram(self.id); }
    #[inline] pub unsafe fn unbind_program() { gl::UseProgram(0); }

    unsafe fn get_program_int(&self, p: GLenum) -> GLint {
        let mut val:GLint = 0;
        gl::GetProgramiv(self.id, p, &mut val as *mut GLint);
        val
    }

    fn program_info_log(&self) -> String {
        unsafe {
            let len = self.get_program_int(gl::INFO_LOG_LENGTH);
            let mut actual: GLint = 0;
            if len > 0 {
                let mut log:Vec<u8> = vec![0; (len) as usize];
                gl::GetProgramInfoLog(self.id, len, &mut actual as *mut GLint, transmute(&mut log[0]));
                String::from_utf8(log).expect("Malformatted shader log")
            } else {
                "".to_owned()
            }
        }
    }

}

impl Drop for ProgramID {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id); }
    }
}

pub unsafe trait Program: Sized {
    fn init(context: &GLProvider) -> Result<Self, GLError>;
}

pub unsafe trait ShaderProgram: Program {}
pub unsafe trait ComputeProgram: Program {}

pub struct Uniform<T: GLSLType> {
    value: Box<T>,
    location: Cell<(GLint, GLuint)>,
    loaded: Cell<bool>,
}

impl<T: GLSLType> Deref for Uniform<T> {
    type Target = T;
    fn deref(&self) -> &T {&*self.value}
}

impl<T: GLSLType> DerefMut for Uniform<T> {
    fn deref_mut(&mut self) -> &mut T { self.loaded.set(false); &mut *self.value}
}

impl<T: GLSLType> Uniform<T> {
    #[inline]
    pub fn set<U: Into<T>>(&mut self, data: U) {
        **self = data.into();
    }

    #[inline]
    pub fn get<U: From<T>>(&self) -> U {
        (**self).into()
    }

}

pub struct UniformLocation {
    id: GLint,
    pid: GLuint
}

impl UniformLocation {

    pub fn get(p: &ProgramID, name: &str) -> Result<UniformLocation, UniformLocation> {
        let id = unsafe {
            gl::GetUniformLocation(p.id, CString::new(name).unwrap().into_raw())
        };

        let loc = UniformLocation { id: id, pid: p.id };

        if id<0 {Err(loc)} else {Ok(loc)}

    }

    pub unsafe fn get_uniform<T:GLSLType>(&self) -> Uniform<T> {

        let value = T::get_uniform(self.pid, self.id);

        Uniform {
            value: Box::new(value),
            location: Cell::new((self.id, self.pid)),
            loaded: Cell::new(true)
        }
    }

    pub unsafe fn load<T:GLSLType>(&self, value: &Uniform<T>) {
        if !self.is_loaded(value) {
            T::load_uniform(self.id, &**value);
            value.loaded.set(true);
            value.location.set((self.id, self.pid));
        }
    }

    #[inline] fn is_loaded<T:GLSLType>(&self, value: &Uniform<T>) -> bool {
        let (id, pid) = value.location.get();
        value.loaded.get() && id == self.id && pid == self.pid
    }

}

#[derive(Clone, Copy)]
pub enum Attribute<'a, A:GLSLType> {
    Value(&'a dyn AttributeValue<A>),
    Array(AttribArray<'a, A>)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttributeLocation {
    id: GLint
}

impl AttributeLocation {

    pub fn get(p: &ProgramID, name: &str) -> Result<Self, Self> {
        let id = unsafe { gl::GetAttribLocation(p.id, CString::new(name).unwrap().into_raw()) };
        let loc = AttributeLocation {id: id};
        if id<0 {Err(loc)} else {Ok(loc)}
    }

    #[inline]
    pub unsafe fn load<'a, A:GLSLType>(&self, a: &Attribute<'a, A>) {
        if self.id < 0 {return};
        match a {
            Attribute::Value(val) => {
                debug_assert_eq!(val.format().size(), ::std::mem::size_of_val(val), "Invalid value size for given attribute format!");

                union Repr<'b, T> {
                    rust: &'b T,
                    void: &'b GLvoid
                }

                gl::DisableVertexAttribArray(self.id as GLuint);
                A::set_attribute(self.id as GLuint, val.format(), Repr{rust: val}.void);
            },
            Attribute::Array(arr) => {
                gl::EnableVertexAttribArray(self.id as GLuint);
                arr.bind();
                A::bind_attribute(self.id as GLuint, arr.format(), arr.stride(), arr.offset());
                AttribArray::<'a, A>::unbind();
            }
        }
    }

}

pub trait InterfaceBlock<L:BlockLayout, T:Layout<L>+?Sized> {
    fn buffer_target() -> IndexedBufferTarget;
    fn binding(&self) -> GLuint;

    #[inline]
    unsafe fn bind_buffer_range<A:BufferAccess>(&self, buffer: &Buffer<T, A>) {
        Self::buffer_target().bind_range(buffer, self.binding());
    }

    #[inline] unsafe fn unbind(&self) {Self::buffer_target().unbind(self.binding())}
}

pub struct UniformBlock<L:BlockLayout, T:Layout<L>+Sized> {
    id: GLuint,
    pid: GLuint,
    binding: GLuint,
    p: PhantomData<(Box<T>, L)>
}

impl<L:BlockLayout, T:Layout<L>+Sized> UniformBlock<L, T> {

    pub unsafe fn get(p: &ProgramID, name: &str) -> Self {
        let mut block = UniformBlock {
            id: gl::GetUniformBlockIndex(p.id, CString::new(name).unwrap().into_raw()),
            pid: p.id,
            binding: 0,
            p: PhantomData
        };

        if block.id!=gl::INVALID_INDEX {
            gl::GetActiveUniformBlockiv(block.pid, block.id, gl::UNIFORM_BLOCK_BINDING, transmute::<&mut GLuint, *mut GLint>(&mut block.binding));
        }

        block
    }

    pub unsafe fn set_binding(&mut self, binding: GLuint) {
        debug_assert!(binding < gl::MAX_UNIFORM_BUFFER_BINDINGS, "UBO Binding higher than maximum!");
        self.binding = binding;
        gl::UniformBlockBinding(self.pid, self.id, self.binding);
    }

}

impl<L:BlockLayout, T:Layout<L>+Sized> InterfaceBlock<L,T> for UniformBlock<L, T> {
    #[inline] fn buffer_target() -> IndexedBufferTarget {IndexedBufferTarget::UniformBuffer}
    #[inline] fn binding(&self) -> GLuint {self.binding}
}


pub struct ShaderStorageBlock<L:BlockLayout, T:Layout<L>+?Sized> {
    id: GLuint,
    pid: GLuint,
    binding: GLuint,
    p: PhantomData<(Box<T>, L)>
}

impl<L:BlockLayout, T:Layout<L>+?Sized> ShaderStorageBlock<L, T> {

    pub unsafe fn get(p: &ProgramID, name: &str) -> Self {
        let mut block = ShaderStorageBlock {
            id: gl::GetProgramResourceIndex(p.id, gl::SHADER_STORAGE_BLOCK, CString::new(name).unwrap().into_raw()),
            pid: p.id,
            binding: 0,
            p: PhantomData
        };

        if block.id!=gl::INVALID_INDEX {
            //aaaand this is officially the worst gl call ever made...
            let props = [gl::BUFFER_BINDING];
            gl::GetProgramResourceiv(
                block.pid, gl::SHADER_STORAGE_BLOCK, block.id,
                1, &props[0] as *const GLenum, 1, ::std::ptr::null_mut(),
                transmute::<&mut GLuint, *mut GLint>(&mut block.binding)
            );
        }

        block
    }

    pub unsafe fn set_binding(&mut self, binding: GLuint) {
        debug_assert!(binding < gl::MAX_VERTEX_ATTRIB_BINDINGS, "UBO Binding higher than maximum!");
        self.binding = binding;
        gl::ShaderStorageBlockBinding(self.pid, self.id, self.binding);
    }

}

impl<L:BlockLayout, T:Layout<L>+?Sized> InterfaceBlock<L,T> for ShaderStorageBlock<L, T> {
    #[inline] fn buffer_target() -> IndexedBufferTarget {IndexedBufferTarget::ShaderStorageBuffer}
    #[inline] fn binding(&self) -> GLuint {self.binding}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubroutineLocation {
    id: GLint,
    stage: ShaderType,
    pid: GLuint
}

impl SubroutineLocation {

}
