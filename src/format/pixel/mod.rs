
use super::*;

pub use self::internal_format::*;
pub use self::client_format::*;
pub use self::compressed::*;

pub mod internal_format;
pub mod client_format;
pub mod compressed;

pub unsafe trait Pixel<F: ClientFormat>: Copy+PartialEq {
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
            impl_int!(@impl $prim; RED_INTEGER $ty);
            impl_int!(@impl [$prim;1]; RED_INTEGER $ty);
            impl_int!(@impl [$prim;2]; RG_INTEGER $ty);
            impl_int!(@impl [$prim;3]; RGB_INTEGER $ty);
            impl_int!(@impl [$prim;4]; RGBA_INTEGER $ty);

            unsafe impl Pixel<ClientFormatDepth> for $prim {
                fn format() -> ClientFormatDepth { IntType::$ty.into() }
            }
            unsafe impl Pixel<ClientFormatDepth> for [$prim;1] {
                fn format() -> ClientFormatDepth { IntType::$ty.into() }
            }
            unsafe impl Pixel<ClientFormatStencil> for $prim {
                fn format() -> ClientFormatStencil { IntType::$ty.into() }
            }
            unsafe impl Pixel<ClientFormatStencil> for [$prim;1] {
                fn format() -> ClientFormatStencil { IntType::$ty.into() }
            }

        )*
    };

    (@impl $prim:ty; $fmt:ident $ty:ident) => {
        unsafe impl Pixel<ClientFormatInt> for $prim {
            fn format() -> ClientFormatInt { ClientFormatInt::Integer(FormatInt::$fmt, IntType::$ty) }
        }
        unsafe impl Pixel<ClientFormatFloat> for $prim {
            fn format() -> ClientFormatFloat { <Self as Pixel<ClientFormatInt>>::format().into() }
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

            unsafe impl Pixel<ClientFormatDepth> for $prim {
                fn format() -> ClientFormatDepth { FloatType::$ty.into() }
            }
            unsafe impl Pixel<ClientFormatDepth> for [$prim;1] {
                fn format() -> ClientFormatDepth { FloatType::$ty.into() }
            }

            unsafe impl Pixel<ClientFormatDepthStencil> for $prim {
                fn format() -> ClientFormatDepthStencil { FloatType::$ty.into() }
            }
            unsafe impl Pixel<ClientFormatDepthStencil> for [$prim;1] {
                fn format() -> ClientFormatDepthStencil { FloatType::$ty.into() }
            }

        )*
    };

    (@impl $prim:ty; $fmt:ident $ty:ident) => {
        unsafe impl Pixel<ClientFormatFloat> for $prim {
            fn format() -> ClientFormatFloat { ClientFormatFloat::Float(FormatFloat::$fmt, FloatType::$ty) }
        }
    };

}

impl_float!(GLfloat Float);

macro_rules! impl_ivec {
    ($($vec:ident $inner:ty),*) => {
        $(
            unsafe impl Pixel<ClientFormatInt> for $vec {
                fn format() -> ClientFormatInt { <$inner as Pixel<ClientFormatInt>>::format() }
            }
        )*
    }
}

macro_rules! impl_vec {
    ($($vec:ident $inner:ty),*) => {
        $(
            unsafe impl Pixel<ClientFormatFloat> for $vec {
                fn format() -> ClientFormatFloat { <$inner as Pixel<ClientFormatFloat>>::format() }
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
