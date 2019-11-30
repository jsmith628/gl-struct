
use super::*;

pub use self::internal_format::*;
pub use self::client_format::*;

pub mod internal_format;
pub mod client_format;

pub unsafe trait Pixel<F: ClientFormat>: Copy+PartialEq {
    fn format() -> F;
    fn swap_bytes() -> bool;
    fn lsb_first() -> bool;
}

pub enum PixelPtr<F:ClientFormat> {
    Slice(F, *const GLvoid),
    Buffer(F, GLuint, *const GLvoid)
}

pub enum PixelPtrMut<F:ClientFormat> {
    Slice(F, *mut GLvoid),
    Buffer(F, GLuint, *mut GLvoid)
}
