
use super::*;
use crate::gl;

use super::Target;

use std::convert::TryInto;
use std::marker::PhantomData;
use std::ops::Range;
use std::collections::HashMap;

use image_format::*;
pub use self::pixel_format::*;
pub use self::pixel_data::*;

mod pixel_format;
mod pixel_data;

glenum! {
    pub enum TextureTarget {
        [Texture1D TEXTURE_1D "Texture 1D"],
        [Texture2D TEXTURE_2D "Texture 2D"],
        [Texture3D TEXTURE_3D "Texture 3D"],
        [Texture1DArray TEXTURE_1D_ARRAY "Texture 1D Array"],
        [Texture2DArray TEXTURE_2D_ARRAY "Texture 2D Array"],
        [TextureRectangle TEXTURE_RECTANGLE "Texture Rectangle"],
        [TextureBuffer TEXTURE_BUFFER "Texture Buffer"],
        [TextureCubeMap TEXTURE_CUBE_MAP "Texture Cube Map"],
        [TextureCubeMapArray TEXTURE_CUBE_MAP_ARRAY "Texture Cube Map Array"],
        [Texture2DMultisample TEXTURE_2D_MULTISAMPLE "Texture 2D Multisample"],
        [Texture1DMultisampleArray TEXTURE_2D_MULTISAMPLE_ARRAY "Texture 2D Multisample Array"]
    }

    pub enum TextureSwizzle {
        [Red RED "Red Component"],
        [Green GREEN "Green Component"],
        [Blue BLUE "Blue Component"],
        [Alpha ALPHA "Alpha Component"],
        [Zero ZERO "Zero"],
        [One ONE "One"]
    }

}

impl TextureTarget {
    #[inline]
    fn multisample(self) -> bool {
        match self {
            Self::Texture2DMultisample | Self::Texture1DMultisampleArray => true,
            _ => false
        }
    }
}

gl_resource!{
    pub struct RawTex {
        gl = GL2,
        target = TextureTarget,
        gen = GenTextures,
        bind = BindTexture,
        is = IsTexture,
        delete = DeleteTextures
    }
}

pub unsafe trait TexDim:Copy {
    fn dim() -> usize;
    fn minimized(&self, level: GLuint) -> Self;

    #[inline] fn width(&self) -> usize {1}
    #[inline] fn height(&self) -> usize {1}
    #[inline] fn depth(&self) -> usize {1}

}

unsafe impl TexDim for [usize;1] {
    #[inline] fn dim() -> usize {1}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn minimized(&self, level: GLuint) -> Self { [(self[0]/(1 << level)).max(1)] }
}

unsafe impl TexDim for [usize;2] {
    #[inline] fn dim() -> usize {2}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn height(&self) -> usize {self[1]}
    #[inline] fn minimized(&self, level: GLuint) -> Self {
        let factor = 1 << level;
        [(self[0] / factor).max(1), (self[1] / factor).max(1)]
    }
}

unsafe impl TexDim for [usize;3] {
    #[inline] fn dim() -> usize {3}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn height(&self) -> usize {self[1]}
    #[inline] fn depth(&self) -> usize {self[2]}
    #[inline] fn minimized(&self, level: GLuint) -> Self {
        let factor = 1 << level;
        [(self[0] / factor).max(1), (self[1] / factor).max(1), (self[2] / factor).max(1)]
    }
}

unsafe impl TexDim for ([usize;1], usize) {
    #[inline] fn dim() -> usize {2}
    #[inline] fn minimized(&self, level: GLuint) -> Self {(self.0.minimized(level), self.1)}
    #[inline] fn width(&self) -> usize {self.0[0]}
    #[inline] fn height(&self) -> usize {self.1}
}

unsafe impl TexDim for ([usize;2], usize) {
    #[inline] fn dim() -> usize {3}
    #[inline] fn minimized(&self, level: GLuint) -> Self {(self.0.minimized(level), self.1)}
    #[inline] fn width(&self) -> usize {self.0[0]}
    #[inline] fn height(&self) -> usize {self.0[1]}
    #[inline] fn depth(&self) -> usize {self.1}
}

unsafe fn texture_storage<T:Texture>(
    _gl:&GL4, tex: &mut RawTex, levels: GLuint, dim: T::Dim, sampling: Option<(GLsizei,GLboolean)>
) where T::InternalFormat: SizedInternalFormat {
    let mut target = T::target().as_loc();
    let binding = target.bind(tex);
    let fmt = T::InternalFormat::glenum();
    let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);

    match sampling {
        Some((samples, fixed)) => match T::Dim::dim() {
            2 => gl::TexStorage2DMultisample(binding.target_id(), samples, fmt, w, h, fixed),
            3 => gl::TexStorage3DMultisample(binding.target_id(), samples, fmt, w, h, d, fixed),
            _ => panic!("{}D Multisample textures not currently supported", T::Dim::dim())
        },
        None => match T::Dim::dim() {
            1 => gl::TexStorage1D(binding.target_id(), levels as GLsizei, fmt, w),
            2 => gl::TexStorage2D(binding.target_id(), levels as GLsizei, fmt, w, h),
            3 => gl::TexStorage3D(binding.target_id(), levels as GLsizei, fmt, w, h, d),
            _ => panic!("{}D Textures not currently supported", T::Dim::dim())
        }
    }
}

pub unsafe trait Texture: Sized {
    type InternalFormat: InternalFormat<FormatType=Self::PixelFormat>;
    type PixelFormat: PixelFormatType;
    type Dim: TexDim;

    type BaseImage: Image<
        Dim = <Self as Texture>::Dim,
        PixelFormat = <Self as Texture>::PixelFormat,
        InternalFormat = <Self as Texture>::InternalFormat,
    >;

    fn target() -> TextureTarget;
    fn id(&self) -> GLuint;
    fn dim(&self) -> Self::Dim;

    unsafe fn alloc(gl:&GL2, dim:Self::Dim) -> Self;
    unsafe fn alloc_image(gl:&GL2, raw:RawTex, dim:Self::Dim) -> Self;
    unsafe fn alloc_storage(gl:&GL4, raw:RawTex, dim:Self::Dim) -> Self where Self::InternalFormat:SizedInternalFormat;

    fn from_pixels<P:PixelData<Self::PixelFormat>+?Sized>(gl:&GL2, dim:Self::Dim, data:&P) -> Self;
    fn image<P:PixelData<Self::PixelFormat>+?Sized>(gl:&GL2, raw:RawTex, dim:Self::Dim, data:&P) -> Self;
    fn storage<P>(gl:&GL4, raw:RawTex, dim:Self::Dim, data:&P) -> Self
        where P:PixelData<Self::PixelFormat>+?Sized, Self::InternalFormat:SizedInternalFormat;

    fn base_image(&mut self) -> Self::BaseImage;
}

pub unsafe trait MipmappedTexture: Texture {

    unsafe fn allocated_levels(&self) -> GLuint;
    unsafe fn base_level(&self) -> GLuint;
    unsafe fn max_level(&self) -> GLuint;

    unsafe fn set_base_level(&mut self);
    unsafe fn set_max_level(&mut self);

    unsafe fn alloc_mipmaps(gl:&GL2, levels:GLuint, dim:Self::Dim) -> Self;
    unsafe fn alloc_image_mipmaps(gl:&GL2, levels:GLuint, dim:Self::Dim) -> Self;
    unsafe fn alloc_storage_mipmaps(gl:&GL2, levels:GLuint, dim:Self::Dim) -> Self;

    fn from_mipmaps<P:PixelData<Self::PixelFormat>>(
        gl:&GL2, levels:GLuint, dim:Self::Dim, base:P, mipmaps: Option<HashMap<GLuint, &P>>
    ) -> Self;

    fn image<P:PixelData<Self::PixelFormat>>(
        gl:&GL2, raw:RawTex, levels:GLuint, dim:Self::Dim, base:P, mipmaps: Option<HashMap<GLuint, &P>>
    ) -> Self;

    fn storage<P>(
        gl:&GL4, raw:RawTex, levels:GLuint, dim:Self::Dim, base:P, mipmaps: Option<HashMap<GLuint, &P>>
    ) -> Self where P:PixelData<Self::PixelFormat>, Self::InternalFormat:SizedInternalFormat;

    fn gen_mipmaps(&mut self, range: Range<GLuint>);
    fn image_level(&mut self, level: GLuint) -> MipmapLevel<Self>;
    fn image_level_range(&mut self, range: Range<GLuint>) -> Box<[MipmapLevel<Self>]>;

}

pub unsafe trait Image: Sized {
    type InternalFormat: InternalFormat<FormatType=Self::PixelFormat>;
    type PixelFormat: PixelFormatType;
    type Dim: TexDim;

    fn target() -> TextureTarget;
    fn id(&self) -> GLuint;
    fn level(&self) -> GLuint;
    fn dim(&self) -> Self::Dim;

    fn image<P:PixelData<Self::PixelFormat>+?Sized>(data:&P) -> Self;
    fn sub_image<P:PixelData<Self::PixelFormat>+?Sized>(offset:Self::Dim, dim:Self::Dim, data:&P);

    fn clear_image<P:PixelType<Self::PixelFormat>+?Sized>(data:&P);
    fn clear_sub_image<P:PixelType<Self::PixelFormat>+?Sized>(offset:Self::Dim, dim:Self::Dim, data:&P);

    fn get_image<P:PixelDataMut<Self::PixelFormat>+?Sized>(data: &mut P) -> Self;
    fn get_sub_image<P:PixelDataMut<Self::PixelFormat>+?Sized>(offset:Self::Dim, dim:Self::Dim, data: &mut P) -> Self;

}

pub struct MipmapLevel<'a, T:MipmappedTexture> {
    tex: &'a mut T,
    level: GLuint
}

pub struct Tex1D<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;1],
    fmt: PhantomData<F>
}

pub struct Tex1DArray<F:InternalFormat> {
    raw: RawTex,
    dim: (usize, [usize;1]),
    fmt: PhantomData<F>
}

pub struct Tex2D<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;2],
    fmt: PhantomData<F>
}

pub struct TexCubemap<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;2],
    fmt: PhantomData<F>
}

pub struct TexRectangle<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;2],
    fmt: PhantomData<F>
}

pub struct Tex2DMultisample<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;2],
    fmt: PhantomData<F>
}

pub struct Tex2DArray<F:InternalFormat> {
    raw: RawTex,
    dim: (usize, [usize;2]),
    fmt: PhantomData<F>
}

pub struct TexCubemapArray<F:InternalFormat> {
    raw: RawTex,
    dim: (usize, [usize;2]),
    fmt: PhantomData<F>
}

pub struct Tex2DMultisampleArray<F:InternalFormat> {
    raw: RawTex,
    dim: (usize, [usize;2]),
    fmt: PhantomData<F>
}

pub struct Tex3D<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;3],
    fmt: PhantomData<F>
}
