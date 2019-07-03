
use super::*;
use glsl::GLSLType;

pub struct ArribArray<'a,A:GLSLType> {
    buf: BSlice<'a,[u8],CopyOnly>,
    stride: usize,
    format: A::AttributeFormat,
}
