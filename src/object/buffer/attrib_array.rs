
use super::*;
use glsl::GLSLType;

pub struct AttribArray<'a,A:GLSLType> {
    buf: Slice<'a,[u8],ReadOnly>,
    stride: usize,
    format: A::AttributeFormat,
}
