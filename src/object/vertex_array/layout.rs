
use super::*;
use glsl::*;

use std::mem::*;
use std::convert::*;

glenum! {

    ///All OpenGL data types that can encode vertex element index information
    #[non_exhaustive]
    pub enum ElementType {
        [UByte UNSIGNED_BYTE "UByte"],
        [UShort UNSIGNED_SHORT "UShort"],
        [UInt UNSIGNED_INT "UInt"]
    }

    //TODO: add GL version checks on the packed attribute types

    ///All OpenGL data types that can encode vertex attribute data
    #[non_exhaustive]
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

impl From<ElementType> for IntType {
    fn from(e: ElementType) -> IntType { (e as GLenum).try_into().unwrap() }
}

impl TryFrom<IntType> for ElementType {
    type Error = GLError;
    fn try_from(e: IntType) -> Result<ElementType,GLError> { (e as GLenum).try_into() }
}

impl From<FloatType> for AttribType {
    fn from(f:FloatType) -> AttribType { (f as GLenum).try_into().unwrap() }
}

impl From<IntType> for AttribType {
    fn from(f:IntType) -> AttribType { (f as GLenum).try_into().unwrap() }
}

pub unsafe trait Element: Copy {
    fn ty() -> ElementType;
}

unsafe impl Element for GLubyte { #[inline] fn ty() -> ElementType {ElementType::UByte} }
unsafe impl Element for GLushort { #[inline] fn ty() -> ElementType {ElementType::UShort} }
unsafe impl Element for GLuint { #[inline] fn ty() -> ElementType {ElementType::UInt} }

pub unsafe trait AttribFormat: Sized + Clone + Copy + PartialEq + Eq + Hash + Debug {
    fn attrib_count() -> usize;
    fn offset(self, index: usize) -> usize;

    fn size(self, index: usize) -> GLenum;
    fn ty(self, index: usize) -> AttribType;
    fn normalized(self, index: usize) -> bool;

    fn packed(self, index: usize) -> bool;
    fn long(self, index: usize) -> bool;
    fn integer(self, index: usize) -> bool;

    fn from_layouts(layouts: &[GenAttribFormat]) -> Result<Self,GLError>;

}

pub unsafe trait AttribData: Sized + Copy {
    type Format: AttribFormat;
    type GLSL: GLSLType<AttribFormat=Self::Format>;
    fn format() -> Self::Format;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GenAttribFormat {
    pub(crate) offset: usize,
    pub(crate) size: GLenum,
    pub(crate) ty: AttribType,
    pub(crate) normalized: bool
}

impl GenAttribFormat {
    pub fn offset(self) -> usize { self.offset }
    pub fn size(self) -> GLenum { self.size }
    pub fn ty(self) -> AttribType { self.ty }
    pub fn normalized(self) -> bool { self.normalized }
}

#[non_exhaustive]
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

impl From<IVecFormat> for VecFormat {
    fn from(ivec: IVecFormat) -> VecFormat { VecFormat::Int(ivec.0, ivec.1) }
}

impl From<DVecFormat> for VecFormat {
    fn from(dvec: DVecFormat) -> VecFormat { VecFormat::Double(dvec.0) }
}

unsafe impl AttribFormat for VecFormat {

    fn from_layouts(layouts: &[GenAttribFormat]) -> Result<Self,GLError> {
        use self::AttribType::*;

        let layout = layouts[0];
        Ok(
            match layout.ty {

                ty @ Half | ty @ Float => Self::Float((ty as GLenum).try_into()?, layout.size),
                Fixed => Self::Fixed(layout.size),
                Double => Self::Double(layout.size),

                ty @ Byte | ty @ UByte | ty @ Short | ty @ UShort | ty @ Int | ty @ UInt => {
                    let ty = (ty as GLenum).try_into()?;
                    match layout.normalized {
                        true => Self::Normalized(ty, layout.size),
                        false => Self::Int(ty, layout.size),
                    }
                },


                Int_2_10_10_10_Rev => Self::Int_2_10_10_10_Rev(layout.normalized, layout.size==gl::BGRA),
                UInt_2_10_10_10_Rev => Self::UInt_2_10_10_10_Rev(layout.normalized, layout.size==gl::BGRA),
                UInt_10F_11F_11F_Rev => Self::UInt_10F_11F_11F_Rev,

            }
        )
    }


    fn attrib_count() -> usize { 1 }
    fn offset(self, _: usize) -> usize { 0 }

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

impl IVecFormat {
    pub fn normalized(self) -> VecFormat {
        VecFormat::Normalized(self.0, self.1)
    }
}

unsafe impl AttribFormat for IVecFormat {

    fn from_layouts(layouts: &[GenAttribFormat]) -> Result<Self,GLError> {
        Ok(IVecFormat((layouts[0].ty as GLenum).try_into()?, layouts[0].size))
    }

    fn attrib_count() -> usize {1}
    fn offset(self, _: usize) -> usize { 0 }

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

    fn from_layouts(layouts: &[GenAttribFormat]) -> Result<Self,GLError> {
        Ok(DVecFormat(layouts[0].size))
    }

    fn attrib_count() -> usize {1}
    fn offset(self, _: usize) -> usize { 0 }

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

    fn from_layouts(_: &[GenAttribFormat]) -> Result<Self,GLError> {
        Err(GLError::InvalidValue("Uninstantiable attribute format".to_string()))
    }

    fn attrib_count() -> usize {0}
    fn offset(self, _: usize) -> usize {self}

    fn size(self, _: usize) -> GLenum { self }
    fn ty(self, _: usize) -> AttribType { self }
    fn normalized(self, _: usize) -> bool { self }

    fn packed(self, _: usize) -> bool { self }
    fn long(self, _: usize) -> bool { self }
    fn integer(self, _: usize) -> bool { self }
}

unsafe impl AttribFormat for () {

    fn from_layouts(_: &[GenAttribFormat]) -> Result<(),GLError> { Ok(()) }

    fn attrib_count() -> usize {1}
    fn offset(self, _: usize) -> usize { 0 }

    fn size(self, _: usize) -> GLenum { 0 }
    fn ty(self, _: usize) -> AttribType { AttribType::Int }
    fn normalized(self, _: usize) -> bool { false }

    fn packed(self, _: usize) -> bool { false }
    fn long(self, _: usize) -> bool { false }
    fn integer(self, _: usize) -> bool { false }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct OffsetFormat<A:AttribFormat> {
    pub offset: usize,
    pub fmt: A
}

unsafe impl<A:AttribFormat> AttribFormat for OffsetFormat<A> {

    fn from_layouts(layouts: &[GenAttribFormat]) -> Result<Self,GLError> {
        let fmt = A::from_layouts(layouts)?;
        Ok(OffsetFormat { offset: layouts[0].offset - fmt.offset(0), fmt: fmt } )
    }

    fn attrib_count() -> usize { A::attrib_count() }
    fn offset(self, i: usize) -> usize { self.offset + self.fmt.offset(i) }

    fn size(self, i: usize) -> GLenum { self.fmt.size(i) }
    fn ty(self, i: usize) -> AttribType { self.fmt.ty(i) }
    fn normalized(self, i: usize) -> bool { self.fmt.normalized(i) }

    fn packed(self, i: usize) -> bool { self.fmt.packed(i) }
    fn long(self, i: usize) -> bool { self.fmt.long(i) }
    fn integer(self, i: usize) -> bool { self.fmt.integer(i) }
}

pub type Mat2Format = [OffsetFormat<VecFormat>; 2];
pub type Mat3Format = [OffsetFormat<VecFormat>; 3];
pub type Mat4Format = [OffsetFormat<VecFormat>; 4];

pub type DMat2Format = [OffsetFormat<DVecFormat>; 2];
pub type DMat3Format = [OffsetFormat<DVecFormat>; 3];
pub type DMat4Format = [OffsetFormat<DVecFormat>; 4];

macro_rules! array_format {

    (fn $fn:ident<$A:ident>() -> $ret:ty) => {
        fn $fn(self, i: usize) -> $ret { self[i / A::attrib_count()].$fn(i % A::attrib_count()) }
    };

    ($($num:literal)*) => {
        $(
            unsafe impl<A:AttribFormat> AttribFormat for [OffsetFormat<A>; $num] {
                fn attrib_count() -> usize { $num * A::attrib_count() }
                array_format!(fn offset<A>() -> usize);

                array_format!(fn size<A>() -> GLenum);
                array_format!(fn ty<A>() -> AttribType);
                array_format!(fn normalized<A>() -> bool);

                array_format!(fn packed<A>() -> bool);
                array_format!(fn long<A>() -> bool);
                array_format!(fn integer<A>() -> bool);


                fn from_layouts(layouts: &[GenAttribFormat]) -> Result<Self,GLError> {
                    let mut fmt = MaybeUninit::<Self>::uninit();

                    for i in 0..$num {
                        unsafe {
                            fmt.get_mut()[i] = OffsetFormat::from_layouts(
                                &layouts[(i*A::attrib_count())..]
                            )?;
                        }
                    }

                    Ok(unsafe {fmt.assume_init()})
                }

            }

            unsafe impl<T:AttribData> AttribData for [T;$num] {

                type Format = [OffsetFormat<T::Format>; $num];
                type GLSL = [T::GLSL; $num];

                fn format() -> Self::Format {
                    let mut fmt = [OffsetFormat { offset: 0, fmt: T::format() }; $num];
                    for i in 0..$num { fmt[i].offset = i*size_of::<T>(); }
                    fmt
                }

            }

        )*
    }
}

//lol no... it you need attribute arrays longer than 32 elements, you *probably* should try something else...
array_format!{
    01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
}

macro_rules! tuple_format {

    //constructs a function that takes in an index, selects the tuple element for that index,
    //and returns the desired property for that index
    (fn $fn:ident<$($A:ident:$a:ident)*>() -> $ret:ty {$default:expr} ) => {

        #[allow(unused_assignments)]
        fn $fn(self, mut i: usize) -> $ret {
            let ($($a,)*) = self;
            $(
                if i < $A::attrib_count() {
                    return $a.$fn(i);
                } else {
                    i -= $A::attrib_count()
                }
            )*
            return $default;
        }
    };

    ($($A:ident:$a:ident)*) => {

        unsafe impl<$($A:AttribFormat),*> AttribFormat for ($($A,)*) {
            fn attrib_count() -> usize { 0 $( + $A::attrib_count())* }

            tuple_format!(fn offset<$($A:$a)*>() -> usize {0});

            tuple_format!(fn size<$($A:$a)*>() -> GLenum {0});
            tuple_format!(fn ty<$($A:$a)*>() -> AttribType {AttribType::Int});
            tuple_format!(fn normalized<$($A:$a)*>() -> bool {false});

            tuple_format!(fn packed<$($A:$a)*>() -> bool {false});
            tuple_format!(fn long<$($A:$a)*>() -> bool {false});
            tuple_format!(fn integer<$($A:$a)*>() -> bool {false});

            #[allow(unused_assignments)]
            fn from_layouts(mut layouts: &[GenAttribFormat]) -> Result<Self,GLError> {
                Ok((
                    $(
                        {
                            let fmt = $A::from_layouts(layouts)?;
                            layouts = &layouts[$A::attrib_count()..];
                            fmt
                        },
                    )*
                ))
            }

        }

        unsafe impl<$($A:AttribData),*> AttribData for ($($A,)*) {
            type Format = ($(OffsetFormat<$A::Format>,)*);
            type GLSL = ($($A::GLSL,)*);

            fn format() -> Self::Format {

                //Here, since there are no guarrantees on the memory layout of tuples,
                //we create an uninitialized tuple and use pointer offsets to figure out
                //exactly where each element is. Yes, this is gross and potentially inefficient,
                //but it should pretty much always get constant folded and/or optimized by the
                //compiler at least...

                let x = MaybeUninit::<Self>::uninit();
                let base_ptr = x.as_ptr();
                let ($($a,)*) = unsafe { &*base_ptr };
                let fmt = (
                    $(
                        OffsetFormat {
                            fmt: $A::format(),
                            offset: ($a as *const $A as usize) - (base_ptr as usize) as usize
                        },
                    )*
                );

                fmt
            }
        }
    }
}

impl_tuple!(
    tuple_format
);

macro_rules! attrib_data {
    ($($ty:ty, $format:ty, $glsl:ty, $expr:expr;)*) => {
        $(
            unsafe impl AttribData for $ty {
                type Format = $format;
                type GLSL = $glsl;
                fn format() -> $format {
                    $expr
                }
            }
        )*
    }
}

use self::IntType::*;
use self::FloatType::*;

attrib_data! {

    bool, IVecFormat, gl_bool, IVecFormat(UByte, 1);
    gl_bool, IVecFormat, gl_bool, IVecFormat(UInt, 1);
    bvec2, IVecFormat, bvec2, IVecFormat(UByte, 2);
    bvec3, IVecFormat, bvec3, IVecFormat(UByte, 3);
    bvec4, IVecFormat, bvec4, IVecFormat(UByte, 4);

    u8, IVecFormat, uint, IVecFormat(UByte, 1);
    u16, IVecFormat, uint, IVecFormat(UShort, 1);
    u32, IVecFormat, uint, IVecFormat(UInt, 1);
    uvec2, IVecFormat, uvec2, IVecFormat(UInt, 2);
    uvec3, IVecFormat, uvec3, IVecFormat(UInt, 3);
    uvec4, IVecFormat, uvec4, IVecFormat(UInt, 4);

    i8, IVecFormat, int, IVecFormat(Byte, 1);
    i16, IVecFormat, int, IVecFormat(Short, 1);
    i32, IVecFormat, int, IVecFormat(Int, 1);
    ivec2, IVecFormat, ivec2, IVecFormat(Int, 2);
    ivec3, IVecFormat, ivec3, IVecFormat(Int, 3);
    ivec4, IVecFormat, ivec4, IVecFormat(Int, 4);

    f32, VecFormat, float, VecFormat::Float(Float, 1);
    vec2, VecFormat, vec2, VecFormat::Float(Float, 2);
    vec3, VecFormat, vec3, VecFormat::Float(Float, 3);
    vec4, VecFormat, vec4, VecFormat::Float(Float, 4);

    mat2x2, Mat2Format, mat2x2, <[vec2; 2] as AttribData>::format();
    mat2x3, Mat2Format, mat2x3, <[vec3; 2] as AttribData>::format();
    mat2x4, Mat2Format, mat2x4, <[vec4; 2] as AttribData>::format();
    mat3x2, Mat3Format, mat3x2, <[vec2; 3] as AttribData>::format();
    mat3x3, Mat3Format, mat3x2, <[vec3; 3] as AttribData>::format();
    mat3x4, Mat3Format, mat3x2, <[vec4; 3] as AttribData>::format();
    mat4x2, Mat4Format, mat4x2, <[vec2; 4] as AttribData>::format();
    mat4x3, Mat4Format, mat4x2, <[vec3; 4] as AttribData>::format();
    mat4x4, Mat4Format, mat4x2, <[vec4; 4] as AttribData>::format();

    f64, DVecFormat, double, DVecFormat(1);
    dvec2, DVecFormat, dvec2, DVecFormat(2);
    dvec3, DVecFormat, dvec3, DVecFormat(3);
    dvec4, DVecFormat, dvec4, DVecFormat(4);

    dmat2x2, DMat2Format, dmat2x2, <[dvec2; 2] as AttribData>::format();
    dmat2x3, DMat2Format, dmat2x3, <[dvec3; 2] as AttribData>::format();
    dmat2x4, DMat2Format, dmat2x4, <[dvec4; 2] as AttribData>::format();
    dmat3x2, DMat3Format, dmat3x2, <[dvec2; 3] as AttribData>::format();
    dmat3x3, DMat3Format, dmat3x2, <[dvec3; 3] as AttribData>::format();
    dmat3x4, DMat3Format, dmat3x2, <[dvec4; 3] as AttribData>::format();
    dmat4x2, DMat4Format, dmat4x2, <[dvec2; 4] as AttribData>::format();
    dmat4x3, DMat4Format, dmat4x2, <[dvec3; 4] as AttribData>::format();
    dmat4x4, DMat4Format, dmat4x2, <[dvec4; 4] as AttribData>::format();

}
