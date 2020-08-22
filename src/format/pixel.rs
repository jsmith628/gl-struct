use super::*;
use crate::version::*;
use std::convert::TryInto;

glenum! {

    //all except DEPTH_STENCIL were present since GL10
    pub enum DepthComponents { DEPTH_COMPONENT }
    pub enum StencilComponents { STENCIL_INDEX }

    //assumes GL_EXT_packed_depth_stencil as that is required to even create an depth-stencil texture
    pub enum DepthStencilComponents { DEPTH_COMPONENT, STENCIL_INDEX, DEPTH_STENCIL }

    //all except BGR and BGRA were present since GL10
    pub enum ColorComponents {
        RED, GREEN, BLUE, RG(GL30), RGB, RGBA, BGR(GL_EXT_bgra), BGRA(GL_EXT_bgra)
    }

    //assumes GL_EXT_texture_integer as that is required to even create an integer texture
    pub enum IntColorComponents {
        RED_INTEGER, GREEN_INTEGER, BLUE_INTEGER,
        RG_INTEGER(GL30), RGB_INTEGER, RGBA_INTEGER, BGR_INTEGER, BGRA_INTEGER
    }

    pub enum PixelFormat {
        DEPTH_COMPONENT,
        STENCIL_INDEX,
        DEPTH_STENCIL,
        RED, GREEN, BLUE, RG, RGB, BGR, RGBA, BGRA,
        RED_INTEGER, GREEN_INTEGER, BLUE_INTEGER,
        RG_INTEGER, RGB_INTEGER, RGBA_INTEGER, BGR_INTEGER, BGRA_INTEGER
    }

}

impl From<DepthComponents> for DepthStencilComponents {
    #[inline] fn from(_fmt: DepthComponents) -> Self {Self::DEPTH_COMPONENT}
}

impl From<DepthComponents> for PixelFormat {
    #[inline] fn from(_fmt: DepthComponents) -> Self { Self::DEPTH_COMPONENT }
}

impl From<StencilComponents> for DepthStencilComponents {
    #[inline] fn from(_fmt: StencilComponents) -> Self {Self::STENCIL_INDEX}
}

impl From<StencilComponents> for PixelFormat {
    #[inline] fn from(_fmt: StencilComponents) -> Self { Self::STENCIL_INDEX }
}

impl From<DepthStencilComponents> for PixelFormat {
    fn from(fmt: DepthStencilComponents) -> Self {
        match fmt {
            DepthStencilComponents::DEPTH_COMPONENT => Self::DEPTH_COMPONENT,
            DepthStencilComponents::STENCIL_INDEX => Self::STENCIL_INDEX,
            DepthStencilComponents::DEPTH_STENCIL => Self::DEPTH_STENCIL
        }
    }
}

impl From<ColorComponents> for PixelFormat {
    fn from(fmt: ColorComponents) -> Self {
        match fmt {
            ColorComponents::RED => Self::RED,
            ColorComponents::GREEN => Self::GREEN,
            ColorComponents::BLUE => Self::BLUE,
            ColorComponents::RG(_) => Self::RG,
            ColorComponents::RGB => Self::RGB,
            ColorComponents::BGR(_) => Self::BGR,
            ColorComponents::RGBA => Self::RGBA,
            ColorComponents::BGRA(_) => Self::BGRA
        }
    }
}

impl From<IntColorComponents> for PixelFormat {
    fn from(fmt: IntColorComponents) -> Self {
        match fmt {
            IntColorComponents::RED_INTEGER => Self::RED_INTEGER,
            IntColorComponents::GREEN_INTEGER => Self::GREEN_INTEGER,
            IntColorComponents::BLUE_INTEGER => Self::BLUE_INTEGER,
            IntColorComponents::RG_INTEGER(_) => Self::RG_INTEGER,
            IntColorComponents::RGB_INTEGER => Self::RGB_INTEGER,
            IntColorComponents::BGR_INTEGER => Self::BGR_INTEGER,
            IntColorComponents::RGBA_INTEGER => Self::RGBA_INTEGER,
            IntColorComponents::BGRA_INTEGER => Self::BGRA_INTEGER
        }
    }
}

glenum! {

    pub enum FloatType {
        [Half(GL_ARB_half_float_pixel) HALF_FLOAT "Half"],
        [Float FLOAT "Float"]
    }

    pub enum PixelType {
        UNSIGNED_BYTE, BYTE, UNSIGNED_SHORT, SHORT, UNSIGNED_INT, INT,
        HALF_FLOAT, FLOAT,
        UNSIGNED_BYTE_3_3_2, UNSIGNED_BYTE_2_3_3_REV,
        UNSIGNED_SHORT_5_6_5, UNSIGNED_SHORT_5_6_5_REV,
        UNSIGNED_SHORT_4_4_4_4, UNSIGNED_SHORT_4_4_4_4_REV,
        UNSIGNED_SHORT_5_5_5_1, UNSIGNED_SHORT_1_5_5_5_REV,
        UNSIGNED_INT_8_8_8_8, UNSIGNED_INT_8_8_8_8_REV,
        UNSIGNED_INT_10_10_10_2, UNSIGNED_INT_2_10_10_10_REV,
        UNSIGNED_INT_10F_11F_11F_REV,
        UNSIGNED_INT_5_9_9_9_REV,
        UNSIGNED_INT_24_8,
        FLOAT_32_UNSIGNED_INT_24_8_REV
    }
}

impl FloatType {
    #[inline]
    pub fn size_of(self) -> usize {
        match self {
            FloatType::Half(_) => 2,
            FloatType::Float => 4,
        }
    }
}

impl PixelType {
    pub fn size(self) -> usize {
        match self {

            Self::UNSIGNED_BYTE | Self::BYTE => 1,
            Self::UNSIGNED_SHORT | Self::SHORT => 2,
            Self::UNSIGNED_INT | Self::INT => 4,
            Self::HALF_FLOAT => 2,
            Self::FLOAT => 4,

            Self::UNSIGNED_BYTE_3_3_2 | Self::UNSIGNED_BYTE_2_3_3_REV => 1,

            Self::UNSIGNED_SHORT_5_6_5 | Self::UNSIGNED_SHORT_5_6_5_REV |
            Self::UNSIGNED_SHORT_4_4_4_4 | Self::UNSIGNED_SHORT_4_4_4_4_REV |
            Self::UNSIGNED_SHORT_5_5_5_1 | Self::UNSIGNED_SHORT_1_5_5_5_REV => 2,

            Self::UNSIGNED_INT_8_8_8_8 | Self::UNSIGNED_INT_8_8_8_8_REV |
            Self::UNSIGNED_INT_10_10_10_2 | Self::UNSIGNED_INT_2_10_10_10_REV |
            Self::UNSIGNED_INT_10F_11F_11F_REV |
            Self::UNSIGNED_INT_5_9_9_9_REV |
            Self::UNSIGNED_INT_24_8 => 4,

            Self::FLOAT_32_UNSIGNED_INT_24_8_REV => 4

        }
    }
}

impl From<FloatType> for PixelType {
    #[inline] fn from(f:FloatType) -> Self {GLenum::from(f).try_into().unwrap()}
}

impl From<IntType> for PixelType {
    #[inline] fn from(f:IntType) -> Self {(f as GLenum).try_into().unwrap()}
}

pub unsafe trait PixelLayout: Copy+Clone+PartialEq+Eq+Hash+Debug {
    fn fmt(self) -> PixelFormat;
    fn ty(self) -> PixelType;
}

//assumes GL_EXT_texture_integer as that is needed to create integer textures
#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub enum IntLayout {
    Integer(IntColorComponents, IntType),
    UByte3_3_2(GL_EXT_packed_pixels),           UByte2_3_3Rev(GL12),
    UShort5_6_5(GL12),                          UShort5_6_5Rev(GL12),
    UShort4_4_4_4(GL_EXT_packed_pixels, bool),  UShort4_4_4_4Rev(GL12, bool),
    UShort5_5_5_1(GL_EXT_packed_pixels, bool),  UShort1_5_5_5Rev(GL12, bool),
    UInt8_8_8_8(GL_EXT_packed_pixels, bool),    UInt8_8_8_8Rev(GL12, bool),
    UInt10_10_10_2(GL_EXT_packed_pixels, bool), UInt10_10_10_2Rev(GL12, bool)
}

display_from_debug!(IntLayout);

unsafe impl PixelLayout for IntLayout {

    #[inline]
    fn fmt(self) -> PixelFormat {
        match self {
            Self::Integer(format, _) => format.into(),

            //MUST be RGB
            Self::UByte3_3_2(_)  | Self::UByte2_3_3Rev(_) |
            Self::UShort5_6_5(_) | Self::UShort5_6_5Rev(_) => PixelFormat::RGB_INTEGER,

            //must be RBGA or BGRA
            Self::UShort4_4_4_4(_, b)  | Self::UShort4_4_4_4Rev(_, b) |
            Self::UShort5_5_5_1(_, b)  | Self::UShort1_5_5_5Rev(_, b) |
            Self::UInt8_8_8_8(_, b)    | Self::UInt8_8_8_8Rev(_, b)   |
            Self::UInt10_10_10_2(_, b) | Self::UInt10_10_10_2Rev(_, b) => {
                if b { PixelFormat::RGBA_INTEGER } else { PixelFormat::BGRA_INTEGER }
            }
        }
    }

    #[inline]
    fn ty(self) -> PixelType {
        match self {
            Self::Integer(_, ty)         => ty.into(),
            Self::UByte3_3_2(_)          => PixelType::UNSIGNED_BYTE_3_3_2,
            Self::UByte2_3_3Rev(_)       => PixelType::UNSIGNED_BYTE_2_3_3_REV,
            Self::UShort5_6_5(_)         => PixelType::UNSIGNED_SHORT_5_6_5,
            Self::UShort5_6_5Rev(_)      => PixelType::UNSIGNED_SHORT_5_6_5_REV,
            Self::UShort4_4_4_4(_,_)     => PixelType::UNSIGNED_SHORT_4_4_4_4,
            Self::UShort4_4_4_4Rev(_,_)  => PixelType::UNSIGNED_SHORT_4_4_4_4_REV,
            Self::UShort5_5_5_1(_,_)     => PixelType::UNSIGNED_SHORT_5_5_5_1,
            Self::UShort1_5_5_5Rev(_,_)  => PixelType::UNSIGNED_SHORT_1_5_5_5_REV,
            Self::UInt8_8_8_8(_,_)       => PixelType::UNSIGNED_INT_8_8_8_8,
            Self::UInt8_8_8_8Rev(_,_)    => PixelType::UNSIGNED_INT_8_8_8_8_REV,
            Self::UInt10_10_10_2(_,_)    => PixelType::UNSIGNED_INT_10_10_10_2,
            Self::UInt10_10_10_2Rev(_,_) => PixelType::UNSIGNED_INT_2_10_10_10_REV
        }
    }
}

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub enum FloatLayout {
    Float(ColorComponents, FloatType),
    Normalized(ColorComponents, IntType),

    //packed normalized types
    UByte3_3_2(GL_EXT_packed_pixels),           UByte2_3_3Rev(GL12),
    UShort5_6_5(GL12),                          UShort5_6_5Rev(GL12),
    UShort4_4_4_4(GL_EXT_packed_pixels, bool),  UShort4_4_4_4Rev(GL12, bool),
    UShort5_5_5_1(GL_EXT_packed_pixels, bool),  UShort1_5_5_5Rev(GL12, bool),
    UInt8_8_8_8(GL_EXT_packed_pixels, bool),    UInt8_8_8_8Rev(GL12, bool),
    UInt10_10_10_2(GL_EXT_packed_pixels, bool), UInt10_10_10_2Rev(GL12, bool),

    //packed floating point types
    #[allow(non_camel_case_types)]
    UInt10F_11F_11F_Rev(GL_EXT_packed_float),
    UInt5_9_9_9Rev(GL_EXT_texture_shared_exponent)
}

display_from_debug!(FloatLayout);

//TODO: find some way to convert between int and float formats

// impl From<IntLayout> for FloatLayout {
//     fn from(fmt: IntLayout) -> Self {
//         match fmt {
//             IntLayout::Integer(fmt, ty)        => Self::Normalized(fmt, ty.into_float()),
//             IntLayout::UByte3_3_2(gl)          => Self::UByte3_3_2(gl),
//             IntLayout::UByte2_3_3Rev(gl)       => Self::UByte2_3_3Rev(gl),
//             IntLayout::UShort5_6_5(gl)         => Self::UShort5_6_5(gl),
//             IntLayout::UShort5_6_5Rev(gl)      => Self::UShort5_6_5Rev(gl),
//             IntLayout::UShort4_4_4_4(gl,b)     => Self::UShort4_4_4_4(gl,b),
//             IntLayout::UShort4_4_4_4Rev(gl,b)  => Self::UShort4_4_4_4Rev(gl,b),
//             IntLayout::UShort5_5_5_1(gl,b)     => Self::UShort5_5_5_1(gl,b),
//             IntLayout::UShort1_5_5_5Rev(gl,b)  => Self::UShort1_5_5_5Rev(gl,b),
//             IntLayout::UInt8_8_8_8(gl,b)       => Self::UInt8_8_8_8(gl,b),
//             IntLayout::UInt8_8_8_8Rev(gl,b)    => Self::UInt8_8_8_8Rev(gl,b),
//             IntLayout::UInt10_10_10_2(gl,b)    => Self::UInt10_10_10_2(gl,b),
//             IntLayout::UInt10_10_10_2Rev(gl,b) => Self::UInt10_10_10_2Rev(gl,b),
//         }
//     }
// }

unsafe impl PixelLayout for FloatLayout {

    #[inline]
    fn fmt(self) -> PixelFormat {
        match self {
            Self::Float(fmt, _) => fmt.into(),
            Self::Normalized(fmt, _) => fmt.into(),

            //MUST be RGB
            Self::UByte3_3_2(_)     | Self::UByte2_3_3Rev(_)  |
            Self::UShort5_6_5(_)    | Self::UShort5_6_5Rev(_) |
            Self::UInt5_9_9_9Rev(_) | Self::UInt10F_11F_11F_Rev(_) => PixelFormat::RGB,

            //must be RBGA or BGRA
            Self::UShort4_4_4_4(_, b)  | Self::UShort4_4_4_4Rev(_, b) |
            Self::UShort5_5_5_1(_, b)  | Self::UShort1_5_5_5Rev(_, b) |
            Self::UInt8_8_8_8(_, b)    | Self::UInt8_8_8_8Rev(_, b)   |
            Self::UInt10_10_10_2(_, b) | Self::UInt10_10_10_2Rev(_, b) => {
                if b { PixelFormat::RGBA } else { PixelFormat::BGRA }
            },
        }
    }

    #[inline]
    fn ty(self) -> PixelType {
        match self {
            Self::Float(_, ty)           => ty.into(),
            Self::Normalized(_, ty)      => ty.into(),
            Self::UByte3_3_2(_)          => PixelType::UNSIGNED_BYTE_3_3_2,
            Self::UByte2_3_3Rev(_)       => PixelType::UNSIGNED_BYTE_2_3_3_REV,
            Self::UShort5_6_5(_)         => PixelType::UNSIGNED_SHORT_5_6_5,
            Self::UShort5_6_5Rev(_)      => PixelType::UNSIGNED_SHORT_5_6_5_REV,
            Self::UShort4_4_4_4(_,_)     => PixelType::UNSIGNED_SHORT_4_4_4_4,
            Self::UShort4_4_4_4Rev(_,_)  => PixelType::UNSIGNED_SHORT_4_4_4_4_REV,
            Self::UShort5_5_5_1(_,_)     => PixelType::UNSIGNED_SHORT_5_5_5_1,
            Self::UShort1_5_5_5Rev(_,_)  => PixelType::UNSIGNED_SHORT_1_5_5_5_REV,
            Self::UInt8_8_8_8(_,_)       => PixelType::UNSIGNED_INT_8_8_8_8,
            Self::UInt8_8_8_8Rev(_,_)    => PixelType::UNSIGNED_INT_8_8_8_8_REV,
            Self::UInt10_10_10_2(_,_)    => PixelType::UNSIGNED_INT_10_10_10_2,
            Self::UInt10_10_10_2Rev(_,_) => PixelType::UNSIGNED_INT_2_10_10_10_REV,
            Self::UInt5_9_9_9Rev(_)      => PixelType::UNSIGNED_INT_5_9_9_9_REV,
            Self::UInt10F_11F_11F_Rev(_) => PixelType::UNSIGNED_INT_10F_11F_11F_REV
        }
    }
}

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub enum DepthLayout {
    Float(FloatType),
    Normalized(IntType)
}

impl From<FloatType> for DepthLayout {
    fn from(ty:FloatType) -> Self { DepthLayout::Float(ty) }
}

impl From<IntType> for DepthLayout {
    fn from(ty:IntType) -> Self { DepthLayout::Normalized(ty) }
}

display_from_debug!(DepthLayout);

unsafe impl PixelLayout for DepthLayout {

    #[inline] fn fmt(self) -> PixelFormat { PixelFormat::DEPTH_COMPONENT }

    #[inline]
    fn ty(self) -> PixelType {
        match self {
            Self::Float(ty) => ty.into(),
            Self::Normalized(ty) => ty.into()
        }
    }

}

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub struct StencilLayout(pub IntType);

display_from_debug!(StencilLayout);

impl From<IntType> for StencilLayout {
    fn from(ty:IntType) -> Self { StencilLayout(ty) }
}

unsafe impl PixelLayout for StencilLayout {
    #[inline] fn fmt(self) -> PixelFormat { PixelFormat::STENCIL_INDEX }
    #[inline] fn ty(self) -> PixelType { self.0.into() }
}

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub enum DepthStencilLayout {
    DepthComponent(DepthLayout),
    StencilIndex(StencilLayout),
    UInt24_8
}

impl From<FloatType> for DepthStencilLayout {
    fn from(fmt:FloatType) -> Self { DepthLayout::from(fmt).into() }
}

impl From<DepthLayout> for DepthStencilLayout {
    fn from(fmt:DepthLayout) -> Self { DepthStencilLayout::DepthComponent(fmt) }
}

impl From<StencilLayout> for DepthStencilLayout {
    fn from(fmt:StencilLayout) -> Self { DepthStencilLayout::StencilIndex(fmt) }
}

unsafe impl PixelLayout for DepthStencilLayout {

    #[inline]
    fn fmt(self) -> PixelFormat {
        match self {
            Self::DepthComponent(_) => PixelFormat::DEPTH_COMPONENT,
            Self::StencilIndex(_) => PixelFormat::STENCIL_INDEX,
            Self::UInt24_8 => PixelFormat::DEPTH_STENCIL,
        }
    }

    #[inline]
    fn ty(self) -> PixelType {
        match self {
            Self::DepthComponent(ty) => ty.ty(),
            Self::StencilIndex(ty) => ty.ty(),
            Self::UInt24_8 => PixelType::UNSIGNED_INT_24_8,
        }
    }

}

display_from_debug!(DepthStencilLayout);


pub unsafe trait Pixel<F: PixelLayout>: Copy+PartialEq {
    type GL: GLVersion;
    fn layout<GL:Supports<Self::GL>>(&GL) -> F;
    fn swap_bytes() -> bool {false}
    fn lsb_first() -> bool {false}
}

macro_rules! impl_int {
    ($($prim:ident $ty:ident)*) => {
        $(
            impl_int!(@color $prim; RED RED_INTEGER $ty);
            impl_int!(@color [$prim;1]; RED RED_INTEGER $ty);
            // impl_int!(@impl [$prim;2]; RG RG_INTEGER $ty);
            impl_int!(@color [$prim;3]; RGB RGB_INTEGER $ty);
            impl_int!(@color [$prim;4]; RGBA RGBA_INTEGER $ty);

            unsafe impl Pixel<DepthLayout> for $prim {
                type GL = ();
                fn layout<GL:Supports<()>>(_:&GL) -> DepthLayout { IntType::$ty.into() }
            }
            unsafe impl Pixel<DepthLayout> for [$prim;1] {
                type GL = ();
                fn layout<GL:Supports<()>>(_:&GL) -> DepthLayout { IntType::$ty.into() }
            }
            unsafe impl Pixel<StencilLayout> for $prim {
                type GL = ();
                fn layout<GL:Supports<()>>(_:&GL) -> StencilLayout { IntType::$ty.into() }
            }
            unsafe impl Pixel<StencilLayout> for [$prim;1] {
                type GL = ();
                fn layout<GL:Supports<()>>(_:&GL) -> StencilLayout { IntType::$ty.into() }
            }

            //since the RG format wasn't added until GL30
            unsafe impl Pixel<IntLayout> for [$prim;2] {
                type GL = GL30;
                fn layout<GL:Supports<GL30>>(gl:&GL) -> IntLayout {
                    IntLayout::Integer(IntColorComponents::RG_INTEGER(downgrade_to(gl)), IntType::$ty)
                }
            }
            unsafe impl Pixel<FloatLayout> for [$prim;2] {
                type GL = GL30;
                fn layout<GL:Supports<GL30>>(gl:&GL) -> FloatLayout {
                    FloatLayout::Normalized(ColorComponents::RG(downgrade_to(gl)), IntType::$ty)
                }
            }

        )*
    };

    (@color $prim:ty; $fmt1:ident $fmt2:ident $ty:ident) => {
        unsafe impl Pixel<IntLayout> for $prim {
            type GL = ();
            fn layout<GL:Supports<()>>(_:&GL) -> IntLayout {
                IntLayout::Integer(IntColorComponents::$fmt2, IntType::$ty)
            }
        }
        unsafe impl Pixel<FloatLayout> for $prim {
            type GL = ();
            fn layout<GL:Supports<()>>(_:&GL) -> FloatLayout {
                FloatLayout::Normalized(ColorComponents::$fmt1, IntType::$ty)
            }
        }
    };

}

impl_int!{
    GLbyte Byte
    GLubyte UByte
    GLshort Short
    GLushort UShort
    GLint Int
    GLuint UInt
}

macro_rules! impl_float {
    ($($prim:ident $ty:ident)*) => {
        $(
            impl_float!(@impl $prim; RED $ty);
            impl_float!(@impl [$prim;1]; RED $ty);
            // impl_float!(@impl [$prim;2]; RG $ty);
            impl_float!(@impl [$prim;3]; RGB $ty);
            impl_float!(@impl [$prim;4]; RGBA $ty);

            unsafe impl Pixel<DepthLayout> for $prim {
                type GL = ();
                fn layout<GL:Supports<()>>(_:&GL) -> DepthLayout { FloatType::$ty.into() }
            }
            unsafe impl Pixel<DepthLayout> for [$prim;1] {
                type GL = ();
                fn layout<GL:Supports<()>>(_:&GL) -> DepthLayout { FloatType::$ty.into() }
            }

            unsafe impl Pixel<DepthStencilLayout> for $prim {
                type GL = ();
                fn layout<GL:Supports<()>>(_:&GL) -> DepthStencilLayout { FloatType::$ty.into() }
            }
            unsafe impl Pixel<DepthStencilLayout> for [$prim;1] {
                type GL = ();
                fn layout<GL:Supports<()>>(_:&GL) -> DepthStencilLayout { FloatType::$ty.into() }
            }

            unsafe impl Pixel<FloatLayout> for [$prim; 2] {
                type GL = GL30;
                fn layout<GL:Supports<GL30>>(gl:&GL) -> FloatLayout {
                    FloatLayout::Float(ColorComponents::RG(downgrade_to(gl)), FloatType::$ty)
                }
            }

        )*
    };

    (@impl $prim:ty; $fmt:ident $ty:ident) => {
        unsafe impl Pixel<FloatLayout> for $prim {
            type GL = ();
            fn layout<GL:Supports<()>>(_:&GL) -> FloatLayout {
                FloatLayout::Float(ColorComponents::$fmt, FloatType::$ty)
            }
        }
    };

}

impl_float!(GLfloat Float);

macro_rules! impl_vec {
    (@$layout:ident $($vec:ident $inner:ty; $gl:ty),*) => {
        $(
            unsafe impl Pixel<$layout> for $vec {
                type GL = $gl;
                fn layout<GL:Supports<$gl>>(gl:&GL) -> $layout {
                    <$inner as Pixel<$layout>>::layout(gl.into())
                }
            }
        )*
    }
}

use glsl::*;

//TODO: add the 3D vecs after reverting the alignment requirements

impl_vec!(
    @IntLayout
    ivec2 [GLint; 2]; GL30,
    uvec2 [GLuint;2]; GL30,
    ivec4 [GLint; 4]; (),
    uvec4 [GLuint;4]; ()
);
impl_vec!{
    @FloatLayout
    ivec2 [GLint;  2]; GL30,
    uvec2 [GLuint; 2]; GL30,
     vec2 [GLfloat;2]; GL30,
    ivec4 [GLint;  4]; (),
    uvec4 [GLuint; 4]; (),
     vec4 [GLfloat;4]; ()
}
