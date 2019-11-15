use super::*;

use glsl::GLSLType;
use std::marker::PhantomData;

pub use self::vertex::*;

mod vertex;

#[repr(C)]
pub struct VertexArray<'a,V:Vertex> {
    id: GLuint,
    data: PhantomData<(Slice<'a,GLuint,CopyOnly>, ArribArray<'a,V>)>
}
