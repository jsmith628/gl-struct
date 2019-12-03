use crate::*;
use super::*;
use crate::context::*;

pub unsafe trait InternalFormat {
    type ClientFormat: ClientFormat;
    type GL: GLVersion;
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

pub unsafe trait Compressed: InternalFormat {}
pub unsafe trait SpecificCompressed: Compressed {
    type Block: Copy;
    fn block_width() -> u8;
    fn block_height() -> u8;
    fn block_depth() -> u8;
    #[inline] fn block_size() -> usize { ::std::mem::size_of::<Self::Block>() }
}

pub unsafe trait InternalFormatColor: InternalFormat {}
pub unsafe trait Renderable: InternalFormat {}
pub unsafe trait ReqRenderBuffer: Renderable {}

pub unsafe trait InternalFormatFloat: InternalFormat<ClientFormat = ClientFormatFloat> + InternalFormatColor {}
pub unsafe trait InternalFormatInt: InternalFormat<ClientFormat = ClientFormatInt> + InternalFormatColor {}
pub unsafe trait InternalFormatUInt: InternalFormat<ClientFormat = ClientFormatInt> + InternalFormatColor {}
pub unsafe trait InternalFormatDepth: InternalFormat<ClientFormat = ClientFormatDepth> {}
pub unsafe trait InternalFormatStencil: InternalFormat<ClientFormat = ClientFormatStencil> {}
pub unsafe trait InternalFormatDepthStencil: InternalFormat<ClientFormat = ClientFormatDepthStencil> {}

pub unsafe trait InternalFormatRed: InternalFormatColor {}
pub unsafe trait InternalFormatRG: InternalFormatColor {}
pub unsafe trait InternalFormatRGB: InternalFormatColor {}
pub unsafe trait InternalFormatRGBA: InternalFormatColor {}

pub unsafe trait ViewCompatible<F:InternalFormat>: InternalFormat {}
pub unsafe trait ImageCompatible<F:SizedInternalFormat>: SizedInternalFormat {}

unsafe impl<F:InternalFormat> ViewCompatible<F> for F {}
unsafe impl<F:SizedInternalFormat> ImageCompatible<F> for F {}

macro_rules! internal_format {

    (@fmt_ty InternalFormatFloat) => {ClientFormatFloat};
    (@fmt_ty InternalFormatInt) => {ClientFormatInt};
    (@fmt_ty InternalFormatUInt) => {ClientFormatInt};
    (@fmt_ty InternalFormatDepth) => {ClientFormatDepth};
    (@fmt_ty InternalFormatStencil) => {ClientFormatStencil};
    (@fmt_ty InternalFormatDepthStencil) => {ClientFormatDepthStencil};

    (@sized $fmt:ident ($D:literal)) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn depth_bits() -> u8 {$D}
        }
    };

    (@sized $fmt:ident ($D:literal, $S:literal)) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn depth_bits() -> u8 {$D}
            #[inline] fn stencil_bits() -> u8 {$S}
        }
    };

    (@sized $fmt:ident [$block:ty; $w:literal, $h:literal, $d:literal]) => {
        unsafe impl InternalFormatColor for $fmt {}
        unsafe impl Compressed for $fmt {}
        unsafe impl SpecificCompressed for $fmt {
            type Block = $block;
            fn block_width() -> u8 {$w}
            fn block_height() -> u8 {$h}
            fn block_depth() -> u8 {$d}
        }
    };

    (@sized $fmt:ident [$R:literal]) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
        }
        unsafe impl InternalFormatColor for $fmt {}
        unsafe impl InternalFormatRed for $fmt {}
    };

    (@sized $fmt:ident [$R:literal, $G:literal]) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
            #[inline] fn green_bits() -> u8 {$G}
        }
        unsafe impl InternalFormatColor for $fmt {}
        unsafe impl InternalFormatRG for $fmt {}
    };

    (@sized $fmt:ident [$R:literal, $G:literal, $B:literal]) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
            #[inline] fn green_bits() -> u8 {$G}
            #[inline] fn blue_bits() -> u8 {$B}
        }
        unsafe impl InternalFormatColor for $fmt {}
        unsafe impl InternalFormatRGB for $fmt {}
    };

    (@sized $fmt:ident [$R:literal, $G:literal, $B:literal, $A:literal]) => {
        unsafe impl SizedInternalFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
            #[inline] fn green_bits() -> u8 {$G}
            #[inline] fn blue_bits() -> u8 {$B}
            #[inline] fn alpha_bits() -> u8 {$A}
        }
        unsafe impl InternalFormatColor for $fmt {}
        unsafe impl InternalFormatRGBA for $fmt {}
    };

    (@$kind:ident $fmt:ident: cmpr + $GL:tt, $($tt:tt)*) => {
        unsafe impl InternalFormatColor for $fmt {}
        unsafe impl Compressed for $fmt {}
        internal_format!(@$kind $fmt: $GL, $($tt)*);
    };

    (@$kind:ident $fmt:ident: req_rend + $GL:tt, $($tt:tt)*) => {
        unsafe impl Renderable for $fmt {}
        unsafe impl ReqRenderBuffer for $fmt {}
        internal_format!(@$kind $fmt: $GL, $($tt)*);
    };

    (@$kind:ident $fmt:ident $sizes:tt: cr + $GL:tt, $($tt:tt)*) => {
        unsafe impl Renderable for $fmt {}
        internal_format!(@$kind $fmt $sizes: $GL, $($tt)*);
    };

    (@$kind:ident $fmt:ident $sizes:tt: req_rend + $GL:tt, $($tt:tt)*) => {
        unsafe impl Renderable for $fmt {}
        unsafe impl ReqRenderBuffer for $fmt {}
        internal_format!(@$kind $fmt $sizes: $GL, $($tt)*);
    };

    (@$kind:ident $fmt:ident $sizes:tt: $GL:tt, $($tt:tt)*) => {
        internal_format!(@$kind $fmt: $GL, $($tt)*);
        internal_format!(@sized $fmt $sizes);
    };

    (@$kind:ident $fmt:ident: $GL:tt, $($tt:tt)*) => {

        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
        pub struct $fmt;

        unsafe impl InternalFormat for $fmt {
            type ClientFormat = internal_format!(@fmt_ty $kind);
            type GL = $GL;
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
        RED: req_rend + GL30,
        RG: req_rend + GL30,
        RGB: req_rend + GL11,
        RGBA: req_rend + GL11,
        COMPRESSED_RED: cmpr + GL30,
        COMPRESSED_RG: cmpr + GL30,
        COMPRESSED_RGB: cmpr + GL13,
        COMPRESSED_RGBA: cmpr + GL13,
        COMPRESSED_SRGB: cmpr + GL21,
        COMPRESSED_SRGB_ALPHA: cmpr + GL21,

        //
        //fixed-point (normalized integer)
        //

        //Red
        R8[8]: req_rend + GL30,
        R8_SNORM[8]: cr + GL31,
        R16[16]: req_rend + GL30,
        R16_SNORM[16]: cr + GL31,

        //RG
        RG8[8,8]: req_rend + GL30,
        RG8_SNORM[8,8]: cr + GL31,
        RG16[16,16]: req_rend + GL30,
        RG16_SNORM[16,16]: cr + GL31,

        //RGB
        R3_G3_B2[3,3,2]: cr + GL11,
        RGB4[4,4,4]: cr + GL11,
        RGB5[4,4,4]: cr + GL11,
        RGB565[5,6,5]: req_rend + GL42,
        RGB8[8,8,8]: cr + GL11,
        RGB8_SNORM[8,8,8]: cr + GL31,
        RGB10[10,10,10]: cr + GL11,
        RGB12[12,12,12]: cr + GL11,
        RGB16[16,16,16]: cr + GL11,
        RGB16_SNORM[16,16,16]: cr + GL31,

        //RGBA
        RGBA2[2,2,2,2]: cr + GL11,
        RGBA4[4,4,4,4]: req_rend + GL11,
        RGB5_A1[5,5,5,1]: req_rend + GL11,
        RGBA8[8,8,8,8]: req_rend + GL11,
        RGBA8_SNORM[8,8,8,8]: cr + GL31,
        RGB10_A2[10,10,10,2]: req_rend + GL11,
        RGBA12[12,12,12,12]: cr + GL11,
        RGBA16[16,16,16,16]: req_rend + GL11,
        RGBA16_SNORM[16,16,16,16]: cr + GL31,
        RGB9_E5: GL30,

        //sRGB
        SRGB8[8,8,8]: cr + GL21,
        SRGB8_ALPHA8[8,8,8,8]: req_rend + GL21,

        //
        //floating point
        //

        //half-precision float
        R16F[16]: req_rend + GL30,
        RG16F[16,16]: req_rend + GL30,
        RGB16F[16,16,16]: cr + GL30,
        RGBA16F[16,16,16,16]: req_rend + GL30, //Half-float

        //single-precision float
        R32F[32]: req_rend + GL30,
        RG32F[32,32]: req_rend + GL30,
        RGB32F[32,32,32]: cr + GL30,
        RGBA32F[32,32,32,32]: req_rend + GL30,

        //weird-ass float
        R11F_G11F_B10F[11,11,10]: req_rend + GL30,

        //
        //compressed
        //

        //Red-green Texture Compression
        COMPRESSED_RED_RGTC1[u64; 4, 4, 1]: GL30,
        COMPRESSED_SIGNED_RED_RGTC1[u64; 4, 4, 1]: GL30,
        COMPRESSED_RG_RGTC2[[u64;2]; 4, 4, 1]: GL30,
        COMPRESSED_SIGNED_RG_RGTC2[[u64;2]; 4, 4, 1]: GL30,

        //BPTC
        COMPRESSED_RGBA_BPTC_UNORM[u128; 4, 4, 1]: GL42,
        COMPRESSED_SRGB_ALPHA_BPTC_UNORM[u128; 4, 4, 1]: GL42,
        COMPRESSED_RGB_BPTC_SIGNED_FLOAT[u128; 4, 4, 1]: GL42,
        COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT[u128; 4, 4, 1]: GL42,

        //Ericsson Texture Compression
        COMPRESSED_RGB8_ETC2[u64; 4, 4, 1]: GL43,
        COMPRESSED_SRGB8_ETC2[u64; 4, 4, 1]: GL43,
        COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2[u64; 4, 4, 1]: GL43,
        COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2[u64; 4, 4, 1]: GL43,
        COMPRESSED_RGBA8_ETC2_EAC[u128; 4, 4, 1]: GL43,
        COMPRESSED_SRGB8_ALPHA8_ETC2_EAC[u128; 4, 4, 1]: GL43,
        COMPRESSED_R11_EAC[u64; 4, 4, 1]: GL43,
        COMPRESSED_SIGNED_R11_EAC[u64; 4, 4, 1]: GL43,
        COMPRESSED_RG11_EAC[[u64;2]; 4, 4, 1]: GL43,
        COMPRESSED_SIGNED_RG11_EAC[[u64;2]; 4, 4, 1]: GL43,
    }

    pub enum InternalFormatInt {
        //1-component
        R8I[8]: req_rend + GL30,
        R16I[16]: req_rend + GL30,
        R32I[32]: req_rend + GL30,

        //2-component
        RG8I[8,8]: req_rend + GL30,
        RG16I[16,16]: req_rend + GL30,
        RG32I[32,32]: req_rend + GL30,

        //3-component
        RGB8I[8,8,8]: cr + GL30,
        RGB16I[16,16,16]: cr + GL30,
        RGB32I[32,32,32]: cr + GL30,

        //4-component
        RGBA8I[8,8,8,8]: req_rend + GL30,
        RGBA16I[16,16,16,16]: req_rend + GL30,
        RGBA32I[32,32,32,32]: req_rend + GL30,
    }

    pub enum InternalFormatUInt {
        //1-component
        R8UI[8]: req_rend + GL30,
        R16UI[16]: req_rend + GL30,
        R32UI[32]: req_rend + GL30,

        //2-component
        RG8UI[8,8]: req_rend + GL30,
        RG16UI[16,16]: req_rend + GL30,
        RG32UI[32,32]: req_rend + GL30,

        //3-component
        RGB8UI[8,8,8]: cr + GL30,
        RGB16UI[16,16,16]: cr + GL30,
        RGB32UI[32,32,32]: cr + GL30,

        //4-component
        RGBA8UI[8,8,8,8]: req_rend + GL30,
        RGBA16UI[16,16,16,16]: req_rend + GL30,
        RGBA32UI[32,32,32,32]: req_rend + GL30,

        //Weird shit
        RGB10_A2UI[10,10,10,2]: req_rend + GL33,
    }

    pub enum InternalFormatDepth {
        DEPTH_COMPONENT: req_rend + GL14, //base internal format
        DEPTH_COMPONENT16(16): req_rend + GL14,
        DEPTH_COMPONENT24(24): req_rend + GL14,
        DEPTH_COMPONENT32(32): cr + !,
        DEPTH_COMPONENT32F(32): req_rend + GL30,
    }

    pub enum InternalFormatStencil {
        STENCIL_INDEX: req_rend + GL44, //base internal format
        STENCIL_INDEX1(0,1): cr + !,
        STENCIL_INDEX4(0,4): cr + !,
        STENCIL_INDEX8(0,8): req_rend + GL44,
        STENCIL_INDEX16(0,16): cr + !,
    }

    pub enum InternalFormatDepthStencil {
        DEPTH_STENCIL: req_rend + GL30, //base internal format
        DEPTH24_STENCIL8(24,8): req_rend + GL30,
        DEPTH32F_STENCIL8(32,8): req_rend + GL30,
    }
}


unsafe impl InternalFormatColor for RGB9_E5 {}
unsafe impl InternalFormatRGB for RGB9_E5 {}
unsafe impl SizedInternalFormat for RGB9_E5 {
    #[inline] fn red_bits() -> u8 {9}
    #[inline] fn green_bits() -> u8 {9}
    #[inline] fn blue_bits() -> u8 {9}
    #[inline] fn shared_bits() -> u8 {5}
}

unsafe impl InternalFormatColor for RED {}
unsafe impl InternalFormatRed for RED {}
unsafe impl InternalFormatColor for RG {}
unsafe impl InternalFormatRG for RG {}
unsafe impl InternalFormatColor for RGB {}
unsafe impl InternalFormatRGB for RGB {}
unsafe impl InternalFormatColor for RGBA {}
unsafe impl InternalFormatRGBA for RGBA {}

unsafe impl InternalFormatRed for COMPRESSED_RED {}
unsafe impl InternalFormatRG for COMPRESSED_RG {}
unsafe impl InternalFormatRGB for COMPRESSED_RGB {}
unsafe impl InternalFormatRGBA for COMPRESSED_RGBA {}

unsafe impl InternalFormatRed for COMPRESSED_RED_RGTC1 {}
unsafe impl InternalFormatRed for COMPRESSED_SIGNED_RED_RGTC1 {}
unsafe impl InternalFormatRG for COMPRESSED_RG_RGTC2 {}
unsafe impl InternalFormatRG for COMPRESSED_SIGNED_RG_RGTC2 {}
unsafe impl InternalFormatRGBA for COMPRESSED_RGBA_BPTC_UNORM {}
unsafe impl InternalFormatRGB for COMPRESSED_RGB_BPTC_SIGNED_FLOAT {}
unsafe impl InternalFormatRGB for COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT {}
unsafe impl InternalFormatRGB for COMPRESSED_RGB8_ETC2 {}
unsafe impl InternalFormatRGB for COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2 {}
unsafe impl InternalFormatRGBA for COMPRESSED_RGBA8_ETC2_EAC {}
unsafe impl InternalFormatRed for COMPRESSED_R11_EAC {}
unsafe impl InternalFormatRed for COMPRESSED_SIGNED_R11_EAC {}
unsafe impl InternalFormatRG for COMPRESSED_RG11_EAC {}
unsafe impl InternalFormatRG for COMPRESSED_SIGNED_RG11_EAC {}

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
