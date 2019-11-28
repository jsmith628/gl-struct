use super::*;
use std::fmt::{Debug, Display, Formatter};

macro_rules! target {

    () => {};

    ([$name:ident $display:expr]; $GL:ty; $dim:ty, $($rest:tt)*) => {
        target!([$name $display]; InternalFormat; $GL; $dim, $($rest)*);
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
        impl<F:$bound> Target<Texture<F,Self>> for $name {
            fn target_id(self) -> GLenum { self.into() }
            unsafe fn bind(self, tex: &Texture<F,Self>) { gl::BindTexture(self.into(), tex.id()) }
            unsafe fn unbind(self) { gl::BindTexture(self.into(), 0) }
        }

        target!($($rest)*);
    }
}

pub unsafe trait TextureType: GLEnum + Default {
    type GL: GLVersion;
    type Dim: TexDim;

    #[inline] fn glenum() -> GLenum {Self::default().into()}

    #[inline]
    fn bind_loc<F>() -> BindingLocation<Texture<F,Self>,Self> where Self: Target<Texture<F,Self>> {
        unsafe {Self::default().as_loc()}
    }

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

pub trait TextureTarget<F> = TextureType + Target<Texture<F,Self>>;

#[marker] pub unsafe trait Mipmapped: TextureType {}
#[marker] pub unsafe trait Multisampled: TextureType {}

impl<T:TextureType> Target<UninitTex<Self>> for T {
    fn target_id(self) -> GLenum { self.into() }
    unsafe fn bind(self, tex: &UninitTex<Self>) { gl::BindTexture(self.into(), tex.id()) }
    unsafe fn unbind(self) { gl::BindTexture(self.into(), 0) }
}

impl<F,T:TextureTarget<F>> Target<Texture<MaybeUninit<F>,Self>> for T {
    fn target_id(self) -> GLenum { self.into() }
    unsafe fn bind(self, tex: &Texture<MaybeUninit<F>,Self>) { gl::BindTexture(self.into(), tex.id()) }
    unsafe fn unbind(self) { gl::BindTexture(self.into(), 0) }
}

target! {
    [TEXTURE_1D "Texture 1D"]; GL10; [usize;1],
    [TEXTURE_2D "Texture 2D"]; GL10; [usize;2],
    [TEXTURE_3D "Texture 3D"]; InternalFormatColor; GL11; [usize;3],
    [TEXTURE_1D_ARRAY "Texture 1D Array"]; GL30; (<TEXTURE_1D as TextureType>::Dim, usize),
    [TEXTURE_2D_ARRAY "Texture 2D Array"]; GL30; (<TEXTURE_2D as TextureType>::Dim, usize),
    [TEXTURE_RECTANGLE "Texture Rectangle"]; GL31; <TEXTURE_2D as TextureType>::Dim,
    [TEXTURE_BUFFER "Texture Buffer"]; InternalFormatColor; GL31; usize,
    [TEXTURE_CUBE_MAP "Texture Cube Map"]; GL13; <TEXTURE_2D as TextureType>::Dim,
    [TEXTURE_CUBE_MAP_ARRAY "Texture Cube Map Array"]; GL40; <TEXTURE_2D_ARRAY as TextureType>::Dim,
    [TEXTURE_2D_MULTISAMPLE "Texture 2D Multisample"]; GL32; <TEXTURE_2D as TextureType>::Dim,
    [TEXTURE_2D_MULTISAMPLE_ARRAY "Texture 2D Multisample Array"]; GL32; <TEXTURE_2D_ARRAY as TextureType>::Dim,
}

unsafe impl Mipmapped for TEXTURE_1D {}
unsafe impl Mipmapped for TEXTURE_2D {}
unsafe impl Mipmapped for TEXTURE_3D {}
unsafe impl Mipmapped for TEXTURE_1D_ARRAY {}
unsafe impl Mipmapped for TEXTURE_2D_ARRAY {}
unsafe impl Mipmapped for TEXTURE_CUBE_MAP {}
unsafe impl Mipmapped for TEXTURE_CUBE_MAP_ARRAY {}

unsafe impl Multisampled for TEXTURE_2D_MULTISAMPLE {}
unsafe impl Multisampled for TEXTURE_2D_MULTISAMPLE_ARRAY {}