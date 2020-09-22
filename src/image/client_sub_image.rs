use super::*;

#[derive(Clone,Copy)]
pub struct ClientSubImage<I:?Sized> {
    offset: [usize;3],
    dim: [usize;3],
    image: I
}


#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SubImageError {
    OutOfBounds([usize;3], [usize;3]),
    NotBlockAligned([usize;3], [usize;3]),
    GLVersion(GLVersionError)
}

impl ::std::fmt::Display for SubImageError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            SubImageError::OutOfBounds([x,y,z], [img_x,img_y,img_z]) => write!(
                f,
                "Subimage corner as {}x{}x{} out of bounds for image dimensions {}x{}x{}",
                 x,y,z, img_x,img_y,img_z
            ),
            SubImageError::NotBlockAligned([x,y,z], [b_x,b_y,b_z]) => write!(
                f, "Subimage dimensions {}x{}x{} not divisible by compressed block dimensions {}x{}x{}",
                x,y,z, b_x,b_y,b_z
            ),
            SubImageError::GLVersion(e) => write!(f, "{}", e)
        }
    }
}

impl<I:?Sized> ClientSubImage<I> {
    pub fn dim(&self) -> [usize; 3] { self.dim }

    pub fn width(&self) -> usize { self.dim()[0] }
    pub fn height(&self) -> usize { self.dim()[1] }
    pub fn depth(&self) -> usize { self.dim()[2] }

}

impl<I:ImageSrc> ClientSubImage<I> {

    pub unsafe fn new_unchecked(offset:[usize;3], dim:[usize;3], image: I) -> Self {
        ClientSubImage { offset, dim, image }
    }

    pub fn try_new(offset:[usize;3], dim:[usize;3], img: I) -> Result<Self, SubImageError> {

        trait BlockSize { fn _block_dim() -> [usize;3]; }
        impl<I> BlockSize for I { default fn _block_dim() -> [usize;3] {[1;3]} }
        impl<I:CompressedImageSrc> BlockSize for I {
            fn _block_dim() -> [usize;3] {
                [I::Format::block_width().into(), I::Format::block_height().into(), I::Format::block_depth().into()]
            }
        }

        let img_dim = img.image().dim();

        let corner = [offset[0]+dim[0], offset[1]+dim[1], offset[2]+dim[2]];
        if corner < img_dim {
            let block = I::_block_dim();
            if dim[0]%block[0] == 0 && dim[1]%block[1] == 0 && dim[2]%block[2] == 0 &&
                offset[0]%block[0] == 0 && offset[1]%block[1] == 0 && offset[2]%block[2] == 0
            {
                unsafe {
                    Ok(ClientSubImage::new_unchecked(offset, dim, img))
                }
            } else {
                Err(SubImageError::NotBlockAligned(dim, block))
            }
        } else {
            Err(SubImageError::OutOfBounds(corner, img_dim))
        }
    }

    pub fn new(offset:[usize;3], dim:[usize;3], img: I) -> Self {
        Self::try_new(offset, dim, img).unwrap()
    }

}


impl<I:UncompressedImage+?Sized> UncompressedImage for ClientSubImage<I> { type Pixel = I::Pixel; }
impl<I:CompressedImage+?Sized> CompressedImage for ClientSubImage<I> { type Format = I::Format; }

impl<I:ImageSrc+?Sized> ImageSrc for ClientSubImage<I> {
    type Pixels = I::Pixels;
    type GL = I::GL;
    fn image(&self) -> ImageRef<Self::Pixels,Self::GL> { unimplemented!() }
}

impl<I:ImageDst+?Sized> ImageDst for ClientSubImage<I> {
    fn image_mut(&mut self) -> ImageMut<Self::Pixels,Self::GL> { unimplemented!() }
}
