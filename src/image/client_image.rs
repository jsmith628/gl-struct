use super::*;

//TODO: consider changing the dimension type to be an isize in order to better fit in line with
//use in the GL

#[derive(Clone,Copy)]
pub struct ClientImage<P:?Sized> {
    dim: [usize;3],
    pixels: P
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

impl<P:?Sized> ClientImage<P> {

    pub fn dim(&self) -> [usize; 3] { self.dim }

    pub fn width(&self) -> usize { self.dim()[0] }
    pub fn height(&self) -> usize { self.dim()[1] }
    pub fn depth(&self) -> usize { self.dim()[2] }

    pub fn as_ref(&self) -> ClientImage<&P> { ClientImage { dim:self.dim, pixels:&self.pixels } }
    pub fn as_mut(&mut self) -> ClientImage<&mut P> { ClientImage { dim:self.dim, pixels:&mut self.pixels } }

}

impl<P:PixelSrc+?Sized> ClientImage<P> {

    pub fn block_dim(&self) -> [usize; 3] { [self.block_width(), self.block_height(), self.block_depth()] }

    pub fn block_width(&self) -> usize { <P::Pixels as PixelData>::block_width() }
    pub fn block_height(&self) -> usize { <P::Pixels as PixelData>::block_height() }
    pub fn block_depth(&self) -> usize { <P::Pixels as PixelData>::block_depth() }

    pub fn block_size(&self) -> usize { <P::Pixels as PixelData>::block_size() }

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
        ClientSubImage::try_new(offset, dim, self.as_ref())
    }

    pub fn try_sub_image_mut(
        &mut self, offset: [usize;3], dim: [usize;3]
    ) -> Result<ClientSubImage<ClientImage<&mut P>>, SubImageError> {
        ClientSubImage::try_new(offset, dim, self.as_mut())
    }

    pub fn sub_image(&self, offset: [usize;3], dim: [usize;3]) -> ClientSubImage<ClientImage<&P>> {
        self.try_sub_image(offset, dim).unwrap()
    }

    pub fn sub_image_mut(&mut self, offset: [usize;3], dim: [usize;3]) -> ClientSubImage<ClientImage<&mut P>> {
        self.try_sub_image_mut(offset, dim).unwrap()
    }

}

//TODO: creation methods for compressed data
impl<P:PixelSrc> ClientImage<P> {

    pub fn try_new(dim: [usize;3], pixels: P) -> Result<Self,ImageError> {
        //compute the array size required to store that many pixels while making sure the value
        //does not overflow
        let count = dim[0].checked_mul(dim[1]).and_then(|m| m.checked_mul(dim[2]));

        //if we did not overflow
        if let Some(n) = count {

            //get a reference to the backing buffer of pixels and make sure
            //the buffer has the exact length required to store the pixels for this image
            let len = pixels.pixels().len();
            if n == len {

                let block = [
                    <P::Pixels as PixelData>::block_width(),
                    <P::Pixels as PixelData>::block_height(),
                    <P::Pixels as PixelData>::block_depth()
                ];

                //make sure the block dimensions perfectly divide the dimensions
                if block[0]%dim[0]==0 && block[0]%dim[0]==0 && block[0]%dim[0]==0 {
                    Ok(ClientImage { dim, pixels })
                } else {
                    Err(ImageError::NotBlockAligned(dim, block))
                }

            } else {
                Err(ImageError::InvalidDimensions(dim, len))
            }

        } else {
            Err(ImageError::SizeOverflow(dim))
        }
    }

    pub fn new(dim: [usize;3], pixels: P) -> Self {
        Self::try_new(dim, pixels).unwrap()
    }

}

impl<'a,P:?Sized,GL1:GLVersion> ClientImage<Pixels<'a,P,GL1>> {

    pub fn lock<GL2:Supports<GL1>>(self) -> ClientImage<Pixels<'a,P,GL2>> {
        ClientImage { dim: self.dim, pixels: self.pixels.lock() }
    }

    pub fn unlock<GL2:Supports<GL1>>(self, gl:&GL2) -> ClientImage<Pixels<'a,P,()>> {
        ClientImage { dim: self.dim, pixels: self.pixels.unlock(gl) }
    }

}

impl<'a,P:?Sized,GL1:GLVersion> ClientImage<PixelsMut<'a,P,GL1>> {

    pub fn lock<GL2:Supports<GL1>>(self) -> ClientImage<PixelsMut<'a,P,GL2>> {
        ClientImage { dim: self.dim, pixels: self.pixels.lock() }
    }

    pub fn unlock<GL2:Supports<GL1>>(self, gl:&GL2) -> ClientImage<PixelsMut<'a,P,()>> {
        ClientImage { dim: self.dim, pixels: self.pixels.unlock(gl) }
    }

}

//NOTE: we need to check the size of the internal buffer since _hypothetically_ some asshole
//_could_ modify the length of the backing image buffer to make it invalid and since we are going
//directly to openGL from here, an invalid buffer size could be EXTREMELY memory unsafe.

fn check_buffer_size(dim:[usize; 3], len: usize ) {
    let req_len: usize = dim.iter().product();
    if req_len != len {
        panic!("image pixel buffer length illegally modified from {} to {}.", req_len, len)
    }
}

impl<P:PixelSrc+?Sized> PixelSrc for ClientImage<P> {
    type Pixels = P::Pixels;
    type GL = P::GL;
    fn pixels(&self) -> Pixels<P::Pixels,P::GL> { self.pixels.pixels() }
}

impl<P:PixelDst+?Sized> PixelDst for ClientImage<P> {
    fn pixels_mut(&mut self) -> PixelsMut<P::Pixels,P::GL> { self.pixels.pixels_mut() }
}

impl<P:PixelSrc+?Sized> ImageSrc for ClientImage<P> {
    type Pixels = P::Pixels;
    type GL = P::GL;
    fn image(&self) -> ImageRef<P::Pixels,P::GL> {
        let (dim, pixels) = (self.dim(), self.pixels.pixels());
        check_buffer_size(dim, pixels.len());
        unsafe { ClientImage::new_unchecked(dim, pixels).into_sub_image([0,0,0], dim) }
    }
}

impl<P:PixelDst+?Sized> ImageDst for ClientImage<P> {
    fn image_mut(&mut self) -> ImageMut<P::Pixels,P::GL> {
        let (dim, pixels) = (self.dim(), self.pixels.pixels_mut());
        check_buffer_size(dim, pixels.len());
        unsafe { ClientImage::new_unchecked(dim, pixels).into_sub_image([0,0,0], dim) }
    }
}
