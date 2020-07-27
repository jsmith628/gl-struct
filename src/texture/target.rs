use super::*;
use std::fmt::{Debug, Display, Formatter};

macro_rules! tex_target {

    () => {};

    ([$name:ident $display:literal]; $GL:ty; $dim:ty, $($rest:tt)*) => {
        tex_target!([$name $display]; InternalFormat; $GL; $dim, $($rest)*);
    };

    //for multisampled targets
    ([$name:ident<$ms:ident> $display:literal]; $bound:ident; $GL:ty; $dim:ty, $($rest:tt)*) => {
        #[allow(non_camel_case_types)]
        #[derive(Derivative)]
        #[derivative(Clone(bound=""), Copy(bound=""), PartialEq, Eq, Hash, Default)]
        pub struct $name<$ms:MultisampleFormat> { ms: ::std::marker::PhantomData<$ms> }

        impl<$ms:MultisampleFormat> Debug for $name<$ms> {
            fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(
                    f, concat!(stringify!($name), "x{}{}"),
                    $ms::SAMPLES,
                    if $ms::FIXED {"FIXED"} else {""}
                )
            }
        }

        impl<$ms:MultisampleFormat> Display for $name<$ms> {
            fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(
                    f, "{} x{} {}", $display, $ms::SAMPLES, if $ms::FIXED {"Fixed"} else {""}
                )
            }
        }

        impl<$ms:MultisampleFormat> From<$name<$ms>> for GLenum {
            fn from(_:$name<$ms>) -> GLenum {gl::$name}
        }

        impl<$ms:MultisampleFormat> TryFrom<GLenum> for $name<$ms> {
            type Error = GLError;
            fn try_from(val:GLenum) -> Result<Self,GLError> {
                if val == gl::$name {
                    Ok(Self{ms: PhantomData})
                } else {
                    Err(GLError::InvalidEnum(val,"".to_string()))
                }
            }
        }
        impl<$ms:MultisampleFormat> GLEnum for $name<$ms> {}

        unsafe impl<$ms:MultisampleFormat> TextureType for $name<$ms> { type GL = $GL; type Dim = $dim; }
        impl<$ms:MultisampleFormat,F:$bound> TextureTarget<F> for $name<$ms> {}

        tex_target!($($rest)*);
    };

    ([$name:ident $display:literal]; $bound:ident; $GL:ty; $dim:ty, $($rest:tt)*) => {
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

#[marker] pub unsafe trait Owned: TextureType + 'static {}
#[marker] pub unsafe trait Sampled: Owned {}
#[marker] pub unsafe trait Mipmapped: Sampled {}
#[marker] pub unsafe trait PixelTransfer: Sampled {}
#[marker] pub unsafe trait CompressedTransfer: Sampled + PixelTransfer {}
#[marker] pub unsafe trait BaseImage: Owned {}
#[marker] pub unsafe trait CubeMapped: Sampled {}
#[marker] pub unsafe trait Layered: Owned {}

pub unsafe trait Multisampled: Owned {
    const SAMPLES: GLuint;
    const FIXED: bool;
}

tex_target! {
    [TEXTURE_1D "Texture 1D"]; GL10; [usize;1],
    [TEXTURE_2D "Texture 2D"]; GL10; [usize;2],
    [TEXTURE_3D "Texture 3D"]; ColorFormat; GL11; [usize;3],
    [TEXTURE_1D_ARRAY "Texture 1D Array"]; GL30; (<TEXTURE_1D as TextureType>::Dim, usize),
    [TEXTURE_2D_ARRAY "Texture 2D Array"]; GL30; (<TEXTURE_2D as TextureType>::Dim, usize),
    [TEXTURE_RECTANGLE "Texture Rectangle"]; GL31; <TEXTURE_2D as TextureType>::Dim,
    // [TEXTURE_BUFFER "Texture Buffer"]; ColorFormat; GL31; usize,
    [TEXTURE_CUBE_MAP "Texture Cube Map"]; GL13; <TEXTURE_2D as TextureType>::Dim,
    [TEXTURE_CUBE_MAP_ARRAY "Texture Cube Map Array"]; GL40; <TEXTURE_2D_ARRAY as TextureType>::Dim,
    [TEXTURE_2D_MULTISAMPLE<MS> "Texture 2D Multisample"]; Renderable; GL32; <TEXTURE_2D as TextureType>::Dim,
    [TEXTURE_2D_MULTISAMPLE_ARRAY<MS> "Texture 2D Multisample Array"]; Renderable; GL32; <TEXTURE_2D_ARRAY as TextureType>::Dim,
}

//we can't really shoehorn in a phantom reference to a buffer in the macro, so
//we have to manually do the TEXTURE_BUFFER targets

#[allow(non_camel_case_types)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""), PartialEq, Eq, Hash, Default)]
pub struct TEXTURE_BUFFER<'a> { buf: PhantomData<Slice<'a, dyn std::any::Any, ReadOnly>> }

#[allow(non_camel_case_types)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""), PartialEq, Eq, Hash, Default)]
pub struct TEXTURE_BUFFER_MUT<'a> { buf: PhantomData<SliceMut<'a, dyn std::any::Any, ReadOnly>> }

impl<'a> Debug for TEXTURE_BUFFER<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result { write!(f, "TEXTURE_BUFFER") }
}
impl<'a> Debug for TEXTURE_BUFFER_MUT<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result { write!(f, "TEXTURE_BUFFER_MUT") }
}

impl<'a> Display for TEXTURE_BUFFER<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result { write!(f, "Texture Buffer") }
}
impl<'a> Display for TEXTURE_BUFFER_MUT<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result { write!(f, "Mutable Texture Buffer") }
}

impl<'a> From<TEXTURE_BUFFER<'a>> for GLenum { fn from(_:TEXTURE_BUFFER) -> GLenum {gl::TEXTURE_BUFFER} }
impl<'a> From<TEXTURE_BUFFER_MUT<'a>> for GLenum { fn from(_:TEXTURE_BUFFER_MUT) -> GLenum {gl::TEXTURE_BUFFER} }
impl<'a> TryFrom<GLenum> for TEXTURE_BUFFER<'a> {
    type Error = GLError;
    fn try_from(val:GLenum) -> Result<Self,GLError> {
        if val == gl::TEXTURE_BUFFER {
            Ok(Self::default())
        } else {
            Err(GLError::InvalidEnum(val,"".to_string()))
        }
    }
}
impl<'a> TryFrom<GLenum> for TEXTURE_BUFFER_MUT<'a> {
    type Error = GLError;
    fn try_from(val:GLenum) -> Result<Self,GLError> {
        if val == gl::TEXTURE_BUFFER {
            Ok(Self::default())
        } else {
            Err(GLError::InvalidEnum(val,"".to_string()))
        }
    }
}

impl<'a> GLEnum for TEXTURE_BUFFER<'a> {}
impl<'a> GLEnum for TEXTURE_BUFFER_MUT<'a> {}

unsafe impl<'a> TextureType for TEXTURE_BUFFER<'a> { type GL = GL31; type Dim = usize; }
unsafe impl<'a> TextureType for TEXTURE_BUFFER_MUT<'a> { type GL = GL31; type Dim = usize; }

impl<'a,F:ColorFormat> TextureTarget<F> for TEXTURE_BUFFER<'a> {}
impl<'a,F:ColorFormat> TextureTarget<F> for TEXTURE_BUFFER_MUT<'a> {}


//All but TEXTURE_BUFFER

unsafe impl Owned for TEXTURE_1D { }
unsafe impl Owned for TEXTURE_2D { }
unsafe impl Owned for TEXTURE_3D { }
unsafe impl Owned for TEXTURE_1D_ARRAY { }
unsafe impl Owned for TEXTURE_2D_ARRAY { }
unsafe impl Owned for TEXTURE_RECTANGLE { }
unsafe impl Owned for TEXTURE_CUBE_MAP {}
unsafe impl Owned for TEXTURE_CUBE_MAP_ARRAY {}
unsafe impl<MS:MultisampleFormat> Owned for TEXTURE_2D_MULTISAMPLE<MS> {}
unsafe impl<MS:MultisampleFormat> Owned for TEXTURE_2D_MULTISAMPLE_ARRAY<MS> {}

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
unsafe impl<MS:MultisampleFormat> BaseImage for TEXTURE_2D_MULTISAMPLE<MS> {}
unsafe impl<MS:MultisampleFormat> BaseImage for TEXTURE_2D_MULTISAMPLE_ARRAY<MS> {}

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
unsafe impl<MS:MultisampleFormat> Layered for TEXTURE_2D_MULTISAMPLE_ARRAY<MS> {}

unsafe impl CubeMapped for TEXTURE_CUBE_MAP {}
unsafe impl CubeMapped for TEXTURE_CUBE_MAP_ARRAY {}

unsafe impl<MS:MultisampleFormat> Multisampled for TEXTURE_2D_MULTISAMPLE<MS> {
    const SAMPLES: GLuint = MS::SAMPLES;
    const FIXED: bool = MS::FIXED;
}
unsafe impl<MS:MultisampleFormat> Multisampled for TEXTURE_2D_MULTISAMPLE_ARRAY<MS> {
    const SAMPLES: GLuint = MS::SAMPLES;
    const FIXED: bool = MS::FIXED;
}

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
