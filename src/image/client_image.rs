use super::*;

#[derive(Clone,Copy)]
pub struct ClientImage<B:?Sized> {
    dim: [usize;3],
    pixels: B
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ImageError {
    SizeOverflow([usize;3]),
    InvalidDimensions([usize;3], usize),
    NotBlockAligned([usize;3], [usize;3])
}

impl ::std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ImageError::SizeOverflow([x,y,z]) => write!(
                f, "Overflow in computing buffer size for a {}x{}x{} image", x, y, z
            ),
            ImageError::InvalidDimensions([x,y,z], buffer_size) => write!(
                f,
                "Invalid dimensions for buffer size. a {}x{}x{} image requires a
                 {} byte buffer but a {} byte was given instead",
                 x, y, z, x*y*z, buffer_size
            ),
            ImageError::NotBlockAligned([x,y,z], [b_x,b_y,b_z]) => write!(
                f, "Image dimensions {}x{}x{} not divisible by compressed block dimensions {}x{}x{}",
                x,y,z, b_x,b_y,b_z
            )
        }
    }
}


impl<P> ClientImage<P> {
    pub unsafe fn new_unchecked(dim: [usize;3], pixels: P) -> Self {
        ClientImage { dim, pixels }
    }
}

impl<B:?Sized> ClientImage<B> {

    pub fn dim(&self) -> [usize; 3] { self.dim }

    pub fn width(&self) -> usize { self.dim()[0] }
    pub fn height(&self) -> usize { self.dim()[1] }
    pub fn depth(&self) -> usize { self.dim()[2] }

    pub fn block_dim(&self) -> [usize; 3] {

        //get the block size for compressed pixels and [1,1,1] for everything else
        trait BlockSize { fn _block_dim() -> [usize;3]; }
        impl<P:?Sized> BlockSize for P { default fn _block_dim() -> [usize;3] {[1;3]} }
        impl<F:SpecificCompressed, P:PixelSrc<Pixels=CompressedPixels<F>>> BlockSize for P {
            fn _block_dim() -> [usize;3] {
                [F::block_width().into(), F::block_height().into(), F::block_depth().into()]
            }
        }

        B::_block_dim()

    }

    pub fn block_width(&self) -> usize { self.block_dim()[0] }
    pub fn block_height(&self) -> usize { self.block_dim()[1] }
    pub fn block_depth(&self) -> usize { self.block_dim()[2] }

    pub fn borrow(&self) -> ClientImage<&B> { ClientImage { dim:self.dim, pixels:&self.pixels } }
    pub fn borrow_mut(&mut self) -> ClientImage<&mut B> { ClientImage { dim:self.dim, pixels:&mut self.pixels } }

}

impl<P:PixelSrc> ClientImage<P> {

    pub fn try_into_sub_image(
        self, offset: [usize;3], dim: [usize;3]
    ) -> Result<ClientSubImage<ClientImage<P>>, SubImageError> {
        ClientSubImage::try_new(offset, dim, self)
    }

    pub fn into_sub_image(self, offset: [usize;3], dim: [usize;3]) -> ClientSubImage<ClientImage<P>> {
        self.try_into_sub_image(offset, dim).unwrap()
    }

}

impl<P:PixelSrc+?Sized> ClientImage<P> {

    pub fn try_sub_image(
        &self, offset: [usize;3], dim: [usize;3]
    ) -> Result<ClientSubImage<ClientImage<&P>>, SubImageError> {
        ClientSubImage::try_new(offset, dim, self.borrow())
    }

    pub fn try_sub_image_mut(
        &mut self, offset: [usize;3], dim: [usize;3]
    ) -> Result<ClientSubImage<ClientImage<&mut P>>, SubImageError> {
        ClientSubImage::try_new(offset, dim, self.borrow_mut())
    }

    pub fn sub_image(&self, offset: [usize;3], dim: [usize;3]) -> ClientSubImage<ClientImage<&P>> {
        self.try_sub_image(offset, dim).unwrap()
    }

    pub fn sub_image_mut(&mut self, offset: [usize;3], dim: [usize;3]) -> ClientSubImage<ClientImage<&mut P>> {
        self.try_sub_image_mut(offset, dim).unwrap()
    }

}

//TODO: creation methods for compressed data
impl<P, B:PixelSrc<Pixels=[P]>> ClientImage<B> {

    pub fn try_new(dim: [usize;3], pixels: B) -> Result<Self,ImageError> {
        //compute the array size required to store that many pixels while making sure the value
        //does not overflow
        let count = dim[0].checked_mul(dim[1]).and_then(|m| m.checked_mul(dim[2]));

        //if we did not overflow
        if let Some(n) = count {

            //get a reference to the backing slice or GL buffer and make sure it has the exact
            //length required to store the pixels for this image
            let len = pixels.pixels().len();
            if n==len {
                Ok( unsafe {Self::new_unchecked(dim, pixels)} )
            } else {
                Err(ImageError::InvalidDimensions(dim, len))
            }

        } else {
            Err(ImageError::SizeOverflow(dim))
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



//a trait for getting the number of pixels in the backing buffer so that we can check
//if it was changed. This is necessary since PixelSrc technically doesn't require having a
//length method, so we have to specifically check for the cases of [P] and CompressedPixels.
//This shouldn't be an issue though since those are currently the only ones that can even
//be sent to OpenGL atm
trait Len { fn _len(&self) -> Option<usize>; }

impl<'a,P:?Sized,GL> Len for Pixels<'a,P,GL> { default fn _len(&self) -> Option<usize> { None } }
impl<'a,P:?Sized,GL> Len for PixelsMut<'a,P,GL> { default fn _len(&self) -> Option<usize> { None } }

impl<'a,P,GL> Len for Pixels<'a,[P],GL> { fn _len(&self) -> Option<usize> { Some(self.len()) } }
impl<'a,P,GL> Len for PixelsMut<'a,[P],GL> { fn _len(&self) -> Option<usize> { Some(self.len()) } }

impl<'a,F:SpecificCompressed,GL> Len for Pixels<'a,CompressedPixels<F>,GL> {
    fn _len(&self) -> Option<usize> { Some(self.len()) }
}
impl<'a,F:SpecificCompressed,GL> Len for PixelsMut<'a,CompressedPixels<F>,GL> {
    fn _len(&self) -> Option<usize> { Some(self.len()) }
}

//NOTE: we need to check the size of the internal buffer since _hypothetically_ some asshole
//_could_ modify the length of the backing image buffer to make it invalid and since we are going
//directly to openGL from here, an invalid buffer size could be EXTREMELY memory unsafe.

fn check_buffer_size(dim:[usize; 3], len: Option<usize> ) {
    if let Some(len) = len {
        let req_len: usize = dim.iter().product();
        if req_len != len {
            panic!("image pixel buffer length illegally modified from {} to {}.", req_len, len)
        }
    }
}

impl<B:PixelSrc+?Sized> ImageSrc for ClientImage<B> {
    type Pixels = B::Pixels;
    type GL = B::GL;
    fn image(&self) -> ImageRef<Self::Pixels,Self::GL> {
        let (dim, pixels) = (self.dim(), self.pixels.pixels());
        check_buffer_size(dim, pixels._len());
        unsafe { ClientImage::new_unchecked(dim, pixels).into_sub_image([0,0,0], dim) }
    }
}

impl<B:PixelDst+?Sized> ImageDst for ClientImage<B> {
    fn image_mut(&mut self) -> ImageMut<Self::Pixels,Self::GL> {
        let (dim, pixels) = (self.dim(), self.pixels.pixels_mut());
        check_buffer_size(dim, pixels._len());
        unsafe { ClientImage::new_unchecked(dim, pixels).into_sub_image([0,0,0], dim) }
    }
}

unsafe impl<B:FromPixels> OwnedImage for ClientImage<B> {
    type Hint = B::Hint;

    unsafe fn from_gl<G:FnOnce(PixelStore, PixelsMut<Self::Pixels,Self::GL>)>(
        gl:&B::GL, hint:B::Hint, dim: [usize;3], get:G
    ) -> Self {
        let settings = Default::default();
        ClientImage {
            dim, pixels: B::from_pixels(gl, hint, pixel_count(dim), |ptr| get(settings, ptr))
        }
    }
}
