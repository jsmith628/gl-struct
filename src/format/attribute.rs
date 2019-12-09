
use super::*;
use glsl::*;

use std::mem::*;
use std::convert::*;

glenum! {
    pub enum AttribType {
        [Byte BYTE "Byte"],
        [UByte UNSIGNED_BYTE "UByte"],
        [Short SHORT "Short"],
        [UShort UNSIGNED_SHORT "UShort"],
        [Int INT "Int"],
        [UInt UNSIGNED_INT "UInt"],

        [Half HALF_FLOAT "Half"],
        [Float FLOAT "Float"],
        [Double DOUBLE "Double"],
        [Fixed FIXED "Fixed"],

        #[allow(non_camel_case_types)]
        [Int_2_10_10_10_Rev INT_2_10_10_10_REV "Int-2-10-10-10-Rev"],

        #[allow(non_camel_case_types)]
        [UInt_2_10_10_10_Rev UNSIGNED_INT_2_10_10_10_REV "UInt-2-10-10-10-Rev"],

        #[allow(non_camel_case_types)]
        [UInt_10F_11F_11F_Rev UNSIGNED_INT_10F_11F_11F_REV "UInt-10F-11F-11F-Rev"]
    }
}

impl From<FloatType> for AttribType {
    fn from(f:FloatType) -> AttribType { (f as GLenum).try_into().unwrap() }
}

impl From<IntType> for AttribType {
    fn from(f:IntType) -> AttribType { (f as GLenum).try_into().unwrap() }
}

pub unsafe trait AttribFormat: Sized + Clone + Copy + PartialEq + Eq + Hash + Debug {
    fn attrib_count() -> usize;

    fn size(self, index: usize) -> GLenum;
    fn ty(self, index: usize) -> AttribType;
    fn normalized(self, index: usize) -> bool;

    fn packed(self, index: usize) -> bool;
    fn long(self, index: usize) -> bool;
    fn integer(self, index: usize) -> bool;
}

pub unsafe trait AttribData<A:AttribFormat>: Sized + Copy {
    fn format() -> A;
    fn offset(index: usize) -> usize;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum VecFormat {
    Float(FloatType, u32),
    Fixed(u32),
    Double(u32),

    Int(IntType, u32),
    Normalized(IntType, u32),

    //(normalized, bgra)
    #[allow(non_camel_case_types)]
    Int_2_10_10_10_Rev(bool, bool),

    //(normalized, bgra)
    #[allow(non_camel_case_types)]
    UInt_2_10_10_10_Rev(bool, bool),

    #[allow(non_camel_case_types)]
    UInt_10F_11F_11F_Rev
}

unsafe impl AttribFormat for VecFormat {
    fn attrib_count() -> usize { 1 }

    fn size(self, index: usize) -> GLenum {
        use self::VecFormat::*;
        match index {
            0 => match self {
                Float(_, n) | Fixed(n) | Double(n) | Int(_, n) | Normalized(_, n) => n.min(4),
                Int_2_10_10_10_Rev(_, true) | UInt_2_10_10_10_Rev(_, true) => gl::BGRA,
                Int_2_10_10_10_Rev(_, false) | UInt_2_10_10_10_Rev(_, false) => 4,
                UInt_10F_11F_11F_Rev => 3
            }
            _ => 0
        }
    }

    fn ty(self, _: usize) -> AttribType {
        use self::VecFormat::*;
        match self {
            Float(f, _) => f.into(),
            Fixed(_) => AttribType::Fixed,
            Double(_) => AttribType::Double,
            Int(f, _) => f.into(),
            Normalized(f, _) => f.into(),
            Int_2_10_10_10_Rev(_,_) => AttribType::Int_2_10_10_10_Rev,
            UInt_2_10_10_10_Rev(_,_) => AttribType::UInt_2_10_10_10_Rev,
            UInt_10F_11F_11F_Rev => AttribType::UInt_10F_11F_11F_Rev
        }
    }

    fn normalized(self, _: usize) -> bool {
        use self::VecFormat::*;
        match self {
            Normalized(_, _) => true,
            Int_2_10_10_10_Rev(b,_) | UInt_2_10_10_10_Rev(b,_) => b,
            _ => false
        }
    }

    fn packed(self, index: usize) -> bool {
        use self::VecFormat::*;
        match index {
            0 => match self {
                Int_2_10_10_10_Rev(_,_) | UInt_2_10_10_10_Rev(_,_) | UInt_10F_11F_11F_Rev => true,
                _ => false
            }
            _ => false
        }
    }

    fn long(self, _: usize) -> bool { false }
    fn integer(self, _: usize) -> bool { false }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct IVecFormat(pub IntType, pub u32);

unsafe impl AttribFormat for IVecFormat {
    fn attrib_count() -> usize {1}

    fn size(self, index: usize) -> GLenum {
        match index {
            0 => self.1.min(4),
            _ => 0
        }
    }

    fn ty(self, _: usize) -> AttribType { self.0.into() }
    fn normalized(self, _: usize) -> bool { false }

    fn packed(self, _: usize) -> bool { false }
    fn long(self, _: usize) -> bool { false }
    fn integer(self, _: usize) -> bool { true }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DVecFormat(pub u32);

unsafe impl AttribFormat for DVecFormat {
    fn attrib_count() -> usize {1}

    fn size(self, index: usize) -> GLenum {
        match index {
            0 => self.0.min(4),
            _ => 0
        }
    }

    fn ty(self, _: usize) -> AttribType { AttribType::Double }
    fn normalized(self, _: usize) -> bool { false }

    fn packed(self, _: usize) -> bool { false }
    fn long(self, _: usize) -> bool { true }
    fn integer(self, _: usize) -> bool { false }
}

unsafe impl AttribFormat for ! {
    fn attrib_count() -> usize {0}

    fn size(self, _: usize) -> GLenum { self }
    fn ty(self, _: usize) -> AttribType { self }
    fn normalized(self, _: usize) -> bool { self }

    fn packed(self, _: usize) -> bool { self }
    fn long(self, _: usize) -> bool { self }
    fn integer(self, _: usize) -> bool { self }
}

unsafe impl AttribFormat for void {
    fn attrib_count() -> usize {1}

    fn size(self, _: usize) -> GLenum { 0 }
    fn ty(self, _: usize) -> AttribType { AttribType::Int }
    fn normalized(self, _: usize) -> bool { false }

    fn packed(self, _: usize) -> bool { false }
    fn long(self, _: usize) -> bool { false }
    fn integer(self, _: usize) -> bool { false }
}

pub type Mat2Format = [VecFormat; 2];
pub type Mat3Format = [VecFormat; 3];
pub type Mat4Format = [VecFormat; 4];

pub type DMat2Format = [DVecFormat; 2];
pub type DMat3Format = [DVecFormat; 3];
pub type DMat4Format = [DVecFormat; 4];

macro_rules! array_format {
    ($($num:tt)*) => {
        $(
            unsafe impl<A:AttribFormat> AttribFormat for [A; $num] {
                fn attrib_count() -> usize { $num * A::attrib_count() }

                fn size(self, i: usize) -> GLenum { self[i / A::attrib_count()].size(i % A::attrib_count()) }
                fn ty(self, i: usize) -> AttribType { self[i / A::attrib_count()].ty(i % A::attrib_count()) }
                fn normalized(self, i: usize) -> bool { self[i / A::attrib_count()].normalized(i % A::attrib_count()) }

                fn packed(self, i: usize) -> bool { self[i / A::attrib_count()].packed(i % A::attrib_count()) }
                fn long(self, i: usize) -> bool { self[i / A::attrib_count()].long(i % A::attrib_count()) }
                fn integer(self, i: usize) -> bool { self[i / A::attrib_count()].integer(i % A::attrib_count()) }
            }

            unsafe impl<A:AttribFormat,T:AttribData<A>> AttribData<[A; $num]> for [T;$num] {
                fn format() -> [A; $num] { [T::format(); $num] }
                fn offset(i:usize) -> usize {
                    let (q, r) = (i / A::attrib_count(), i % A::attrib_count());
                    q * size_of::<T>() + T::offset(r)
                }
            }

        )*
    }
}

//lol no... it you need attribute arrays longer than 32 elements, you *probably* should try something else...
array_format!{
    01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
}

macro_rules! prim_attr {
    ($(@$kind:ident $prim:ident $value:ident)*) => {
        $(
            prim_attr!(@$kind $value 1 $prim);
            prim_attr!(@$kind $value 1 [$prim; 1]);
            prim_attr!(@$kind $value 2 [$prim; 2]);
            prim_attr!(@$kind $value 3 [$prim; 3]);
            prim_attr!(@$kind $value 4 [$prim; 4]);
        )*
    };

    (@int $value:ident $num:literal $ty:ty) => {
        unsafe impl AttribData<IVecFormat> for $ty {
            fn format() -> IVecFormat { IVecFormat(IntType::$value, $num) }
            fn offset(_:usize) -> usize { 0 }
        }

        unsafe impl AttribData<VecFormat> for $ty {
            fn format() -> VecFormat { VecFormat::Int(IntType::$value, $num) }
            fn offset(_:usize) -> usize { 0 }
        }
    };

    (@float $value:ident $num:literal $ty:ty) => {
        unsafe impl AttribData<VecFormat> for $ty {
            fn format() -> VecFormat { VecFormat::Float(FloatType::$value, $num) }
            fn offset(_:usize) -> usize { 0 }
        }
    };

    (@double $value:ident $num:literal $ty:ty) => {
        unsafe impl AttribData<VecFormat> for $ty {
            fn format() -> VecFormat { VecFormat::Double($num) }
            fn offset(_:usize) -> usize { 0 }
        }
        unsafe impl AttribData<DVecFormat> for $ty {
            fn format() -> DVecFormat { DVecFormat($num) }
            fn offset(_:usize) -> usize { 0 }
        }
    };
}

prim_attr! {
    @int bool UByte
    @int gl_bool UInt
    @int i8 Byte
    @int u8 UByte
    @int i16 Short
    @int u16 UShort
    @int i32 Int
    @int u32 UInt
    @float f32 Float
    @double f64 Double
}
