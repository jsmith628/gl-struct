use super::*;

pub struct ClientImage<B:PixelSrc+?Sized> {
    dim: [usize;3],
    pixels: B
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ImageCreationError {
    SizeOverflow,
    InvalidDimensions([usize;3], usize),
}

impl ::std::fmt::Display for ImageCreationError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ImageCreationError::SizeOverflow => write!(f, "Overflow in computing buffer size"),
            ImageCreationError::InvalidDimensions(_,_) => write!(f, "Invalid dimensions for buffer size")
        }
    }
}


impl<B:PixelSrc> ClientImage<B> {
    pub unsafe fn new_unchecked(dim: [usize;3], pixels: B) -> Self {
        ClientImage { dim: dim, pixels: pixels }
    }
}

impl<P, B:PixelSrc<Pixels=[P]>> ClientImage<B> {

    pub fn try_new(dim: [usize;3], pixels: B) -> Result<Self,ImageCreationError> {
        let count = dim[0].checked_mul(dim[1]).and_then(|m| m.checked_mul(dim[2]));
        if let Some(n) = count {
            let len = pixels.pixel_ptr().len();
            if n!=len {
                Ok( unsafe {Self::new_unchecked(dim, pixels)} )
            } else {
                Err(ImageCreationError::InvalidDimensions(dim, len))
            }
        } else {
            Err(ImageCreationError::SizeOverflow)
        }
    }

    pub fn new(dim: [usize;3], pixels: B) -> Self {
        Self::try_new(dim, pixels).unwrap()
    }

}

impl<P, B:PixelSrc<Pixels=[P]>> ImageSrc for ClientImage<B> {

    type Pixel = P;

    fn swap_bytes(&self) -> bool {false}
    fn lsb_first(&self) -> bool {false}
    fn row_alignment(&self) -> PixelRowAlignment {PixelRowAlignment(1)}

    fn width(&self) -> usize {self.dim[0]}
    fn height(&self) -> usize {self.dim[1]}
    fn depth(&self) -> usize {self.dim[2]}

    fn dim(&self) -> [usize; 3] { self.dim }

    fn pixels(&self) -> PixelPtr<[P]> { self.pixels.pixel_ptr() }

}

impl<P, B:PixelDst<Pixels=[P]>> ImageDst for ClientImage<B> {
    fn pixels_mut(&mut self) -> PixelPtrMut<[P]> { self.pixels.pixel_ptr_mut() }
}

impl<P, B:FromPixels<Pixels=[P]>> OwnedImage for ClientImage<B> {

    type GL = B::GL;
    type Hint = B::Hint;

    unsafe fn from_gl<G:FnOnce(PixelStoreSettings, PixelPtrMut<[P]>)>(
        gl:&B::GL, hint:B::Hint, dim: [usize;3], get:G
    ) -> Self {
        let settings = Default::default();
        ClientImage {
            dim: dim, pixels: B::from_pixels(gl, hint, pixel_count(dim), |ptr| get(settings, ptr))
        }
    }
}
