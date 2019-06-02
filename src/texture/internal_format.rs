use super::*;

pub unsafe trait InternalFormat {
    type FormatType: PixelFormatType;
    fn glenum() -> GLenum;
}

pub unsafe trait SizedInternalFormat: InternalFormat {
    #[inline] fn bits() -> u16 {
        Self::red_bits() as u16 + Self::green_bits() as u16 + Self::blue_bits() as u16 +
        Self::alpha_bits() as u16 + Self::depth_bits() as u16 + Self::stencil_bits() as u16 +
        Self::shared_bits() as u16
    }

    #[inline] fn red_bits() -> u8 {0}
    #[inline] fn green_bits() -> u8 {0}
    #[inline] fn blue_bits() -> u8 {0}
    #[inline] fn alpha_bits() -> u8 {0}
    #[inline] fn depth_bits() -> u8 {0}
    #[inline] fn stencil_bits() -> u8 {0}
    #[inline] fn shared_bits() -> u8 {0}
}

pub unsafe trait CompressedInternalFormat: InternalFormat {}

pub unsafe trait InternalFormatFloat: InternalFormat<FormatType = FloatFormatType> {}
pub unsafe trait InternalFormatInt: InternalFormat<FormatType = IntFormatType> {}
pub unsafe trait InternalFormatUInt: InternalFormat<FormatType = IntFormatType> {}
pub unsafe trait InternalFormatDepth: InternalFormat<FormatType = DepthFormatType> {}
pub unsafe trait InternalFormatStencil: InternalFormat<FormatType = StencilFormatType> {}
pub unsafe trait InternalFormatDepthStencil: InternalFormat<FormatType = DepthStencilFormatType> {}

pub unsafe trait InternalFormatRed: InternalFormat {}
pub unsafe trait InternalFormatRG: InternalFormat {}
pub unsafe trait InternalFormatRGB: InternalFormat {}
pub unsafe trait InternalFormatRGBA: InternalFormat {}

pub unsafe trait ViewCompatible<F:InternalFormat>: InternalFormat {}
pub unsafe trait ImageCompatible<F:SizedInternalFormat>: SizedInternalFormat {}

unsafe impl<F:InternalFormat> ViewCompatible<F> for F {}
unsafe impl<F:SizedInternalFormat> ImageCompatible<F> for F {}

macro_rules! internal_format {

    (@fmt_ty InternalFormatFloat) => {FloatFormatType};
    (@fmt_ty InternalFormatInt) => {IntFormatType};
    (@fmt_ty InternalFormatUInt) => {IntFormatType};
    (@fmt_ty InternalFormatDepth) => {DepthFormatType};
    (@fmt_ty InternalFormatStencil) => {StencilFormatType};
    (@fmt_ty InternalFormatDepthStencil) => {DepthStencilFormatType};

    (@sized $fmt:ident ($D:tt)) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn depth_bits() -> u8 {$D}
        }
    };

    (@sized $fmt:ident ($D:tt, $S:tt)) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn depth_bits() -> u8 {$D}
            #[inline] fn stencil_bits() -> u8 {$S}
        }
    };

    (@sized $fmt:ident [$R:tt, $G:tt]) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
            #[inline] fn green_bits() -> u8 {$G}
        }
        unsafe impl InternalFormatRG for $fmt {}
    };

    (@sized $fmt:ident [$R:tt]) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
        }
        unsafe impl InternalFormatRed for $fmt {}
    };

    (@sized $fmt:ident [$R:tt, $G:tt]) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
            #[inline] fn green_bits() -> u8 {$G}
        }
        unsafe impl InternalFormatRG for $fmt {}
    };

    (@sized $fmt:ident [$R:tt, $G:tt, $B:tt]) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
            #[inline] fn green_bits() -> u8 {$G}
            #[inline] fn blue_bits() -> u8 {$B}
        }
        unsafe impl InternalFormatRGB for $fmt {}
    };

    (@sized $fmt:ident [$R:tt, $G:tt, $B:tt, $A:tt]) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
            #[inline] fn green_bits() -> u8 {$G}
            #[inline] fn blue_bits() -> u8 {$B}
            #[inline] fn alpha_bits() -> u8 {$A}
        }
        unsafe impl InternalFormatRGBA for $fmt {}
    };

    (@$kind:ident $fmt:ident cmpr, $($tt:tt)*) => {
        internal_format!(@$kind $fmt,);
        unsafe impl CompressedInternalFormat for $fmt {}
        internal_format!(@$kind $($tt)*);
    };

    (@$kind:ident $fmt:ident $sizes:tt, $($tt:tt)*) => {
        internal_format!(@$kind $fmt,);
        internal_format!(@sized $fmt $sizes);
        internal_format!(@$kind $($tt)*);
    };

    (@$kind:ident $fmt:ident, $($tt:tt)*) => {

        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
        pub struct $fmt;

        unsafe impl InternalFormat for $fmt {
            type FormatType = internal_format!(@fmt_ty $kind);
            #[inline] fn glenum() -> GLenum {gl::$fmt}
        }

        unsafe impl $kind for $fmt {}

        internal_format!(@$kind $($tt)*);

    };

    (@$kind:ident) => {};

    ($( pub enum $kind:ident {$($items:tt)*} )*) => {
        $(internal_format!(@$kind $($items)*);)*
    };

}

internal_format! {
    pub enum InternalFormatFloat {
        //Base Internal Formats (ie let the implementation decide the specifics)
        RED, RG, RGB, RGBA, //uncompressed
        COMPRESSED_RED cmpr, COMPRESSED_RG cmpr, COMPRESSED_RGB cmpr, COMPRESSED_RGBA cmpr, //compressed
        COMPRESSED_SRGB cmpr, COMPRESSED_SRGB_ALPHA cmpr, //compressed sRGB

        //
        //fixed-point (normalized integer)
        //

        //Red
        R8[8],R8_SNORM[8],
        R16[16],R16_SNORM[16],

        //RG
        RG8[8,8],RG8_SNORM[8,8],
        RG16[16,16],RG16_SNORM[16,16],

        //RGB
        R3_G3_B2[3,3,2],
        RGB4[4,4,4], RGB5[4,4,4], RGB565[5,6,5],
        RGB8[8,8,8],RGB8_SNORM[8,8,8],
        RGB10[10,10,10],RGB12[12,12,12],
        RGB16[16,16,16],RGB16_SNORM[16,16,16],

        //RGBA
        RGBA2[2,2,2,2],
        RGBA4[4,4,4,4],
        RGB5_A1[5,5,5,1],
        RGBA8[8,8,8,8],RGBA8_SNORM[8,8,8,8],
        RGB10_A2[10,10,10,2], RGBA12[12,12,12,12],
        RGBA16[16,16,16,16],RGBA16_SNORM[16,16,16,16],
        RGB9_E5,

        //sRGB
        SRGB8[8,8,8],SRGB8_ALPHA8[8,8,8,8],

        //
        //floating point
        //

        R16F[16], RG16F[16,16], RGB16F[16,16,16], RGBA16F[16,16,16,16], //Half-float
        R32F[32], RG32F[32,32], RGB32F[32,32,32], RGBA32F[32,32,32,32], //Single-float
        R11F_G11F_B10F[11,11,10], //Weird-ass-float

        //
        //compressed
        //

        COMPRESSED_RED_RGTC1 cmpr, COMPRESSED_SIGNED_RED_RGTC1 cmpr,
        COMPRESSED_RG_RGTC2 cmpr, COMPRESSED_SIGNED_RG_RGTC2 cmpr,
        COMPRESSED_RGBA_BPTC_UNORM cmpr, COMPRESSED_SRGB_ALPHA_BPTC_UNORM cmpr,
        COMPRESSED_RGB_BPTC_SIGNED_FLOAT cmpr, COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT cmpr,
        COMPRESSED_RGB8_ETC2 cmpr, COMPRESSED_SRGB8_ETC2 cmpr,
        COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2 cmpr, COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2 cmpr,
        COMPRESSED_RGBA8_ETC2_EAC cmpr, COMPRESSED_SRGB8_ALPHA8_ETC2_EAC cmpr,
        COMPRESSED_R11_EAC cmpr, COMPRESSED_SIGNED_R11_EAC cmpr, COMPRESSED_RG11_EAC cmpr, COMPRESSED_SIGNED_RG11_EAC cmpr,
    }

    pub enum InternalFormatInt {
        R8I[8], R16I[16], R32I[32], //1-component
        RG8I[8,8], RG16I[16,16], RG32I[32,32], //2-component
        RGB8I[8,8,8], RGB16I[16,16,16], RGB32I[32,32,32], //3-component
        RGBA8I[8,8,8,8], RGBA16I[16,16,16,16], RGBA32I[32,32,32,32], //4-component
    }

    pub enum InternalFormatUInt {
        R8UI[8], R16UI[16], R32UI[32],  //1-component
        RG8UI[8,8], RG16UI[16,16], RG32UI[32,32],  //2-component
        RGB8UI[8,8,8], RGB16UI[16,16,16], RGB32UI[32,32,32],  //3-component
        RGBA8UI[8,8,8,8], RGBA16UI[16,16,16,16], RGBA32UI[32,32,32,32],  //4-component
        RGB10_A2UI[10,10,10,2],  //Weird shit
    }

    pub enum InternalFormatDepth {
        DEPTH_COMPONENT, //base internal format
        DEPTH_COMPONENT16(16),
        DEPTH_COMPONENT24(24),
        DEPTH_COMPONENT32(32),
        DEPTH_COMPONENT32F(32),
    }

    pub enum InternalFormatStencil {
        STENCIL_INDEX(0, 32), //base internal format
        STENCIL_INDEX1(0,1),
        STENCIL_INDEX4(0,4),
        STENCIL_INDEX8(0,8),
        STENCIL_INDEX16(0,16),
    }

    pub enum InternalFormatDepthStencil {
        DEPTH_STENCIL, //base internal format
        DEPTH24_STENCIL8(24,8),
        DEPTH32F_STENCIL8(32,8),
    }
}


unsafe impl SizedInternalFormat for RGB9_E5 {
    #[inline] fn red_bits() -> u8 {9}
    #[inline] fn green_bits() -> u8 {9}
    #[inline] fn blue_bits() -> u8 {9}
    #[inline] fn shared_bits() -> u8 {5}
}

macro_rules! compat_with {
    ($ty0:ident $($ty:ident)*; $trait:ident) => {
        $(
            unsafe impl $trait<$ty> for $ty0{}
            unsafe impl $trait<$ty0> for $ty{}
        )*
        compat_with!($($ty)*; $trait);
    };

    (;$trait:ident) => {};
}

compat_with!(RGBA32F RGBA32UI RGBA32I; ViewCompatible);
compat_with!(RGB32F RGB32UI RGB32I; ViewCompatible);
compat_with!(RGBA16F RG32F RGBA16UI RG32UI RGBA16I RG32I RGBA16 RGBA16_SNORM; ViewCompatible);
compat_with!(RGB16 RGB16_SNORM RGB16F RGB16UI RGB16I; ViewCompatible);
compat_with!(
    RG16F R11F_G11F_B10F R32F RGB10_A2UI RGBA8UI RG16UI R32UI RGBA8I RG16I R32I RGB10_A2
    RGBA8 RG16 RGBA8_SNORM RG16_SNORM SRGB8_ALPHA8 RGB9_E5; ViewCompatible
);
compat_with!(RGB8 RGB8_SNORM SRGB8 RGB8UI RGB8I; ViewCompatible);
compat_with!(R16F RG8UI R16UI RG8I R16I RG8 R16 RG8_SNORM R16_SNORM; ViewCompatible);
compat_with!(R8UI R8I R8 R8_SNORM; ViewCompatible);
compat_with!(COMPRESSED_RED_RGTC1 COMPRESSED_SIGNED_RED_RGTC1; ViewCompatible);
compat_with!(COMPRESSED_RG_RGTC2 COMPRESSED_SIGNED_RG_RGTC2; ViewCompatible);
compat_with!(COMPRESSED_RGBA_BPTC_UNORM COMPRESSED_SRGB_ALPHA_BPTC_UNORM; ViewCompatible);
compat_with!(COMPRESSED_RGB_BPTC_SIGNED_FLOAT COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT; ViewCompatible);
