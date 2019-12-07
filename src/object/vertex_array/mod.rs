use super::*;

use std::marker::PhantomData;
use std::mem::MaybeUninit;

use crate::object::buffer::AttribArray;
use glsl::GLSLType;

pub use self::attrib::*;

mod attrib;

pub trait Vertex: GLSLType {}


pub struct VertexArray<'a,V:Vertex> {
    id: GLuint,
    buffers: PhantomData<(&'a Buffer<GLuint, ReadOnly>, AttribArray<'a,V>)>
}
