use super::*;
use std::convert::TryInto;

glenum! {
    pub enum FormatDepth { DEPTH_COMPONENT }
    pub enum FormatStencil { STENCIL_INDEX }
    pub enum FormatDepthStencil { DEPTH_COMPONENT, STENCIL_INDEX, DEPTH_STENCIL }
    pub enum FormatFloat { RED, GREEN, BLUE, RG, RGB, BGR, RGBA, BGRA }
    pub enum FormatInt {
        RED_INTEGER, GREEN_INTEGER, BLUE_INTEGER, RG_INTEGER, RGB_INTEGER, BGR_INTEGER, RGBA_INTEGER, BGRA_INTEGER
    }
}

impl From<FormatInt> for FormatFloat {
    #[inline]
    fn from(fmt: FormatInt) -> Self {
        match fmt {
            FormatInt::RED_INTEGER => Self::RED,
            FormatInt::GREEN_INTEGER => Self::GREEN,
            FormatInt::BLUE_INTEGER => Self::BLUE,
            FormatInt::RG_INTEGER => Self::RG,
            FormatInt::RGB_INTEGER => Self::RGB,
            FormatInt::BGR_INTEGER => Self::BGR,
            FormatInt::RGBA_INTEGER => Self::RGBA,
            FormatInt::BGRA_INTEGER => Self::BGRA
        }
    }
}

impl From<FormatFloat> for FormatInt {
    #[inline]
    fn from(fmt: FormatFloat) -> Self {
        match fmt {
            FormatFloat::RED => Self::RED_INTEGER,
            FormatFloat::GREEN => Self::GREEN_INTEGER,
            FormatFloat::BLUE => Self::BLUE_INTEGER,
            FormatFloat::RG => Self::RG_INTEGER,
            FormatFloat::RGB => Self::RGB_INTEGER,
            FormatFloat::BGR => Self::BGR_INTEGER,
            FormatFloat::RGBA => Self::RGBA_INTEGER,
            FormatFloat::BGRA => Self::BGRA_INTEGER
        }
    }
}

impl From<FormatDepth> for FormatDepthStencil {
    #[inline] fn from(_fmt: FormatDepth) -> Self {Self::DEPTH_COMPONENT}
}

impl From<FormatStencil> for FormatDepthStencil {
    #[inline] fn from(_fmt: FormatStencil) -> Self {Self::STENCIL_INDEX}
}

pub unsafe trait PixelFormat: GLEnum { fn components(self) -> usize; }

unsafe impl PixelFormat for FormatDepth { #[inline] fn components(self) -> usize {1} }
unsafe impl PixelFormat for FormatStencil { #[inline] fn components(self) -> usize {1} }
unsafe impl PixelFormat for FormatDepthStencil {
    #[inline] fn components(self) -> usize {
        if self == FormatDepthStencil::DEPTH_STENCIL {2} else {1}
    }
}
unsafe impl PixelFormat for FormatFloat {
    #[inline]
    fn components(self) -> usize {
        match self {
            Self::RED | Self::GREEN | Self::BLUE => 1,
            Self::RG => 2,
            Self::RGB | Self::BGR => 3,
            Self::RGBA | Self::BGRA => 4,
        }
    }
}
unsafe impl PixelFormat for FormatInt {
    #[inline]
    fn components(self) -> usize {
        match self {
            Self::RED_INTEGER | Self::GREEN_INTEGER | Self::BLUE_INTEGER => 1,
            Self::RG_INTEGER => 2,
            Self::RGB_INTEGER | Self::BGR_INTEGER => 3,
            Self::RGBA_INTEGER | Self::BGRA_INTEGER => 4,
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

impl From<FloatType> for PixelType {
    #[inline] fn from(f:FloatType) -> Self {(f as GLenum).try_into().unwrap()}
}

impl From<IntType> for PixelType {
    #[inline] fn from(f:IntType) -> Self {(f as GLenum).try_into().unwrap()}
}

pub trait ClientFormat: Copy+Clone+PartialEq+Eq+Hash+Debug {
    type Format: PixelFormat;
    fn size(self) -> usize;
    unsafe fn format_type(self) -> (Self::Format, PixelType);
}

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub enum ClientFormatInt {
    Integer(FormatInt, IntType),
    UShort4_4_4_4, UShort4_4_4_4Rev,
    UShort5_5_5_1, UShort1_5_5_5Rev,
    UInt8_8_8_8, UInt8_8_8_8Rev,
    UInt10_10_10_2, UInt10_10_10_2Rev
}

display_from_debug!(ClientFormatInt);

impl ClientFormat for ClientFormatInt {
    type Format = FormatInt;

    #[inline]
    fn size(self) -> usize {
        match self {
            Self::Integer(format, ty) => format.components() * ty.size_of(),
            Self::UShort4_4_4_4 | Self::UShort4_4_4_4Rev | Self::UShort5_5_5_1 | Self::UShort1_5_5_5Rev => 2,
            Self::UInt8_8_8_8 | Self::UInt8_8_8_8Rev | Self::UInt10_10_10_2 | Self::UInt10_10_10_2Rev => 4
        }
    }

    #[inline]
    unsafe fn format_type(self) -> (Self::Format, PixelType) {
        match self {
            Self::Integer(format, ty) => (format, ty.into()),
            _ => (
                FormatInt::RGBA_INTEGER,
                match self {
                    Self::UShort4_4_4_4 => PixelType::UNSIGNED_SHORT_4_4_4_4,
                    Self::UShort4_4_4_4Rev => PixelType::UNSIGNED_SHORT_4_4_4_4_REV,
                    Self::UShort5_5_5_1 => PixelType::UNSIGNED_SHORT_5_5_5_1,
                    Self::UShort1_5_5_5Rev => PixelType::UNSIGNED_SHORT_1_5_5_5_REV,
                    Self::UInt8_8_8_8 => PixelType::UNSIGNED_INT_8_8_8_8,
                    Self::UInt8_8_8_8Rev => PixelType::UNSIGNED_INT_8_8_8_8_REV,
                    Self::UInt10_10_10_2 => PixelType::UNSIGNED_INT_10_10_10_2,
                    Self::UInt10_10_10_2Rev => PixelType::UNSIGNED_INT_2_10_10_10_REV,
                    _ => panic!("Unknown type: {}", self)
                }
            )
        }
    }
}

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub enum ClientFormatFloat {
    Float(FormatFloat, FloatType),
    Normalized(ClientFormatInt),
    UByte3_3_2, UByte2_3_3Rev,
    UShort5_6_5, UShort5_6_5Rev
}

display_from_debug!(ClientFormatFloat);

impl From<ClientFormatInt> for ClientFormatFloat {
    fn from(fmt: ClientFormatInt) -> Self { ClientFormatFloat::Normalized(fmt) }
}

impl ClientFormat for ClientFormatFloat {
    type Format = FormatFloat;

    #[inline]
    fn size(self) -> usize {
        match self {
            Self::Float(format, ty) => format.components() * ty.size_of(),
            Self::Normalized(int) => int.size(),
            Self::UByte3_3_2 | Self::UByte2_3_3Rev => 1,
            Self::UShort5_6_5 | Self::UShort5_6_5Rev => 2
        }
    }

    #[inline]
    unsafe fn format_type(self) -> (Self::Format, PixelType) {
        match self {
            Self::Float(format, ty) => (format, ty.into()),
            Self::Normalized(int) => {let ft = int.format_type(); (ft.0.into(), ft.1)},
            Self::UByte3_3_2 => (FormatFloat::RGB, PixelType::UNSIGNED_BYTE_3_3_2),
            Self::UByte2_3_3Rev => (FormatFloat::RGB, PixelType::UNSIGNED_BYTE_2_3_3_REV),
            Self::UShort5_6_5 => (FormatFloat::RGB, PixelType::UNSIGNED_SHORT_5_6_5),
            Self::UShort5_6_5Rev => (FormatFloat::RGB, PixelType::UNSIGNED_SHORT_5_6_5_REV)
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

impl ClientFormat for ClientFormatDepth {
    type Format = FormatDepth;

    #[inline]
    fn size(self) -> usize {
        match self {
            Self::Float(ty) => ty.size_of(),
            Self::Normalized(ty) => ty.size_of()
        }
    }

    #[inline]
    unsafe fn format_type(self) -> (Self::Format, PixelType) {
        match self {
            Self::Float(ty) => (FormatDepth::DEPTH_COMPONENT, ty.into()),
            Self::Normalized(ty) => (FormatDepth::DEPTH_COMPONENT, ty.into())
        }
    }
}

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub struct ClientFormatStencil(pub IntType);

display_from_debug!(ClientFormatStencil);

impl From<IntType> for ClientFormatStencil {
    fn from(ty:IntType) -> Self { ClientFormatStencil(ty) }
}

impl ClientFormat for ClientFormatStencil {
    type Format = FormatStencil;
    #[inline] fn size(self) -> usize { self.0.size_of() }
    #[inline] unsafe fn format_type(self) -> (FormatStencil, PixelType) { (FormatStencil::STENCIL_INDEX, self.0.into()) }
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

impl ClientFormat for ClientFormatDepthStencil {
    type Format = FormatDepthStencil;

    #[inline]
    fn size(self) -> usize {
        match self {
            Self::DepthComponent(ty) => ty.size(),
            Self::StencilIndex(ty) => ty.size(),
            Self::UInt24_8 => 4
        }
    }

    #[inline]
    unsafe fn format_type(self) -> (Self::Format, PixelType) {
        match self {
            Self::DepthComponent(ty) => (FormatDepthStencil::DEPTH_COMPONENT, ty.format_type().1),
            Self::StencilIndex(ty) => (FormatDepthStencil::STENCIL_INDEX, ty.format_type().1),
            Self::UInt24_8 => (FormatDepthStencil::DEPTH_STENCIL, PixelType::UNSIGNED_INT_24_8),
        }
    }
}

display_from_debug!(ClientFormatDepthStencil);
