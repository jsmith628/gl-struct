use super::*;
use crate::gl;
use crate::Target;

use image_format::*;
use buffer_new::BufferTarget;

use std::convert::TryInto;
use std::marker::PhantomData;
use std::ops::{Bound,RangeBounds};
use std::collections::HashMap;
use std::mem::{size_of, uninitialized};

use self::helper_methods::*;
#[macro_use] mod helper_methods;

pub use self::raw::*;
pub use self::dim::*;
pub use self::mipmapped::*;
pub use self::multisampled::*;
pub use self::rectangle::*;

mod raw;
mod dim;
mod mipmapped;
mod multisampled;
mod rectangle;

glenum! {
    pub enum TextureSwizzle {
        [Red RED "Red Component"],
        [Green GREEN "Green Component"],
        [Blue BLUE "Blue Component"],
        [Alpha ALPHA "Alpha Component"],
        [Zero ZERO "Zero"],
        [One ONE "One"]
    }
}

pub unsafe trait Texture: Sized {
    //the types that matter
    type InternalFormat: InternalFormat<ClientFormat=Self::ClientFormat>;
    type Target: TextureTarget<Dim=Self::Dim, GL=Self::GL>;

    //the types that are effectively aliases
    type ClientFormat: ClientFormat;
    type Dim: TexDim;
    type GL: GLProvider;

    fn dim(&self) -> Self::Dim;
    fn raw(&self) -> &RawTex<Self::Target>;

    unsafe fn from_raw(raw:RawTex<Self::Target>, dim:Self::Dim) -> Self;

    #[inline]
    fn immutable_storage(&self) -> bool {
        unsafe { get_tex_parameter_iv(self, gl::TEXTURE_IMMUTABLE_FORMAT) != 0 }
    }

    #[inline]
    unsafe fn alloc(gl:&Self::GL, dim:Self::Dim) -> Self {
        let raw = RawTex::gen(gl);
        if let Ok(gl4) = gl.try_as_gl4() {
            if_sized!(
                helper()(_gl:&GL4,tex:RawTex<T::Target>,d:T::Dim) -> T
                    {unsafe{T::image(tex, d)}}
                    {unsafe{T::storage(_gl, tex, d)}}
                where
            );
            Self::InternalFormat::helper(&gl4, raw, dim)
        } else {
            Self::image(raw, dim)
        }
    }

    #[inline]
    unsafe fn image(mut raw:RawTex<Self::Target>, dim:Self::Dim) -> Self {
        if Self::Target::multisampled() {
            tex_image_multisample::<Self>(&mut raw, dim, 0, false)
        } else {
            tex_image_null::<Self>(raw.id(), 0, dim)
        }
        Self::from_raw(raw, dim)
    }

    #[inline]
    unsafe fn storage(gl:&GL4, raw:RawTex<Self::Target>, dim:Self::Dim) -> Self
        where Self::InternalFormat:SizedInternalFormat
    {
        let sampling = if Self::Target::multisampled() {Some((0,false))} else {None};
        tex_storage::<Self>(gl, raw, 1, dim, sampling)
    }

}

pub unsafe trait PixelTransfer: Texture {

    type BaseImage: Image<
        Dim = Self::Dim,
        ClientFormat = <Self as Texture>::ClientFormat,
        InternalFormat = <Self as Texture>::InternalFormat,
    >;

    #[inline] fn swizzle_r(&mut self, param:TextureSwizzle) where Self::InternalFormat: InternalFormatColor
        { unsafe{ swizzle_param(self, gl::TEXTURE_SWIZZLE_R, param) } }
    #[inline] fn swizzle_g(&mut self, param:TextureSwizzle) where Self::InternalFormat: InternalFormatColor
        { unsafe{ swizzle_param(self, gl::TEXTURE_SWIZZLE_G, param) } }
    #[inline] fn swizzle_b(&mut self, param:TextureSwizzle) where Self::InternalFormat: InternalFormatColor
        { unsafe{ swizzle_param(self, gl::TEXTURE_SWIZZLE_B, param) } }
    #[inline] fn swizzle_a(&mut self, param:TextureSwizzle) where Self::InternalFormat: InternalFormatColor
        { unsafe{ swizzle_param(self, gl::TEXTURE_SWIZZLE_A, param) } }

    fn swizzle_rgba(&mut self, red:TextureSwizzle, green:TextureSwizzle, blue:TextureSwizzle, alpha:TextureSwizzle)
    where Self::InternalFormat: InternalFormatColor
    {
        unsafe {
            let mut swizzle = [red as GLint, green as GLint, blue as GLint, alpha as GLint];
            if gl::TextureParameteriv::is_loaded() {
                gl::TextureParameteriv(self.raw().id(), gl::TEXTURE_SWIZZLE_RGBA, &mut swizzle[0] as *mut GLint);
            } else {
                let mut target = Self::Target::binding_location();
                let binding = target.bind(self.raw());
                gl::TexParameteriv(binding.target_id(), gl::TEXTURE_SWIZZLE_RGBA, &mut swizzle[0] as *mut GLint);
            }
        }
    }

    #[inline] fn get_swizzle_r(&self) -> TextureSwizzle where Self::InternalFormat: InternalFormatColor
        { unsafe{ get_swizzle_param(self, gl::TEXTURE_SWIZZLE_R) } }
    #[inline] fn get_swizzle_g(&self) -> TextureSwizzle where Self::InternalFormat: InternalFormatColor
        { unsafe{ get_swizzle_param(self, gl::TEXTURE_SWIZZLE_G) } }
    #[inline] fn get_swizzle_b(&self) -> TextureSwizzle where Self::InternalFormat: InternalFormatColor
        { unsafe{ get_swizzle_param(self, gl::TEXTURE_SWIZZLE_B) } }
    #[inline] fn get_swizzle_a(&self) -> TextureSwizzle where Self::InternalFormat: InternalFormatColor
        { unsafe{ get_swizzle_param(self, gl::TEXTURE_SWIZZLE_A) } }

    fn get_swizzle_rgba(&self) -> [TextureSwizzle;4] where Self::InternalFormat: InternalFormatColor{
        unsafe {
            let mut swizzle = uninitialized::<[GLint;4]>();
            if gl::GetTextureParameteriv::is_loaded() {
                gl::GetTextureParameteriv(self.raw().id(), gl::TEXTURE_SWIZZLE_RGBA, &mut swizzle[0] as *mut GLint);
            } else {
                let mut target = Self::Target::binding_location();
                let binding = target.bind(self.raw());
                gl::GetTexParameteriv(binding.target_id(), gl::TEXTURE_SWIZZLE_RGBA, &mut swizzle[0] as *mut GLint);
            }
            [
                (swizzle[0] as GLenum).try_into().unwrap(),
                (swizzle[1] as GLenum).try_into().unwrap(),
                (swizzle[2] as GLenum).try_into().unwrap(),
                (swizzle[3] as GLenum).try_into().unwrap(),
            ]
        }
    }

    fn from_pixels<P:PixelData<Self::ClientFormat>+?Sized>(
        gl:&Self::GL, dim:Self::Dim, data:&P
    ) -> Self {
        let raw = RawTex::gen(gl);
        if let Ok(gl4) = gl.try_as_gl4() {
            if_sized!(
                helper(P:PixelData<T::ClientFormat>+?Sized)(
                    _gl:&GL4,tex:RawTex<T::Target>,d:T::Dim,p:&P
                ) -> T
                    {T::image_from_pixels(tex, d, p)}
                    {T::storage_from_pixels(_gl, tex, d, p)}
                where T:PixelTransfer
            );
            Self::InternalFormat::helper(&gl4, raw, dim, data)
        } else {
            Self::image_from_pixels(raw, dim, data)
        }

    }

    fn image_from_pixels<P:PixelData<Self::ClientFormat>+?Sized>(raw:RawTex<Self::Target>, dim:Self::Dim, data:&P) -> Self {
        unsafe {
            tex_image_data::<Self, _>(raw.id(), 0, dim, data);
            Self::from_raw(raw, dim)
        }
    }

    fn storage_from_pixels<P>(gl:&GL4, raw:RawTex<Self::Target>, dim:Self::Dim, data:&P) -> Self
        where P:PixelData<Self::ClientFormat>+?Sized, Self::InternalFormat:SizedInternalFormat
    {
        let mut tex = unsafe { Self::storage(gl, raw, dim) };
        tex.base_image().image(data);
        tex
    }

    fn base_image(&mut self) -> Self::BaseImage;
}

pub unsafe trait Image: Sized {
    type InternalFormat: InternalFormat<ClientFormat=Self::ClientFormat>;
    type ClientFormat: ClientFormat;
    type Dim: TexDim;
    type Target: TextureTarget;

    fn raw(&self) -> &RawTex<Self::Target>;
    fn dim(&self) -> Self::Dim;
    fn level(&self) -> GLuint;

    fn image<P:PixelData<Self::ClientFormat>+?Sized>(&mut self, data:&P);
    fn sub_image<P:PixelData<Self::ClientFormat>+?Sized>(&mut self, offset:Self::Dim, dim:Self::Dim, data:&P);

    fn clear_image<P:PixelType<Self::ClientFormat>>(&mut self, data:P);
    fn clear_sub_image<P:PixelType<Self::ClientFormat>>(&mut self, offset:Self::Dim, dim:Self::Dim, data:P);

    fn get_image<P:PixelDataMut<Self::ClientFormat>+?Sized>(&self, data: &mut P);

    fn into_box<P:PixelType<Self::ClientFormat>>(&self) -> Box<[P]> {
        let size = size_of::<P>()*self.dim().pixels();
        let mut dest = Vec::with_capacity(size);
        unsafe { dest.set_len(size) };
        self.get_image(dest.as_mut_slice());
        dest.into_boxed_slice()
    }

}

macro_rules! impl_tex {
    ($name:ident; $target:ident; $kind:ident) => {
        unsafe impl<F:InternalFormat> Texture for $name<F> {
            type InternalFormat = F;
            type Target = $target;

            type ClientFormat = F::ClientFormat;
            type Dim = <$target as TextureTarget>::Dim;
            type GL = <$target as TextureTarget>::GL;

            #[inline] fn dim(&self) -> Self::Dim {self.dim}
            #[inline] fn raw(&self) -> &RawTex<Self::Target> {&self.raw}

            #[inline] unsafe fn from_raw(raw:RawTex<Self::Target>, dim:Self::Dim) -> Self {
                $name {raw: raw, dim: dim, fmt: PhantomData}
            }
        }

        impl_tex!(@$kind $name; $target);
    };

    (@mipmap $name:ident; $target:ident) => {
        unsafe impl<F:InternalFormat> PixelTransfer for $name<F> {
            type BaseImage = MipmapLevel<Self>;
            #[inline] fn base_image(&mut self) -> MipmapLevel<Self> {self.level(0)}
        }

        unsafe impl<F:InternalFormat> MipmappedTexture for $name<F> {}
    };

    (@multisample $name:ident; $target:ident) => {
        unsafe impl<F:InternalFormat> MultisampledTexture for $name<F> {}
    }
}

pub struct Tex1D<F:InternalFormat> {
    raw: RawTex<TEXTURE_1D>,
    dim: [usize;1],
    fmt: PhantomData<F>
}
impl_tex!(Tex1D; TEXTURE_1D; mipmap);

pub struct Tex2D<F:InternalFormat> {
    raw: RawTex<TEXTURE_2D>,
    dim: [usize;2],
    fmt: PhantomData<F>
}
impl_tex!(Tex2D; TEXTURE_2D; mipmap);

pub struct Tex3D<F:InternalFormat> {
    raw: RawTex<TEXTURE_3D>,
    dim: [usize;3],
    fmt: PhantomData<F>
}
impl_tex!(Tex3D; TEXTURE_3D; mipmap);

pub struct Tex1DArray<F:InternalFormat> {
    raw: RawTex<TEXTURE_1D_ARRAY>,
    dim: ([usize;1], usize),
    fmt: PhantomData<F>
}
impl_tex!(Tex1DArray; TEXTURE_1D_ARRAY; mipmap);

pub struct Tex2DArray<F:InternalFormat> {
    raw: RawTex<TEXTURE_2D_ARRAY>,
    dim: ([usize;2], usize),
    fmt: PhantomData<F>
}
impl_tex!(Tex2DArray; TEXTURE_2D_ARRAY; mipmap);

pub struct Tex2DMultisample<F:InternalFormat> {
    raw: RawTex<TEXTURE_2D_MULTISAMPLE>,
    dim: [usize;2],
    fmt: PhantomData<F>
}
impl_tex!(Tex2DMultisample; TEXTURE_2D_MULTISAMPLE; multisample);

pub struct Tex2DMultisampleArray<F:InternalFormat> {
    raw: RawTex<TEXTURE_2D_MULTISAMPLE_ARRAY>,
    dim: ([usize;2], usize),
    fmt: PhantomData<F>
}
impl_tex!(Tex2DMultisampleArray; TEXTURE_2D_MULTISAMPLE_ARRAY; multisample);

// pub struct TexCubeMap<F:InternalFormat> {
//     raw: RawTex<TEXTURE_CUBE_MAP>,
//     dim: [usize;2],
//     fmt: PhantomData<F>
// }
//

// pub struct TexCubemapArray<F:InternalFormat> {
//     raw: RawTex<TEXTURE_CUBE_MAP_ARRAY>,
//     dim: (usize, [usize;2]),
//     fmt: PhantomData<F>
// }
