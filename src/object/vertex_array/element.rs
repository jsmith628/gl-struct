use super::*;
use std::convert::*;

glenum! {
    pub enum ElementType {
        [UByte UNSIGNED_BYTE "UByte"],
        [UShort UNSIGNED_SHORT "UShort"],
        [UInt UNSIGNED_INT "UInt"]
    }
}

impl From<ElementType> for IntType {
    fn from(e: ElementType) -> IntType { (e as GLenum).try_into().unwrap() }
}

impl TryFrom<IntType> for ElementType {
    type Error = GLError;
    fn try_from(e: IntType) -> Result<ElementType,GLError> { (e as GLenum).try_into() }
}

pub unsafe trait Element: Copy {
    fn ty() -> ElementType;
}

unsafe impl Element for GLubyte { #[inline] fn ty() -> ElementType {ElementType::UByte} }
unsafe impl Element for GLushort { #[inline] fn ty() -> ElementType {ElementType::UShort} }
unsafe impl Element for GLuint { #[inline] fn ty() -> ElementType {ElementType::UInt} }
