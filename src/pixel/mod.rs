
use super::*;
use crate::format::*;

pub use self::compressed::*;

pub mod compressed;

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
