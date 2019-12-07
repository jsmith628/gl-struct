use super::*;

use std::marker::PhantomData;
use std::mem::MaybeUninit;

use object::buffer::AttribArray;
use format::attribute::*;
use glsl::GLSLType;

pub use self::attrib::*;
pub use self::vertex::*;

mod attrib;
mod vertex;

pub struct VertexArray<'a,V:GLSLType> {
    id: GLuint,
    buffers: PhantomData<(&'a Buffer<GLuint, ReadOnly>, AttribArray<'a,V>)>
}

impl<'a,V:GLSLType> VertexArray<'a,V> {
    pub fn id(&self) -> GLuint { self.id }
}
