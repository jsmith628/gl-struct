use super::*;
use crate::format::*;
use crate::pixels::*;
use crate::buffer::*;
use crate::version::*;

pub use self::client_image::*;
pub use self::client_sub_image::*;

mod client_image;
mod client_sub_image;
// mod impls;

use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;
use std::convert::TryInto;

pub type ImageRef<'a,P,GL> = ClientSubImage<ClientImage<Pixels<'a,P,GL>>>;
pub type ImageMut<'a,P,GL> = ClientSubImage<ClientImage<PixelsMut<'a,P,GL>>>;

pub trait ImageSrc {
    type Pixels: PixelData+?Sized;
    type GL: GLVersion;
    fn image(&self) -> ImageRef<Self::Pixels,Self::GL>;
}

pub trait ImageDst: ImageSrc {
    fn image_mut(&mut self) -> ImageMut<Self::Pixels,Self::GL>;
}
