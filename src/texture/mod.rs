
use super::*;
use crate::gl;
use crate::gl::types::*;

use super::Target;

use std::convert::TryInto;

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

unsafe impl Target for TextureTarget {
    type Resource = RawTex;
    #[inline] unsafe fn bind(self, id:GLuint) {gl::BindTexture(self as GLenum, id)}
}


gl_resource!{
    pub struct RawTex {
        gl = GL2,
        target = TextureTarget,
        gen = GenTextures,
        is = IsTexture,
        delete = DeleteTextures
    }
}

pub unsafe trait TexDim:Copy {
    fn dim() -> usize;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn depth(&self) -> usize;
}

unsafe impl TexDim for [usize;1] {
    fn dim() -> usize {1}
    fn width(&self) -> usize {self[0]}
    fn height(&self) -> usize {0}
    fn depth(&self) -> usize {0}
}

unsafe impl TexDim for [usize;2] {
    fn dim() -> usize {2}
    fn width(&self) -> usize {self[0]}
    fn height(&self) -> usize {self[1]}
    fn depth(&self) -> usize {0}
}

unsafe impl TexDim for [usize;3] {
    fn dim() -> usize {3}
    fn width(&self) -> usize {self[0]}
    fn height(&self) -> usize {self[1]}
    fn depth(&self) -> usize {self[2]}
}

pub unsafe trait Texture: Sized {
    type InternalFormat: InternalFormat<TypeFormat=Self::PixelFormat>;
    type PixelFormat: PixelFormatType;
    type Dim: TexDim;

    fn target() -> TextureTarget;
    fn id(&self) -> GLuint;
    fn format(&self) -> Self::InternalFormat;
    fn dim(&self) -> Self::Dim;

    fn storage<P:PixelData<Self::PixelFormat>>(
        _gl:&GL4,
        raw:RawTex,
        levels:usize,
        internalformat:Self::InternalFormat,
        dim:Self::Dim,
        pixels:P
    ) -> Self
    {
        unsafe {
            let mut target = Self::target().as_loc();
            let binding = target.bind(&raw);
            match Self::Dim::dim() {
                1 => gl::TexStorage1D(
                    binding.target_id(),
                    levels as GLint, internalformat.into() as GLuint,
                    dim.width() as GLsizei
                ),
                2 => gl::TexStorage2D(
                    binding.target_id(),
                    levels as GLint, internalformat.into() as GLuint,
                    dim.width() as GLsizei, dim.height() as GLsizei
                ),
                3 => gl::TexStorage3D(
                    binding.target_id(),
                    levels as GLint, internalformat.into() as GLuint,
                    dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei
                ),
                _ => panic!("{}D textures not supported", Self::Dim::dim())
            };
            ::std::mem::uninitialized()
        }
    }

}

pub struct SubTexture<'a, T:Texture> {
    tex: &'a T,
    offset: T::Dim,
    dim: T::Dim
}

pub struct SubTextureMut<'a, T:Texture> {
    tex: &'a mut T,
    offset: T::Dim,
    dim: T::Dim
}

impl<'a,T:Texture> SubTextureMut<'a,T> {
    pub fn sub_image<P:PixelData<T::PixelFormat>>(&mut self, pixels:P) {
        unsafe {

            let mut target = T::target().as_loc();
            let mut buf_target = buffer_new::BufferTarget::PixelPackBuffer.as_loc();

            let binding = target.bind_raw(self.tex.id()).unwrap();
            let buf_binding = pixels.bind_pixel_buffer(&mut buf_target);

            let (fmt, ty) = pixels.format_type().format_type();
            let (x,y,z) = (
                self.offset.width() as GLsizei,
                self.offset.height() as GLsizei,
                self.offset.depth() as GLsizei
            );
            let (w,h,d) = (
                self.dim.width() as GLsizei,
                self.dim.height() as GLsizei,
                self.dim.depth() as GLsizei
            );

            let dim = T::Dim::dim();
            if cfg!(debug_assertions) {
                let len = if dim == 3 {w*h*d} else if dim == 2 {w*h} else {w};
                if len != pixels.count() as GLsizei {
                    panic!("invalid number of pixels for dimensions ({},{},{})",w,h,d);
                }
            }

            apply_packing_settings(&pixels);
            match dim {
                1 => gl::TexSubImage1D(
                    binding.target_id(), 0, x, w,
                    fmt.into(), ty as GLenum, pixels.pixels()
                ),
                2 => gl::TexSubImage2D(
                    binding.target_id(), 0, x,y, w,h,
                    fmt.into(), ty as GLenum, pixels.pixels()
                ),
                3 => gl::TexSubImage3D(
                    binding.target_id(), 0, x,y,z, w,h,d,
                    fmt.into(), ty as GLenum, pixels.pixels()
                ),
                _ => panic!("{}D textures not supported", T::Dim::dim())
            };
            drop(buf_binding);
        }
    }
}


pub struct Tex1D<F:InternalFormat> {
    raw: RawTex,
    format: F,
    dim: [usize;1]
}



pub struct Tex2D<F:InternalFormat> {
    raw: RawTex,
    format: F,
    dim: [usize;2]
}

pub struct Tex3D<F:InternalFormat> {
    raw: RawTex,
    format: F,
    dim: [usize;3]
}
