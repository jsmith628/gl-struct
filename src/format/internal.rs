use crate::*;
use super::*;
use crate::version::*;

pub unsafe trait InternalFormat {
    type PixelLayout: PixelLayout;
    type GL: GLVersion;
    fn glenum() -> GLenum;
}

#[marker] pub unsafe trait ColorFormat: InternalFormat {}
pub unsafe trait SizedPixelFormat: SizedFormat {
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

#[marker] pub unsafe trait CompressedFormat: InternalFormat {}
pub unsafe trait SpecificCompressed: CompressedFormat + SizedFormat {
    type Block: Copy;
    fn block_width() -> u8;
    fn block_height() -> u8;
    fn block_depth() -> u8;
    #[inline] fn block_size() -> usize { ::std::mem::size_of::<Self::Block>() }
}

#[marker] pub unsafe trait SizedFormat: InternalFormat {}
unsafe impl<F:SizedPixelFormat> SizedFormat for F {}
unsafe impl<F:SpecificCompressed> SizedFormat for F {}

#[marker] pub unsafe trait Renderable: InternalFormat {}
#[marker] pub unsafe trait ReqRenderBuffer: Renderable {}

#[marker] pub unsafe trait RedFormat: ColorFormat {}
#[marker] pub unsafe trait RGFormat: ColorFormat {}
#[marker] pub unsafe trait RGBFormat: ColorFormat {}
#[marker] pub unsafe trait RGBAFormat: ColorFormat {}

#[marker] pub unsafe trait SRGBFormat: ColorFormat {}

#[marker] pub unsafe trait SignedFormat: InternalFormat {}
#[marker] pub unsafe trait UnsignedFormat: InternalFormat {}

pub trait FloatFormat = InternalFormat<PixelLayout = FloatLayout> + ColorFormat;
pub trait IntFormat = InternalFormat<PixelLayout = IntLayout> + ColorFormat + SignedFormat;
pub trait UIntFormat = InternalFormat<PixelLayout = IntLayout> + ColorFormat + UnsignedFormat;
pub trait DepthFormat = InternalFormat<PixelLayout = DepthLayout>;
pub trait StencilFormat = InternalFormat<PixelLayout = StencilLayout>;
pub trait DepthStencilFormat = InternalFormat<PixelLayout = DepthStencilLayout>;

pub unsafe trait BufferTextureFormat: ColorFormat {
    type Pixel: Pixel<Self::PixelLayout>;
}

#[marker] pub unsafe trait ViewCompatible<F:SizedFormat>: SizedFormat {}
unsafe impl<F:SizedFormat> ViewCompatible<F> for F {}

#[marker] pub unsafe trait ImageLoadStore: SizedPixelFormat {}
pub trait ImageCompatible<F:SizedPixelFormat> = ViewCompatible<F> + ImageLoadStore;


macro_rules! internal_format {

    (@fmt_ty FloatFormat) => {FloatLayout};
    (@fmt_ty IntFormat) => {IntLayout};
    (@fmt_ty UIntFormat) => {IntLayout};
    (@fmt_ty DepthFormat) => {DepthLayout};
    (@fmt_ty StencilFormat) => {StencilLayout};
    (@fmt_ty DepthStencilFormat) => {DepthStencilLayout};

    (@sized $fmt:ident ($D:literal)) => {
        unsafe impl SizedPixelFormat for $fmt {
            #[inline] fn depth_bits() -> u8 {$D}
        }
    };

    (@sized $fmt:ident ($D:literal, $S:literal)) => {
        unsafe impl SizedPixelFormat for $fmt {
            #[inline] fn depth_bits() -> u8 {$D}
            #[inline] fn stencil_bits() -> u8 {$S}
        }
    };

    (@sized $fmt:ident [$block:ty; $w:literal, $h:literal, $d:literal]) => {
        unsafe impl ColorFormat for $fmt {}
        unsafe impl CompressedFormat for $fmt {}
        unsafe impl SpecificCompressed for $fmt {
            type Block = $block;
            fn block_width() -> u8 {$w}
            fn block_height() -> u8 {$h}
            fn block_depth() -> u8 {$d}
        }
    };

    (@sized $fmt:ident [$R:literal]) => {
        unsafe impl SizedPixelFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
        }
        unsafe impl ColorFormat for $fmt {}
        unsafe impl RedFormat for $fmt {}
    };

    (@sized $fmt:ident [$R:literal, $G:literal]) => {
        unsafe impl SizedPixelFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
            #[inline] fn green_bits() -> u8 {$G}
        }
        unsafe impl ColorFormat for $fmt {}
        unsafe impl RGFormat for $fmt {}
    };

    (@sized $fmt:ident [$R:literal, $G:literal, $B:literal]) => {
        unsafe impl SizedPixelFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
            #[inline] fn green_bits() -> u8 {$G}
            #[inline] fn blue_bits() -> u8 {$B}
        }
        unsafe impl ColorFormat for $fmt {}
        unsafe impl RGBFormat for $fmt {}
    };

    (@sized $fmt:ident [$R:literal, $G:literal, $B:literal, $A:literal]) => {
        unsafe impl SizedPixelFormat for $fmt {
            #[inline] fn red_bits() -> u8 {$R}
            #[inline] fn green_bits() -> u8 {$G}
            #[inline] fn blue_bits() -> u8 {$B}
            #[inline] fn alpha_bits() -> u8 {$A}
        }
        unsafe impl ColorFormat for $fmt {}
        unsafe impl RGBAFormat for $fmt {}
    };

    (@$kind:ident $fmt:ident: img + $($tt:tt)*) => {
        unsafe impl ImageLoadStore for $fmt {}
        internal_format!(@$kind $fmt: $($tt)*);
    };

    (@$kind:ident $fmt:ident: usign + $($tt:tt)*) => {
        unsafe impl UnsignedFormat for $fmt {}
        internal_format!(@$kind $fmt: $($tt)*);
    };

    (@$kind:ident $fmt:ident: sign + $($tt:tt)*) => {
        unsafe impl SignedFormat for $fmt {}
        internal_format!(@$kind $fmt: $($tt)*);
    };

    (@$kind:ident $fmt:ident: srgb + $($tt:tt)*) => {
        unsafe impl SRGBFormat for $fmt {}
        internal_format!(@$kind $fmt: $($tt)*);
    };

    (@$kind:ident $fmt:ident: cmpr + $($tt:tt)*) => {
        unsafe impl ColorFormat for $fmt {}
        unsafe impl CompressedFormat for $fmt {}
        internal_format!(@$kind $fmt: $($tt)*);
    };

    (@$kind:ident $fmt:ident: cr + $($tt:tt)*) => {
        unsafe impl Renderable for $fmt {}
        internal_format!(@$kind $fmt: $($tt)*);
    };

    (@$kind:ident $fmt:ident: req_rend + $($tt:tt)*) => {
        unsafe impl Renderable for $fmt {}
        unsafe impl ReqRenderBuffer for $fmt {}
        internal_format!(@$kind $fmt: $($tt)*);
    };

    (@$kind:ident $fmt:ident ($($sizes:tt)*): $($tt:tt)*) => {
        internal_format!(@sized $fmt ($($sizes)*));
        internal_format!(@$kind $fmt: $($tt)*);
    };

    (@$kind:ident $fmt:ident [$($sizes:tt)*]: $($tt:tt)*) => {
        internal_format!(@sized $fmt [$($sizes)*]);
        internal_format!(@$kind $fmt: $($tt)*);
    };

    (@$kind:ident $fmt:ident: $GL:tt, $($tt:tt)*) => {

        #[allow(non_camel_case_types)]
        pub struct $fmt(!);

        unsafe impl InternalFormat for $fmt {
            type PixelLayout = internal_format!(@fmt_ty $kind);
            type GL = $GL;
            #[inline] fn glenum() -> GLenum {gl::$fmt}
        }

        internal_format!(@$kind $($tt)*);

    };

    (@$kind:ident) => {};

    ($( pub enum $kind:ident {$($items:tt)*} )*) => {
        $(internal_format!(@$kind $($items)*);)*
    };

}

internal_format! {
    pub enum FloatFormat {
        //Base Internal Formats (ie let the implementation decide the specifics)
        RED: req_rend + usign + GL30,
        RG: req_rend + usign + GL30,
        RGB: req_rend + usign + GL11,
        RGBA: req_rend + usign + GL11,
        COMPRESSED_RED: cmpr + usign + GL30,
        COMPRESSED_RG: cmpr + usign + GL30,
        COMPRESSED_RGB: cmpr + usign + GL13,
        COMPRESSED_RGBA: cmpr + usign + GL13,
        COMPRESSED_SRGB: srgb + cmpr + usign + GL21,
        COMPRESSED_SRGB_ALPHA: srgb + cmpr + usign + GL21,

        //
        //fixed-point (normalized integer)
        //

        //Red
        R8[8]: img + req_rend + usign + GL30,
        R8_SNORM[8]: img + cr + sign + GL31,
        R16[16]: img + req_rend + usign + GL30,
        R16_SNORM[16]: img + cr + sign + GL31,

        //RG
        RG8[8,8]: img + req_rend + usign + GL30,
        RG8_SNORM[8,8]: img + cr + sign + GL31,
        RG16[16,16]: img + req_rend + usign + GL30,
        RG16_SNORM[16,16]: img + cr + sign + GL31,

        //RGB
        R3_G3_B2[3,3,2]: cr + usign + GL11,
        RGB4[4,4,4]: cr + usign + GL11,
        RGB5[4,4,4]: cr + usign + GL11,
        RGB565[5,6,5]: req_rend + usign + GL42,
        RGB8[8,8,8]: cr + usign + GL11,
        RGB8_SNORM[8,8,8]: cr + sign + GL31,
        RGB10[10,10,10]: cr + usign + GL11,
        RGB12[12,12,12]: cr + usign + GL11,
        RGB16[16,16,16]: cr + usign + GL11,
        RGB16_SNORM[16,16,16]: cr + sign + GL31,

        //RGBA
        RGBA2[2,2,2,2]: cr + usign + GL11,
        RGBA4[4,4,4,4]: req_rend + usign + GL11,
        RGB5_A1[5,5,5,1]: req_rend + usign + GL11,
        RGBA8[8,8,8,8]: img + req_rend + usign + GL11,
        RGBA8_SNORM[8,8,8,8]: img + cr + sign + GL31,
        RGB10_A2[10,10,10,2]: img + req_rend + usign + GL11,
        RGBA12[12,12,12,12]: cr + usign + GL11,
        RGBA16[16,16,16,16]: img + req_rend + usign + GL11,
        RGBA16_SNORM[16,16,16,16]: img + cr + sign + GL31,
        RGB9_E5: usign + GL30,

        //sRGB
        SRGB8[8,8,8]: srgb + cr + usign + GL21,
        SRGB8_ALPHA8[8,8,8,8]: srgb + req_rend + usign + GL21,

        //
        //floating point
        //

        //half-precision float
        R16F[16]: img + req_rend + sign + GL30,
        RG16F[16,16]: img + req_rend + sign + GL30,
        RGB16F[16,16,16]: cr + sign + GL30,
        RGBA16F[16,16,16,16]: img + req_rend + sign + GL30, //Half-float

        //single-precision float
        R32F[32]: img + req_rend + sign + GL30,
        RG32F[32,32]: req_rend + sign + GL30,
        RGB32F[32,32,32]: img + cr + sign + GL30,
        RGBA32F[32,32,32,32]: img + req_rend + sign + GL30,

        //weird-ass float
        R11F_G11F_B10F[11,11,10]: img + req_rend + sign + GL30,

        //
        //compressed
        //

        //Red-green Texture Compression
        COMPRESSED_RED_RGTC1[u64; 4, 4, 1]: usign + GL30,
        COMPRESSED_SIGNED_RED_RGTC1[u64; 4, 4, 1]: sign + GL30,
        COMPRESSED_RG_RGTC2[[u64;2]; 4, 4, 1]: usign + GL30,
        COMPRESSED_SIGNED_RG_RGTC2[[u64;2]; 4, 4, 1]: sign + GL30,

        //BPTC
        COMPRESSED_RGBA_BPTC_UNORM[u128; 4, 4, 1]: usign + GL42,
        COMPRESSED_SRGB_ALPHA_BPTC_UNORM[u128; 4, 4, 1]: srgb + usign + GL42,
        COMPRESSED_RGB_BPTC_SIGNED_FLOAT[u128; 4, 4, 1]: sign + GL42,
        COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT[u128; 4, 4, 1]: usign + GL42,

        //Ericsson Texture Compression
        COMPRESSED_RGB8_ETC2[u64; 4, 4, 1]: usign + GL43,
        COMPRESSED_SRGB8_ETC2[u64; 4, 4, 1]: srgb + usign + GL43,
        COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2[u64; 4, 4, 1]: usign + GL43,
        COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2[u64; 4, 4, 1]: srgb + usign + GL43,
        COMPRESSED_RGBA8_ETC2_EAC[u128; 4, 4, 1]: usign + GL43,
        COMPRESSED_SRGB8_ALPHA8_ETC2_EAC[u128; 4, 4, 1]: srgb + usign + GL43,
        COMPRESSED_R11_EAC[u64; 4, 4, 1]: usign + GL43,
        COMPRESSED_SIGNED_R11_EAC[u64; 4, 4, 1]: sign + GL43,
        COMPRESSED_RG11_EAC[[u64;2]; 4, 4, 1]: usign + GL43,
        COMPRESSED_SIGNED_RG11_EAC[[u64;2]; 4, 4, 1]: sign + GL43,
    }

    pub enum IntFormat {
        //1-component
        R8I[8]: img + req_rend + sign + GL30,
        R16I[16]: img + req_rend + sign + GL30,
        R32I[32]: img + req_rend + sign + GL30,

        //2-component
        RG8I[8,8]: img + req_rend + sign + GL30,
        RG16I[16,16]: img + req_rend + sign + GL30,
        RG32I[32,32]: img + req_rend + sign + GL30,

        //3-component
        RGB8I[8,8,8]: cr + sign + GL30,
        RGB16I[16,16,16]: cr + sign + GL30,
        RGB32I[32,32,32]: cr + sign + GL30,

        //4-component
        RGBA8I[8,8,8,8]: img + req_rend + sign + GL30,
        RGBA16I[16,16,16,16]: img + req_rend + sign + GL30,
        RGBA32I[32,32,32,32]: img + req_rend + sign + GL30,
    }

    pub enum UIntFormat {
        //1-component
        R8UI[8]: img + req_rend + usign + GL30,
        R16UI[16]: img + req_rend + usign + GL30,
        R32UI[32]: img + req_rend + usign + GL30,

        //2-component
        RG8UI[8,8]: img + req_rend + usign + GL30,
        RG16UI[16,16]: img + req_rend + usign + GL30,
        RG32UI[32,32]: img + req_rend + usign + GL30,

        //3-component
        RGB8UI[8,8,8]: cr + usign + GL30,
        RGB16UI[16,16,16]: cr + usign + GL30,
        RGB32UI[32,32,32]: cr + usign + GL30,

        //4-component
        RGBA8UI[8,8,8,8]: img + req_rend + usign + GL30,
        RGBA16UI[16,16,16,16]: img + req_rend + usign + GL30,
        RGBA32UI[32,32,32,32]: img + req_rend + usign + GL30,

        //Weird shit
        RGB10_A2UI[10,10,10,2]: img + req_rend + usign + GL33,
    }

    pub enum DepthFormat {
        DEPTH_COMPONENT: req_rend + GL14, //base internal format
        DEPTH_COMPONENT16(16): req_rend + GL14,
        DEPTH_COMPONENT24(24): req_rend + GL14,
        DEPTH_COMPONENT32(32): cr + !,
        DEPTH_COMPONENT32F(32): req_rend + GL30,
    }

    pub enum StencilFormat {
        STENCIL_INDEX: req_rend + GL44, //base internal format
        STENCIL_INDEX1(0,1): cr + !,
        STENCIL_INDEX4(0,4): cr + !,
        STENCIL_INDEX8(0,8): req_rend + GL44,
        STENCIL_INDEX16(0,16): cr + !,
    }

    pub enum DepthStencilFormat {
        DEPTH_STENCIL: req_rend + GL30, //base internal format
        DEPTH24_STENCIL8(24,8): req_rend + GL30,
        DEPTH32F_STENCIL8(32,8): req_rend + GL30,
    }
}


unsafe impl ColorFormat for RGB9_E5 {}
unsafe impl RGBFormat for RGB9_E5 {}
unsafe impl SizedPixelFormat for RGB9_E5 {
    #[inline] fn red_bits() -> u8 {9}
    #[inline] fn green_bits() -> u8 {9}
    #[inline] fn blue_bits() -> u8 {9}
    #[inline] fn shared_bits() -> u8 {5}
}

unsafe impl ColorFormat for RED {}
unsafe impl RedFormat for RED {}
unsafe impl ColorFormat for RG {}
unsafe impl RGFormat for RG {}
unsafe impl ColorFormat for RGB {}
unsafe impl RGBFormat for RGB {}
unsafe impl ColorFormat for RGBA {}
unsafe impl RGBAFormat for RGBA {}

unsafe impl RedFormat for COMPRESSED_RED {}
unsafe impl RGFormat for COMPRESSED_RG {}
unsafe impl RGBFormat for COMPRESSED_RGB {}
unsafe impl RGBAFormat for COMPRESSED_RGBA {}

unsafe impl RedFormat for COMPRESSED_RED_RGTC1 {}
unsafe impl RedFormat for COMPRESSED_SIGNED_RED_RGTC1 {}
unsafe impl RGFormat for COMPRESSED_RG_RGTC2 {}
unsafe impl RGFormat for COMPRESSED_SIGNED_RG_RGTC2 {}
unsafe impl RGBAFormat for COMPRESSED_RGBA_BPTC_UNORM {}
unsafe impl RGBFormat for COMPRESSED_RGB_BPTC_SIGNED_FLOAT {}
unsafe impl RGBFormat for COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT {}
unsafe impl RGBFormat for COMPRESSED_RGB8_ETC2 {}
unsafe impl RGBFormat for COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2 {}
unsafe impl RGBAFormat for COMPRESSED_RGBA8_ETC2_EAC {}
unsafe impl RedFormat for COMPRESSED_R11_EAC {}
unsafe impl RedFormat for COMPRESSED_SIGNED_R11_EAC {}
unsafe impl RGFormat for COMPRESSED_RG11_EAC {}
unsafe impl RGFormat for COMPRESSED_SIGNED_RG11_EAC {}

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

// macro_rules! impl_buffer_tex {
//     ($($fmt:ty = $pixel:ty;)*) => {
//         $(
//             unsafe impl BufferTextureFormat for $fmt { type Pixel = $pixel; }
//         )*
//     }
// }

// impl_buffer_tex!{
//     //RED
//     R8       = GLubyte;
//     R16      = GLushort;
// //  R16F     = GLhalf;
//     R32F     = GLfloat;
//     R8I      = GLbyte;
//     R16I     = GLshort;
//     R32I     = GLint;
//     R8UI     = GLubyte;
//     R16UI    = GLushort;
//     R32UI    = GLuint;
//
//     //RG
//     RG8      = [GLubyte;  2];
//     RG16     = [GLushort; 2];
// //  RG16F    = [GLhalf;   2];
//     RG32F    = [GLfloat;  2];
//     RG8I     = [GLbyte;   2];
//     RG16I    = [GLshort;  2];
//     RG32I    = [GLint;    2];
//     RG8UI    = [GLubyte;  2];
//     RG16UI   = [GLushort; 2];
//     RG32UI   = [GLuint;   2];
//
//     //RGB
//     RGB32F   = [GLfloat;  3];
//     RGB32I   = [GLint;    3];
//     RGB32UI  = [GLuint;   3];
//
//     //RGBA
//     RGBA8    = [GLubyte;  4];
//     RGBA16   = [GLushort; 4];
// //  RGBA16F  = [GLhalf;   4];
//     RGBA32F  = [GLfloat;  4];
//     RGBA8I   = [GLbyte;   4];
//     RGBA16I  = [GLshort;  4];
//     RGBA32I  = [GLint;    4];
//     RGBA8UI  = [GLubyte;  4];
//     RGBA16UI = [GLushort; 4];
//     RGBA32UI = [GLuint;   4];
//
// }
