use super::*;
use std::convert::TryInto;

glenum! {
    pub enum DepthComponents { DEPTH_COMPONENT }
    pub enum StencilComponents { STENCIL_INDEX }
    pub enum DepthStencilComponents { DEPTH_COMPONENT, STENCIL_INDEX, DEPTH_STENCIL }
    pub enum ColorComponents { RED, GREEN, BLUE, RG, RGB, BGR, RGBA, BGRA }
    pub enum IntColorComponents {
        RED_INTEGER, GREEN_INTEGER, BLUE_INTEGER,
        RG_INTEGER, RGB_INTEGER, BGR_INTEGER, RGBA_INTEGER, BGRA_INTEGER
    }

    pub enum PixelFormat {
        DEPTH_COMPONENT,
        STENCIL_INDEX,
        DEPTH_STENCIL,
        RED, GREEN, BLUE, RG, RGB, BGR, RGBA, BGRA,
        RED_INTEGER, GREEN_INTEGER, BLUE_INTEGER,
        RG_INTEGER, RGB_INTEGER, BGR_INTEGER, RGBA_INTEGER, BGRA_INTEGER
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

impl ColorComponents {
    #[inline]
    pub fn into_int(self) -> IntColorComponents {
        match self {
            Self::RED => IntColorComponents::RED_INTEGER,
            Self::GREEN => IntColorComponents::GREEN_INTEGER,
            Self::BLUE => IntColorComponents::BLUE_INTEGER,
            Self::RG => IntColorComponents::RG_INTEGER,
            Self::RGB => IntColorComponents::RGB_INTEGER,
            Self::BGR => IntColorComponents::BGR_INTEGER,
            Self::RGBA => IntColorComponents::RGBA_INTEGER,
            Self::BGRA => IntColorComponents::BGRA_INTEGER
        }
    }
}

impl From<ColorComponents> for PixelFormat {
    fn from(fmt: ColorComponents) -> Self {
        match fmt {
            ColorComponents::RED => Self::RED,
            ColorComponents::GREEN => Self::GREEN,
            ColorComponents::BLUE => Self::BLUE,
            ColorComponents::RG => Self::RG,
            ColorComponents::RGB => Self::RGB,
            ColorComponents::BGR => Self::BGR,
            ColorComponents::RGBA => Self::RGBA,
            ColorComponents::BGRA => Self::BGRA
        }
    }
}

impl IntColorComponents {
    #[inline]
    pub fn into_float(self) -> ColorComponents {
        match self {
            Self::RED_INTEGER => ColorComponents::RED,
            Self::GREEN_INTEGER => ColorComponents::GREEN,
            Self::BLUE_INTEGER => ColorComponents::BLUE,
            Self::RG_INTEGER => ColorComponents::RG,
            Self::RGB_INTEGER => ColorComponents::RGB,
            Self::BGR_INTEGER => ColorComponents::BGR,
            Self::RGBA_INTEGER => ColorComponents::RGBA,
            Self::BGRA_INTEGER => ColorComponents::BGRA
        }
    }
}

impl From<IntColorComponents> for PixelFormat {
    fn from(fmt: IntColorComponents) -> Self {
        match fmt {
            IntColorComponents::RED_INTEGER => Self::RED_INTEGER,
            IntColorComponents::GREEN_INTEGER => Self::GREEN_INTEGER,
            IntColorComponents::BLUE_INTEGER => Self::BLUE_INTEGER,
            IntColorComponents::RG_INTEGER => Self::RG_INTEGER,
            IntColorComponents::RGB_INTEGER => Self::RGB_INTEGER,
            IntColorComponents::BGR_INTEGER => Self::BGR_INTEGER,
            IntColorComponents::RGBA_INTEGER => Self::RGBA_INTEGER,
            IntColorComponents::BGRA_INTEGER => Self::BGRA_INTEGER
        }
    }
}

glenum! {
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
    #[inline] fn from(f:FloatType) -> Self {(f as GLenum).try_into().unwrap()}
}

impl From<IntType> for PixelType {
    #[inline] fn from(f:IntType) -> Self {(f as GLenum).try_into().unwrap()}
}

pub unsafe trait ClientFormat: Copy+Clone+PartialEq+Eq+Hash+Debug {
    fn fmt(self) -> PixelFormat;
    fn ty(self) -> PixelType;
}

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub enum ClientFormatInt {
    Integer(IntColorComponents, IntType),
    UShort4_4_4_4, UShort4_4_4_4Rev,
    UShort5_5_5_1, UShort1_5_5_5Rev,
    UInt8_8_8_8, UInt8_8_8_8Rev,
    UInt10_10_10_2, UInt10_10_10_2Rev
}

display_from_debug!(ClientFormatInt);

unsafe impl ClientFormat for ClientFormatInt {

    #[inline]
    fn fmt(self) -> PixelFormat {
        match self {
            Self::Integer(format, _) => format.into(),
            _ => PixelFormat::RGBA_INTEGER
        }
    }

    #[inline]
    fn ty(self) -> PixelType {
        match self {
            Self::Integer(_, ty) => ty.into(),
            Self::UShort4_4_4_4 => PixelType::UNSIGNED_SHORT_4_4_4_4,
            Self::UShort4_4_4_4Rev => PixelType::UNSIGNED_SHORT_4_4_4_4_REV,
            Self::UShort5_5_5_1 => PixelType::UNSIGNED_SHORT_5_5_5_1,
            Self::UShort1_5_5_5Rev => PixelType::UNSIGNED_SHORT_1_5_5_5_REV,
            Self::UInt8_8_8_8 => PixelType::UNSIGNED_INT_8_8_8_8,
            Self::UInt8_8_8_8Rev => PixelType::UNSIGNED_INT_8_8_8_8_REV,
            Self::UInt10_10_10_2 => PixelType::UNSIGNED_INT_10_10_10_2,
            Self::UInt10_10_10_2Rev => PixelType::UNSIGNED_INT_2_10_10_10_REV
        }
    }
}

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub enum ClientFormatFloat {
    Float(ColorComponents, FloatType),
    Normalized(ClientFormatInt),
    UByte3_3_2, UByte2_3_3Rev,
    UShort5_6_5, UShort5_6_5Rev
}

display_from_debug!(ClientFormatFloat);

impl From<ClientFormatInt> for ClientFormatFloat {
    fn from(fmt: ClientFormatInt) -> Self { ClientFormatFloat::Normalized(fmt) }
}

unsafe impl ClientFormat for ClientFormatFloat {

    #[inline]
    fn fmt(self) -> PixelFormat {
        match self {
            Self::Float(format, _) => format.into(),
            Self::Normalized(int) => int.fmt(),
            Self::UByte3_3_2 | Self::UByte2_3_3Rev |
            Self::UShort5_6_5 | Self::UShort5_6_5Rev => PixelFormat::RGB,
        }
    }

    #[inline]
    fn ty(self) -> PixelType {
        match self {
            Self::Float(_, ty) => ty.r#into(),
            Self::Normalized(int) => int.ty(),
            Self::UByte3_3_2 => PixelType::UNSIGNED_BYTE_3_3_2,
            Self::UByte2_3_3Rev => PixelType::UNSIGNED_BYTE_2_3_3_REV,
            Self::UShort5_6_5 => PixelType::UNSIGNED_SHORT_5_6_5,
            Self::UShort5_6_5Rev => PixelType::UNSIGNED_SHORT_5_6_5_REV
        }
    }
}

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub enum ClientFormatDepth {
    Float(FloatType),
    Normalized(IntType)
}

impl From<FloatType> for ClientFormatDepth {
    fn from(ty:FloatType) -> Self { ClientFormatDepth::Float(ty) }
}

impl From<IntType> for ClientFormatDepth {
    fn from(ty:IntType) -> Self { ClientFormatDepth::Normalized(ty) }
}

display_from_debug!(ClientFormatDepth);

unsafe impl ClientFormat for ClientFormatDepth {

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
pub struct ClientFormatStencil(pub IntType);

display_from_debug!(ClientFormatStencil);

impl From<IntType> for ClientFormatStencil {
    fn from(ty:IntType) -> Self { ClientFormatStencil(ty) }
}

unsafe impl ClientFormat for ClientFormatStencil {
    #[inline] fn fmt(self) -> PixelFormat { PixelFormat::STENCIL_INDEX }
    #[inline] fn ty(self) -> PixelType { self.0.into() }
}

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub enum ClientFormatDepthStencil {
    DepthComponent(ClientFormatDepth),
    StencilIndex(ClientFormatStencil),
    UInt24_8
}

impl From<FloatType> for ClientFormatDepthStencil {
    fn from(fmt:FloatType) -> Self { ClientFormatDepth::from(fmt).into() }
}

impl From<ClientFormatDepth> for ClientFormatDepthStencil {
    fn from(fmt:ClientFormatDepth) -> Self { ClientFormatDepthStencil::DepthComponent(fmt) }
}

impl From<ClientFormatStencil> for ClientFormatDepthStencil {
    fn from(fmt:ClientFormatStencil) -> Self { ClientFormatDepthStencil::StencilIndex(fmt) }
}

unsafe impl ClientFormat for ClientFormatDepthStencil {

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
            Self::DepthComponent(ty) => ty.ty().into(),
            Self::StencilIndex(ty) => ty.ty().into(),
            Self::UInt24_8 => PixelType::UNSIGNED_INT_24_8,
        }
    }

}

display_from_debug!(ClientFormatDepthStencil);
