
use super::*;
use buffer_new::{UninitBuf};
use std::mem::*;

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub struct PixelRowAlignment(pub(super) u8);

display_from_debug!(PixelRowAlignment);
impl ::std::fmt::Debug for PixelRowAlignment {
    #[inline]
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub const ALIGN_1: PixelRowAlignment = PixelRowAlignment(1);
pub const ALIGN_2: PixelRowAlignment = PixelRowAlignment(2);
pub const ALIGN_4: PixelRowAlignment = PixelRowAlignment(4);
pub const ALIGN_8: PixelRowAlignment = PixelRowAlignment(8);

impl From<u8> for PixelRowAlignment {
    #[inline] fn from(a:u8) -> Self {
        let mut shift = a;
        let mut count = 0;
        while shift!=0 {
            shift >>= 1;
            count += 1;
        }
        PixelRowAlignment(1<<count)
    }
}

pub unsafe trait PixelData<F:PixelFormatType> {

    #[inline] fn swap_bytes(&self) -> bool {false}
    #[inline] fn lsb_first(&self) -> bool {false}

    #[inline] fn alignment(&self) -> PixelRowAlignment {ALIGN_4}

    fn format_type(&self) -> F;
    fn len(&self) -> usize;

    fn bind_pixel_buffer<'a>(&'a self, target:&'a mut BindingLocation<UninitBuf>) -> Option<Binding<'a,UninitBuf>>;
    unsafe fn pixels(&self) -> *const GLvoid;
}

pub unsafe trait PixelDataMut<F:PixelFormatType> {
    unsafe fn pixels_mut(&mut self) -> *mut GLvoid;
}

pub unsafe trait PixelType<F: PixelFormatType>: Sized+Copy+Clone+PartialEq {
    fn format_type() -> F;
    fn swap_bytes() -> bool;
    fn lsb_first() -> bool;
}

unsafe impl<F:PixelFormatType,T:PixelType<F>> PixelData<F> for [T] {
    #[inline] fn swap_bytes(&self) -> bool {T::swap_bytes()}
    #[inline] fn lsb_first(&self) -> bool {T::lsb_first()}

    #[inline] fn alignment(&self) -> PixelRowAlignment { PixelRowAlignment(align_of::<T>().min(8) as u8) }

    #[inline] fn format_type(&self) -> F {T::format_type()}
    #[inline] fn len(&self) -> usize {self.len()}

    #[inline]
    fn bind_pixel_buffer<'a>(&'a self, _target:&'a mut BindingLocation<UninitBuf>) -> Option<Binding<'a,UninitBuf>> {
        None
    }

    #[inline] unsafe fn pixels(&self) -> *const GLvoid {&self[0] as *const T as *const GLvoid}
}

unsafe impl<F:PixelFormatType,T:PixelType<F>> PixelDataMut<F> for [T] {
    #[inline] unsafe fn pixels_mut(&mut self) -> *mut GLvoid {&mut self[0] as *mut T as *mut GLvoid}
}
