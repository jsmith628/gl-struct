
use super::*;
use crate::format::*;

pub use self::compressed::*;

pub mod compressed;

pub unsafe trait Pixel<F: PixelLayout>: Copy+PartialEq {
    fn format() -> F;
    fn swap_bytes() -> bool {false}
    fn lsb_first() -> bool {false}
}

#[derive(Clone, Copy)]
pub enum PixelPtr<P:?Sized> {
    Slice(*const P),
    Buffer(GLuint, *const P)
}

impl<P:?Sized> PixelPtr<P> {
    pub fn size(self) -> usize {
        match self {
            Self::Slice(ptr) => unsafe { ::std::mem::size_of_val(&*ptr) },
            Self::Buffer(_,ptr) => unsafe { ::std::mem::size_of_val(&*ptr) },
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl<P> PixelPtr<[P]> {
    pub fn len(self) -> usize {
        match self {
            Self::Slice(ptr) => unsafe { (&*ptr).len() },
            Self::Buffer(_,ptr) => unsafe { (&*ptr).len() },
        }
    }
}

#[derive(Clone, Copy)]
pub enum PixelPtrMut<P:?Sized> {
    Slice(*mut P),
    Buffer(GLuint, *mut P)
}

impl<P:?Sized> PixelPtrMut<P> {
    pub fn size(self) -> usize {
        match self {
            Self::Slice(ptr) => unsafe { ::std::mem::size_of_val(&*ptr) },
            Self::Buffer(_,ptr) => unsafe { ::std::mem::size_of_val(&*ptr) },
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl<P> PixelPtrMut<[P]> {
    pub fn len(self) -> usize {
        match self {
            Self::Slice(ptr) => unsafe { (&*ptr).len() },
            Self::Buffer(_,ptr) => unsafe { (&*ptr).len() },
        }
    }
}

macro_rules! impl_int {
    ($($prim:ident $ty:ident)*) => {
        $(
            impl_int!(@impl $prim; RED RED_INTEGER $ty);
            impl_int!(@impl [$prim;1]; RED RED_INTEGER $ty);
            impl_int!(@impl [$prim;2]; RG RG_INTEGER $ty);
            impl_int!(@impl [$prim;3]; RGB RGB_INTEGER $ty);
            impl_int!(@impl [$prim;4]; RGBA RGBA_INTEGER $ty);

            unsafe impl Pixel<DepthLayout> for $prim {
                fn format() -> DepthLayout { IntType::$ty.into() }
            }
            unsafe impl Pixel<DepthLayout> for [$prim;1] {
                fn format() -> DepthLayout { IntType::$ty.into() }
            }
            unsafe impl Pixel<StencilLayout> for $prim {
                fn format() -> StencilLayout { IntType::$ty.into() }
            }
            unsafe impl Pixel<StencilLayout> for [$prim;1] {
                fn format() -> StencilLayout { IntType::$ty.into() }
            }

        )*
    };

    (@impl $prim:ty; $fmt1:ident $fmt2:ident $ty:ident) => {
        unsafe impl Pixel<IntLayout> for $prim {
            fn format() -> IntLayout { IntLayout::Integer(IntColorComponents::$fmt2, IntType::$ty) }
        }
        unsafe impl Pixel<FloatLayout> for $prim {
            fn format() -> FloatLayout { FloatLayout::Normalized(ColorComponents::$fmt1, IntType::$ty) }
        }
    };

}

impl_int!{
    GLbyte Byte
    GLubyte UByte
    GLshort Short
    GLushort UShort
    GLint Int
    GLuint UInt
}

macro_rules! impl_float {
    ($($prim:ident $ty:ident)*) => {
        $(
            impl_float!(@impl $prim; RED $ty);
            impl_float!(@impl [$prim;1]; RED $ty);
            impl_float!(@impl [$prim;2]; RG $ty);
            impl_float!(@impl [$prim;3]; RGB $ty);
            impl_float!(@impl [$prim;4]; RGBA $ty);

            unsafe impl Pixel<DepthLayout> for $prim {
                fn format() -> DepthLayout { FloatType::$ty.into() }
            }
            unsafe impl Pixel<DepthLayout> for [$prim;1] {
                fn format() -> DepthLayout { FloatType::$ty.into() }
            }

            unsafe impl Pixel<DepthStencilLayout> for $prim {
                fn format() -> DepthStencilLayout { FloatType::$ty.into() }
            }
            unsafe impl Pixel<DepthStencilLayout> for [$prim;1] {
                fn format() -> DepthStencilLayout { FloatType::$ty.into() }
            }

        )*
    };

    (@impl $prim:ty; $fmt:ident $ty:ident) => {
        unsafe impl Pixel<FloatLayout> for $prim {
            fn format() -> FloatLayout { FloatLayout::Float(ColorComponents::$fmt, FloatType::$ty) }
        }
    };

}

impl_float!(GLfloat Float);

macro_rules! impl_ivec {
    ($($vec:ident $inner:ty),*) => {
        $(
            unsafe impl Pixel<IntLayout> for $vec {
                fn format() -> IntLayout { <$inner as Pixel<IntLayout>>::format() }
            }
        )*
    }
}

macro_rules! impl_vec {
    ($($vec:ident $inner:ty),*) => {
        $(
            unsafe impl Pixel<FloatLayout> for $vec {
                fn format() -> FloatLayout { <$inner as Pixel<FloatLayout>>::format() }
            }
        )*
    }
}

use glsl::*;

impl_ivec!(ivec2 [GLint;2], ivec4 [GLint;4], uvec2 [GLuint;2], uvec4 [GLuint;4]);
impl_vec!{
    ivec2 [GLint;  2], ivec4 [GLint;  4],
    uvec2 [GLuint; 2], uvec4 [GLuint; 4],
     vec2 [GLfloat;2],  vec4 [GLfloat;4]
}
