
use super::*;

pub mod image;
pub mod pixel;

glenum! {
    pub enum IntType {
        [Byte BYTE "Byte"],
        [UByte UNSIGNED_BYTE "UByte"],
        [Short SHORT "Short"],
        [UShort UNSIGNED_SHORT "UShort"],
        [Int INT "Int"],
        [UInt UNSIGNED_INT "UInt"]
    }

    pub enum FloatType {
        [Half HALF_FLOAT "Half"],
        [Float FLOAT "Float"]
    }
}

impl IntType {
    #[inline]
    pub fn size_of(self) -> usize {
        match self {
            IntType::Byte | IntType::UByte => 1,
            IntType::Short |IntType::UShort => 2,
            IntType::Int | IntType::UInt => 4
        }
    }
}

impl FloatType {
    #[inline]
    pub fn size_of(self) -> usize {
        match self {
            FloatType::Half => 2,
            FloatType::Float => 4,
        }
    }
}
