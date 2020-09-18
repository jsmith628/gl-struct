use super::*;

#[derive(Clone,Copy)]
pub struct ClientImage<B:?Sized> {
    dim: [usize;3],
    pixels: B
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ImageError {
    SizeOverflow,
    InvalidDimensions([usize;3], usize),
    NotBlockAligned,
}

impl ::std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ImageError::SizeOverflow => write!(f, "Overflow in computing buffer size"),
            ImageError::InvalidDimensions([x,y,z],s) => write!(
                f,
                "Invalid dimensions for buffer size. a {}x{}x{} image requires a
                 {} byte buffer but a {} byte was given instead",
                 x, y, z, x*y*z, s
            ),
            ImageError::NotBlockAligned => write!(f, "Image dimensions not divisible by compressed block dimensions")
        }
    }
}


impl<B> ClientImage<B> {

    pub unsafe fn new_unchecked(dim: [usize;3], pixels: B) -> Self {
        ClientImage { dim, pixels }
    }

    pub fn dim(&self) -> [usize; 3] { self.dim }

    pub fn width(&self) -> usize { self.dim()[0] }
    pub fn height(&self) -> usize { self.dim()[1] }
    pub fn depth(&self) -> usize { self.dim()[2] }

}

//TODO: creation methods for compressed data
//TODO: add support for images with Buffers
impl<P, B:PixelSrc<Pixels=[P],GL=()>> ClientImage<B> {

    pub fn try_new(dim: [usize;3], pixels: B) -> Result<Self,ImageError> {
        let count = dim[0].checked_mul(dim[1]).and_then(|m| m.checked_mul(dim[2]));
        if let Some(n) = count {
            let len = pixels.pixels(()).len();
            if n!=len {
                Ok( unsafe {Self::new_unchecked(dim, pixels)} )
            } else {
                Err(ImageError::InvalidDimensions(dim, len))
            }
        } else {
            Err(ImageError::SizeOverflow)
        }
    }

    pub fn new(dim: [usize;3], pixels: B) -> Self {
        Self::try_new(dim, pixels).unwrap()
    }

}

impl<P,B:PixelSrc<Pixels=[P]>> UncompressedImage for ClientImage<B> { type Pixel = P; }
impl<F:SpecificCompressed,B:PixelSrc<Pixels=CompressedPixels<F>>> CompressedImage for ClientImage<B> {
    type Format = F;
}

impl<B:PixelSrc> ImageSrc for ClientImage<B> {
    type Pixels = B::Pixels;
    type GL = B::GL;
    fn image(&self, gl:&Self::GL) -> ImagePtr<Self::Pixels> { unimplemented!() }
}

impl<B:PixelDst> ImageDst for ClientImage<B> {
    fn image_mut(&mut self, gl:&Self::GL) -> ImagePtrMut<Self::Pixels> { unimplemented!() }
}

unsafe impl<B:FromPixels> OwnedImage for ClientImage<B> {
    type Hint = B::Hint;

    unsafe fn from_gl<G:FnOnce(PixelStore, PixelsMut<Self::Pixels>)>(
        gl:&B::GL, hint:B::Hint, dim: [usize;3], get:G
    ) -> Self {
        let settings = Default::default();
        ClientImage {
            dim, pixels: B::from_pixels(gl, hint, pixel_count(dim), |ptr| get(settings, ptr))
        }
    }
}
