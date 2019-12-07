
use super::*;
use glsl::GLSLType;

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub struct AttribArray<'a,A:GLSLType> {
    buf: Slice<'a,[u8],ReadOnly>,
    stride: usize,
    format: A::AttributeFormat,
}
