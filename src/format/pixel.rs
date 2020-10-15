use super::*;

use crate::version::*;
use crate::glsl::*;
use crate::buffer::*;

use std::convert::TryInto;
use std::mem::size_of;

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
        [Byte BYTE "Byte"],
        [UByte UNSIGNED_BYTE "UByte"],
        [Short SHORT "Short"],
        [UShort UNSIGNED_SHORT "UShort"],
        [Int INT "Int"],
        [UInt UNSIGNED_INT "UInt"],
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
    pub fn size(self) -> usize {
        match self {
            Self::Byte  | Self::UByte => 1,
            Self::Short | Self::UShort | Self::Half(_) => 2,
            Self::Int   | Self::UInt   | Self::Float => 4
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

impl From<IntType> for FloatType {
    #[inline] fn from(f:IntType) -> Self {(f as GLenum).try_into().unwrap()}
}

impl From<IntType> for PixelType {
    #[inline] fn from(f:IntType) -> Self {(f as GLenum).try_into().unwrap()}
}

impl From<FloatType> for PixelType {
    #[inline] fn from(f:FloatType) -> Self {GLenum::from(f).try_into().unwrap()}
}

impl From<!> for PixelType {
    #[inline] fn from(x:!) -> Self { x }
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

impl From<IntType> for IntLayout {
    fn from(ty:IntType) -> IntLayout { IntLayout::Integer(IntColorComponents::RED_INTEGER, ty) }
}

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

impl From<IntType> for FloatLayout {
    fn from(ty:IntType) -> FloatLayout { FloatLayout::Float(ColorComponents::RED, ty.into()) }
}

impl From<FloatType> for FloatLayout {
    fn from(ty:FloatType) -> FloatLayout { FloatLayout::Float(ColorComponents::RED, ty) }
}

unsafe impl PixelLayout for FloatLayout {

    #[inline]
    fn fmt(self) -> PixelFormat {
        match self {
            Self::Float(fmt, _) => fmt.into(),

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
pub struct DepthLayout(FloatType);

impl From<FloatType> for DepthLayout {
    fn from(ty:FloatType) -> Self { DepthLayout(ty) }
}

impl From<IntType> for DepthLayout {
    fn from(ty:IntType) -> Self { DepthLayout(ty.into()) }
}

display_from_debug!(DepthLayout);

unsafe impl PixelLayout for DepthLayout {
    #[inline] fn fmt(self) -> PixelFormat { PixelFormat::DEPTH_COMPONENT }
    #[inline] fn ty(self) -> PixelType { self.0.into() }
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

pub trait ByteOrder {
    fn swap_bytes() -> bool;
}

macro_rules! impl_byte_order {
    ($($p:ty),*) => {
        $(
            impl ByteOrder for $p {
                fn swap_bytes() -> bool {false}
            }
        )*
    }
}

impl_byte_order!(

    !, (),

    u8, i8, u16, i16, u32, i32, f32, u64, i64, f64, u128, i128,

    uvec2, uvec3, uvec4,
    ivec2, ivec3, ivec4,
     vec2,  vec3,  vec4,
    dvec2, dvec3, dvec4

);

impl<F:SpecificCompressed> ByteOrder for Cmpr<F> {
    fn swap_bytes() -> bool {false}
}

impl<P:ByteOrder, const N:usize> ByteOrder for [P; N] {
    fn swap_bytes() -> bool {P::swap_bytes()}
}

impl<P:ByteOrder> ByteOrder for [P] {
    fn swap_bytes() -> bool {P::swap_bytes()}
}

pub unsafe trait SubPixel: ByteOrder + Copy + PartialEq {
    type GL: GLVersion;
}

pub unsafe trait SubPixelData<T:Copy+Into<PixelType>>: SubPixel {
    fn ty<GL:Supports<Self::GL>>(&GL) -> T;
}

unsafe impl SubPixel for ! { type GL = !; }

macro_rules! impl_subpixel {

    (Version; ) => {};
    (Version; $p:ty, $($rest:tt)*) => { impl_subpixel!(Version; $p = (), $($rest)*); };
    (Version; $p:ty) => { impl_subpixel!(Version; $p = (),); };
    (Version; $p:ty = $gl:ty) => { impl_subpixel!(Version; $p = $gl,); };
    (Version; $p:ty = $gl:ty, $($rest:tt)*) => {
        unsafe impl SubPixel for $p { type GL = $gl; }
        impl_subpixel!(Version; $($rest)*);
    };

    (; $($p:ty = $ty:ident $(($gl:ident))?),*) => { };
    ($kind0:ty $(, $kind:ty)*; $($p:ty = $ty:ident),*) => {
        $(
            unsafe impl SubPixelData<$kind0> for $p {
                fn ty<GL:Supports<Self::GL>>(_:&GL) -> $kind0 {<$kind0>::$ty}
            }
        )*
        impl_subpixel!($($kind),*; $($p = $ty),*);
    };
}

impl_subpixel!(Version; u8, i8, u16, i16, u32, i32, f32);

impl_subpixel!{
    IntType, FloatType;
    u8 = UByte, u16 = UShort, u32 = UInt, i8 = Byte, i16 = UShort, i32 = Int
}

impl_subpixel!(FloatType; f32 = Float);

pub trait PackedPixel = Pixel<SubPixel=!>;

pub unsafe trait Pixel: PixelData + Copy + PartialEq {
    type SubPixel: SubPixel;
    fn components() -> usize;
}

macro_rules! impl_pixel {

    () => {};

    ($p:ident; $($rest:tt)*) => {

        unsafe impl Pixel for $p {
            type SubPixel = $p;
            fn components() -> usize {1}
        }

        unsafe impl PixelData for $p {
            fn block_width() -> usize {1}
            fn block_height() -> usize {1}
            fn block_depth() -> usize {1}
            fn block_size() -> usize {size_of::<Self>()}
            fn len_ref(&self) -> usize {1}
            fn len_buf<A:BufferStorage>(_: Slice<Self, A>) -> usize {1}
        }

        impl_pixel!($($rest)*);
    };

    ($vec:ident as [$p:ident; $n:literal]; $($rest:tt)*) => {
        unsafe impl Pixel for $vec {
            type SubPixel = $p;
            fn components() -> usize {$n}
        }

        unsafe impl PixelData for $vec {
            fn block_width() -> usize {1}
            fn block_height() -> usize {1}
            fn block_depth() -> usize {1}
            fn block_size() -> usize {size_of::<Self>()}
            fn len_ref(&self) -> usize {1}
            fn len_buf<A:BufferStorage>(_: Slice<Self, A>) -> usize {1}
        }

        impl_pixel!($($rest)*);
    };

    ([$T:ident; $n:literal]; $($rest:tt)*) => {
        unsafe impl<$T:SubPixel> Pixel for [$T; $n] {
            type SubPixel = $T;
            fn components() -> usize {$n}
        }

        unsafe impl<$T:SubPixel> PixelData for [$T; $n] {
            fn block_width() -> usize {1}
            fn block_height() -> usize {1}
            fn block_depth() -> usize {1}
            fn block_size() -> usize {size_of::<Self>()}
            fn len_ref(&self) -> usize {1}
            fn len_buf<A:BufferStorage>(_: Slice<Self, A>) -> usize {1}
        }

        impl_pixel!($($rest)*);
    }
}

impl_pixel! {
    u8; i8; u16; i16; u32; i32; f32;
    [T; 1]; [T; 2]; [T; 3]; [T; 4];

    uvec2 as [u32;2]; uvec3 as [u32;3]; uvec4 as [u32;4];
    ivec2 as [i32;2]; ivec3 as [i32;3]; ivec4 as [i32;4];
     vec2 as [f32;2];  vec3 as [f32;3];  vec4 as [f32;4];

}

pub unsafe trait PixelData: ByteOrder {
    fn block_width() -> usize;
    fn block_height() -> usize;
    fn block_depth() -> usize;
    fn block_size() -> usize;
    fn len_ref(&self) -> usize;
    fn len_buf<A:BufferStorage>(this: Slice<Self, A>) -> usize;
}

unsafe impl<P:PixelData> PixelData for [P] {
    fn block_width() -> usize {1}
    fn block_height() -> usize {1}
    fn block_depth() -> usize {1}
    fn block_size() -> usize {size_of::<P>()}
    fn len_ref(&self) -> usize { self.len() }
    fn len_buf<A:BufferStorage>(this: Slice<Self, A>) -> usize { this.len() }
}

unsafe impl<F:SpecificCompressed> PixelData for Cmpr<F> {
    fn block_width() -> usize { F::block_width() }
    fn block_height() -> usize { F::block_height() }
    fn block_depth() -> usize { F::block_depth() }
    fn block_size() -> usize { F::block_size() }
    fn len_ref(&self) -> usize { self.len() }
    fn len_buf<A:BufferStorage>(this: Slice<Self, A>) -> usize { this.len() }
}

pub unsafe trait CompressedPixelData: PixelData {
    type Format: SpecificCompressed;
}

unsafe impl<F:SpecificCompressed> CompressedPixelData for Cmpr<F> {
    type Format = F;
}

pub unsafe trait UncompressedPixelData<F: PixelLayout>: PixelData {
    type GL: GLVersion;
    fn layout<GL:Supports<Self::GL>>(gl: &GL) -> F;
}

unsafe impl<F: PixelLayout, P:Pixel+UncompressedPixelData<F>> UncompressedPixelData<F> for [P] {
    type GL = P::GL;
    //TODO: Fix the panic that occurs when the array is empty
    fn layout<GL:Supports<Self::GL>>(gl: &GL) -> F { P::layout(gl) }
}

macro_rules! impl_vec_data {
    (; $($vec:ident as $inner:tt with GL=$gl:tt),*) => {};
    ($layout:ident $(, $layouts:ident)*; $($vec:ident as $inner:tt with GL=$gl:tt),*) => {
        $(
            unsafe impl UncompressedPixelData<$layout> for $vec {
                type GL = $gl;
                fn layout<GL:Supports<$gl>>(gl: &GL) -> $layout {
                    let gl: $gl = downgrade_to(gl);
                    <$inner as UncompressedPixelData<$layout>>::layout(&gl)
                }
            }
        )*
        impl_vec_data!($($layouts),*;  $($vec as $inner with GL=$gl),*);
    }
}

//TODO re-add vec3s once the alignment is fixed

impl_vec_data!{
    IntLayout, FloatLayout;
    uvec2 as [u32;2] with GL=GL30, ivec2 as [i32;2] with GL=GL30,
    // uvec3 as [u32;3] with GL=(),   ivec3 as [i32;3] with GL=(),
    uvec4 as [u32;4] with GL=(),   ivec4 as [i32;4] with GL=()
}

impl_vec_data!{
    FloatLayout;
    vec2 as [f32;2] with GL=GL30, vec3 as [f32;3] with GL=(), vec4 as [f32;4] with GL=()
}

macro_rules! impl_arr_data {
    ([$C:ident; 1] $ty:ident $layout:ident) => {
        unsafe impl<$C:SubPixelData<$ty>> UncompressedPixelData<$layout> for [C;1] {
            type GL = C::GL;
            fn layout<GL:Supports<Self::GL>>(gl: &GL) -> $layout { C::ty(gl).into() }
        }
    };

    ([$C:ident; $N:literal] $layout:ident::$variant:ident($comp:expr, $ty:ident)) => {
        unsafe impl<$C:SubPixelData<$ty>> UncompressedPixelData<$layout> for [C;$N] {
            type GL = C::GL;
            fn layout<GL:Supports<Self::GL>>(gl: &GL) -> $layout {
                $layout::$variant($comp, <$C as SubPixelData<$ty>>::ty(gl))
            }
        }
    }
}

impl_arr_data!([C; 4] IntLayout::Integer(IntColorComponents::RGBA_INTEGER, IntType));
impl_arr_data!([C; 4] FloatLayout::Float(ColorComponents::RGBA, FloatType));

impl_arr_data!([C; 3] IntLayout::Integer(IntColorComponents::RGB_INTEGER, IntType));
impl_arr_data!([C; 3] FloatLayout::Float(ColorComponents::RGB, FloatType));

unsafe impl<C:SubPixelData<IntType>> UncompressedPixelData<IntLayout> for [C;2] {
    type GL = (GL30, C::GL);
    fn layout<GL:Supports<(GL30, C::GL)>>(gl: &GL) -> IntLayout {
        let gl: Self::GL = downgrade_to(gl);
        IntLayout::Integer(IntColorComponents::RG_INTEGER(downgrade_to(&gl)), C::ty(&gl))
    }
}

unsafe impl<C:SubPixelData<FloatType>> UncompressedPixelData<FloatLayout> for [C;2] {
    type GL = (GL30, C::GL);
    fn layout<GL:Supports<(GL30, C::GL)>>(gl: &GL) -> FloatLayout {
        let gl: Self::GL = downgrade_to(gl);
        FloatLayout::Float(ColorComponents::RG(downgrade_to(&gl)), C::ty(&gl))
    }
}

impl_arr_data!([C; 1] IntLayout::Integer(IntColorComponents::RED_INTEGER, IntType));
impl_arr_data!([C; 1] FloatLayout::Float(ColorComponents::RED, FloatType));
impl_arr_data!([C; 1] IntType StencilLayout);
impl_arr_data!([C; 1] FloatType DepthLayout);
// impl_arr_data!([C; 1] FloatType DepthStencilLayout);

macro_rules! impl_prim_data {

    ($layout:ident for $p:ident with $ty:ident) => {
        unsafe impl UncompressedPixelData<$layout> for $p {
            type GL = ();
            fn layout<GL:Supports<()>>(gl: &GL) -> $layout {
                <Self as SubPixelData<$ty>>::ty(gl).into()
            }
        }
    };

    (@int $($p:ident $gl:ty;)*) => {
        $(
            impl_prim_data!(IntLayout for $p with IntType);
            impl_prim_data!(FloatLayout for $p with IntType);
            impl_prim_data!(StencilLayout for $p with IntType);
            impl_prim_data!(DepthLayout for $p with IntType);
        )*
    };

    (@float $($p:ident $gl:ty;)*) => {
        $(
            impl_prim_data!(FloatLayout for $p with FloatType);
            impl_prim_data!(DepthLayout for $p with FloatType);
            impl_prim_data!(DepthStencilLayout for $p with FloatType);
        )*
    }

}

impl_prim_data!(@int u8(); i8(); u16(); i16(); u32(); i32(););
impl_prim_data!(@float f32(););
