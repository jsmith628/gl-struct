use super::*;

pub struct ClientImage<B:PixelSrc+?Sized> {
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
            ImageError::InvalidDimensions(_,_) => write!(f, "Invalid dimensions for buffer size"),
            ImageError::NotBlockAligned => write!(f, "Image dimensions not divisible by compressed block dimensions")
        }
    }
}


impl<B:PixelSrc> ClientImage<B> {
    pub unsafe fn new_unchecked(dim: [usize;3], pixels: B) -> Self {
        ClientImage { dim: dim, pixels: pixels }
    }
}

impl<P, B:PixelSrc<Pixels=[P]>> ClientImage<B> {

    pub fn try_new(dim: [usize;3], pixels: B) -> Result<Self,ImageError> {
        let count = dim[0].checked_mul(dim[1]).and_then(|m| m.checked_mul(dim[2]));
        if let Some(n) = count {
            let len = pixels.pixel_ptr().len();
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

unsafe impl<B:PixelSrc> ImageSrc for ClientImage<B> {

    type Pixels = B::Pixels;

    fn swap_bytes(&self) -> bool {false}
    fn lsb_first(&self) -> bool {false}
    fn row_alignment(&self) -> PixelRowAlignment {PixelRowAlignment(1)}

    fn row_length(&self) -> usize {self.width()}
    fn image_height(&self) -> usize {self.height()}

    fn width(&self) -> usize {self.dim[0]}
    fn height(&self) -> usize {self.dim[1]}
    fn depth(&self) -> usize {self.dim[2]}

    fn skip_pixels(&self) -> usize {0}
    fn skip_rows(&self) -> usize {0}
    fn skip_images(&self) -> usize {0}

    fn pixels(&self) -> PixelPtr<Self::Pixels> { self.pixels.pixel_ptr() }

}

unsafe impl<B:PixelDst> ImageDst for ClientImage<B> {
    fn pixels_mut(&mut self) -> PixelPtrMut<Self::Pixels> { self.pixels.pixel_ptr_mut() }
}

unsafe impl<B:FromPixels> OwnedImage for ClientImage<B> {

    type GL = B::GL;
    type Hint = B::Hint;

    unsafe fn from_gl<G:FnOnce(PixelStore, PixelPtrMut<Self::Pixels>)>(
        gl:&B::GL, hint:B::Hint, dim: [usize;3], get:G
    ) -> Self {
        let settings = Default::default();
        ClientImage {
            dim: dim, pixels: B::from_pixels(gl, hint, pixel_count(dim), |ptr| get(settings, ptr))
        }
    }
}
