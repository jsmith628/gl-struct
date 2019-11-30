use super::*;

use std::fmt::{Debug, Formatter};

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub struct InvalidPixelRowAlignment(pub(crate) u8);
display_from_debug!(InvalidPixelRowAlignment);

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub struct PixelRowAlignment(pub(crate) u8);

display_from_debug!(PixelRowAlignment);
impl Debug for PixelRowAlignment {
    #[inline] fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result { write!(f, "{}", self.0) }
}

pub const ALIGN_1: PixelRowAlignment = PixelRowAlignment(1);
pub const ALIGN_2: PixelRowAlignment = PixelRowAlignment(2);
pub const ALIGN_4: PixelRowAlignment = PixelRowAlignment(4);
pub const ALIGN_8: PixelRowAlignment = PixelRowAlignment(8);

impl Into<u8> for PixelRowAlignment { #[inline] fn into(self) -> u8 {self.0} }

impl TryFrom<u8> for PixelRowAlignment {
    type Error = InvalidPixelRowAlignment;
    #[inline] fn try_from(a:u8) -> Result<Self,Self::Error> {
        match a {
            1 | 2 | 4 | 8 => Ok(PixelRowAlignment(a)),
            _ => Err(InvalidPixelRowAlignment(a))
        }
    }
}
