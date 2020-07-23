use super::*;
use std::fmt::{Debug, Display, Formatter};

macro_rules! tex_target {

    () => {};

    ([$name:ident $display:expr]; $GL:ty; $dim:ty, $($rest:tt)*) => {
        tex_target!([$name $display]; InternalFormat; $GL; $dim, $($rest)*);
    };

    ([$name:ident $display:expr]; $bound:ident; $GL:ty; $dim:ty, $($rest:tt)*) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub struct $name;

        impl Debug for $name {
            #[inline] fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, stringify!($name))
            }
        }

        impl Display for $name {
            #[inline] fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, $display)
            }
        }

        impl From<$name> for GLenum { #[inline(always)] fn from(_:$name) -> GLenum {gl::$name} }
        impl TryFrom<GLenum> for $name {
            type Error = GLError;
            #[inline(always)] fn try_from(val:GLenum) -> Result<Self,GLError> {
                if val == $name.into() { Ok($name) } else {Err(GLError::InvalidEnum(val,"".to_string()))}
            }
        }
        impl GLEnum for $name {}

        unsafe impl TextureType for $name { type GL = $GL; type Dim = $dim; }
        impl<F:$bound> TextureTarget<F> for $name {}

        tex_target!($($rest)*);
    }
}

pub unsafe trait TextureType: GLEnum + Default {
    type GL: GLVersion;
    type Dim: TexDim;

    #[inline] fn glenum() -> GLenum {Self::default().into()}

    #[inline]
    fn multisampled() -> bool {
        match Self::glenum() {
            gl::TEXTURE_2D_MULTISAMPLE | gl::TEXTURE_2D_MULTISAMPLE_ARRAY => true,
            _ => false
        }
    }

    #[inline]
    fn mipmapped() -> bool {
        match Self::glenum() {
            gl::TEXTURE_2D_MULTISAMPLE | gl::TEXTURE_2D_MULTISAMPLE_ARRAY => false,
            gl::TEXTURE_RECTANGLE | gl::TEXTURE_BUFFER => false,
            _ => true
        }
    }

    #[inline]
    fn cube_mapped() -> bool {
        match Self::glenum() {
            gl::TEXTURE_CUBE_MAP | gl::TEXTURE_CUBE_MAP_ARRAY => true,
            _ => false
        }
    }

}

#[marker] pub unsafe trait Owned: TextureType {}
#[marker] pub unsafe trait Sampled: Owned {}
#[marker] pub unsafe trait Multisampled: Owned {}
#[marker] pub unsafe trait Mipmapped: Sampled {}
#[marker] pub unsafe trait PixelTransfer: Sampled {}
#[marker] pub unsafe trait CompressedTransfer: Sampled + PixelTransfer {}
#[marker] pub unsafe trait BaseImage: Owned {}
#[marker] pub unsafe trait CubeMapped: Sampled {}
#[marker] pub unsafe trait Layered: Owned {}

tex_target! {
    [TEXTURE_1D "Texture 1D"]; GL10; [usize;1],
    [TEXTURE_2D "Texture 2D"]; GL10; [usize;2],
    [TEXTURE_3D "Texture 3D"]; ColorFormat; GL11; [usize;3],
    [TEXTURE_1D_ARRAY "Texture 1D Array"]; GL30; (<TEXTURE_1D as TextureType>::Dim, usize),
    [TEXTURE_2D_ARRAY "Texture 2D Array"]; GL30; (<TEXTURE_2D as TextureType>::Dim, usize),
    [TEXTURE_RECTANGLE "Texture Rectangle"]; GL31; <TEXTURE_2D as TextureType>::Dim,
    [TEXTURE_BUFFER "Texture Buffer"]; ColorFormat; GL31; usize,
    [TEXTURE_CUBE_MAP "Texture Cube Map"]; GL13; <TEXTURE_2D as TextureType>::Dim,
    [TEXTURE_CUBE_MAP_ARRAY "Texture Cube Map Array"]; GL40; <TEXTURE_2D_ARRAY as TextureType>::Dim,
    [TEXTURE_2D_MULTISAMPLE "Texture 2D Multisample"]; Renderable; GL32; <TEXTURE_2D as TextureType>::Dim,
    [TEXTURE_2D_MULTISAMPLE_ARRAY "Texture 2D Multisample Array"]; Renderable; GL32; <TEXTURE_2D_ARRAY as TextureType>::Dim,
}

//All but TEXTURE_BUFFER

unsafe impl Owned for TEXTURE_1D { }
unsafe impl Owned for TEXTURE_2D { }
unsafe impl Owned for TEXTURE_3D { }
unsafe impl Owned for TEXTURE_1D_ARRAY { }
unsafe impl Owned for TEXTURE_2D_ARRAY { }
unsafe impl Owned for TEXTURE_RECTANGLE { }
unsafe impl Owned for TEXTURE_CUBE_MAP {}
unsafe impl Owned for TEXTURE_CUBE_MAP_ARRAY {}
unsafe impl Owned for TEXTURE_2D_MULTISAMPLE {}
unsafe impl Owned for TEXTURE_2D_MULTISAMPLE_ARRAY {}

//All but TEXTURE_BUFFER and the multisample textures

unsafe impl Sampled for TEXTURE_1D { }
unsafe impl Sampled for TEXTURE_2D { }
unsafe impl Sampled for TEXTURE_3D { }
unsafe impl Sampled for TEXTURE_1D_ARRAY { }
unsafe impl Sampled for TEXTURE_2D_ARRAY { }
unsafe impl Sampled for TEXTURE_RECTANGLE { }
unsafe impl Sampled for TEXTURE_CUBE_MAP {}
unsafe impl Sampled for TEXTURE_CUBE_MAP_ARRAY {}

//All but TEXTURE_BUFFER and TEXTURE_CUBE_MAP

unsafe impl BaseImage for TEXTURE_1D { }
unsafe impl BaseImage for TEXTURE_2D { }
unsafe impl BaseImage for TEXTURE_3D { }
unsafe impl BaseImage for TEXTURE_1D_ARRAY { }
unsafe impl BaseImage for TEXTURE_2D_ARRAY { }
unsafe impl BaseImage for TEXTURE_RECTANGLE { }
unsafe impl BaseImage for TEXTURE_CUBE_MAP_ARRAY {}
unsafe impl BaseImage for TEXTURE_2D_MULTISAMPLE {}
unsafe impl BaseImage for TEXTURE_2D_MULTISAMPLE_ARRAY {}

//All but TEXTURE_BUFFER and the multisample textures

unsafe impl PixelTransfer for TEXTURE_1D { }
unsafe impl PixelTransfer for TEXTURE_2D { }
unsafe impl PixelTransfer for TEXTURE_3D { }
unsafe impl PixelTransfer for TEXTURE_1D_ARRAY { }
unsafe impl PixelTransfer for TEXTURE_2D_ARRAY { }
unsafe impl PixelTransfer for TEXTURE_RECTANGLE { }
unsafe impl PixelTransfer for TEXTURE_CUBE_MAP {}
unsafe impl PixelTransfer for TEXTURE_CUBE_MAP_ARRAY {}

//All but TEXTURE_BUFFER, TEXTURE_RECTANGLE, and the multisample textures

unsafe impl CompressedTransfer for TEXTURE_1D { }
unsafe impl CompressedTransfer for TEXTURE_2D { }
unsafe impl CompressedTransfer for TEXTURE_3D { }
unsafe impl CompressedTransfer for TEXTURE_1D_ARRAY { }
unsafe impl CompressedTransfer for TEXTURE_2D_ARRAY { }
unsafe impl CompressedTransfer for TEXTURE_CUBE_MAP {}
unsafe impl CompressedTransfer for TEXTURE_CUBE_MAP_ARRAY {}

//All but TEXTURE_BUFFER, TEXTURE_RECTANGLE, and the multisample textures

unsafe impl Mipmapped for TEXTURE_1D {}
unsafe impl Mipmapped for TEXTURE_2D {}
unsafe impl Mipmapped for TEXTURE_3D {}
unsafe impl Mipmapped for TEXTURE_1D_ARRAY {}
unsafe impl Mipmapped for TEXTURE_2D_ARRAY {}
unsafe impl Mipmapped for TEXTURE_CUBE_MAP {}
unsafe impl Mipmapped for TEXTURE_CUBE_MAP_ARRAY {}

//the array textures

unsafe impl Layered for TEXTURE_1D_ARRAY {}
unsafe impl Layered for TEXTURE_2D_ARRAY {}
unsafe impl Layered for TEXTURE_CUBE_MAP_ARRAY {}
unsafe impl Layered for TEXTURE_2D_MULTISAMPLE_ARRAY {}

unsafe impl CubeMapped for TEXTURE_CUBE_MAP {}
unsafe impl CubeMapped for TEXTURE_CUBE_MAP_ARRAY {}

unsafe impl Multisampled for TEXTURE_2D_MULTISAMPLE {}
unsafe impl Multisampled for TEXTURE_2D_MULTISAMPLE_ARRAY {}

#[marker] pub trait TextureTarget<F>: TextureType {}

impl<T:TextureType> TextureTarget<!> for T {}
impl<F,T:TextureTarget<F>> TextureTarget<MaybeUninit<F>> for T {}

//for some reason, the trait aliases have to be down here or else
//the syntax highlighting freaks out

pub trait OwnedTarget<F> = Owned + TextureTarget<F>;
pub trait SampledTarget<F> = Sampled + TextureTarget<F>;
pub trait MipmappedTarget<F> = Mipmapped + TextureTarget<F>;
pub trait MultisampledTarget<F> = Multisampled + TextureTarget<F>;
pub trait PixelTransferTarget<F> = PixelTransfer + TextureTarget<F>;
pub trait CompressedTransferTarget<F> = CompressedTransfer + TextureTarget<F>;
pub trait LayeredTarget<F> = Layered + TextureTarget<F>;
pub trait CubeMapTarget<F> = CubeMapped + TextureTarget<F>;