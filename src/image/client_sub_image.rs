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
    NotBlockAligned([usize;3], [usize;3])
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
                f, "Subimage corner lies at {}x{}x{} on the base image which us not divisible by
                    the block dimensions of the current pixel format: {}x{}x{}",
                x,y,z, b_x,b_y,b_z
            )
        }
    }
}

fn check_bounds(
    offset:[usize;3], dim:[usize;3], img_dim:[usize;3], block:[usize;3]
) -> Result<(), SubImageError> {
    let corner = [offset[0]+dim[0], offset[1]+dim[1], offset[2]+dim[2]];
    if corner < img_dim {
        if offset[0]%block[0] == 0 && offset[1]%block[1] == 0 && offset[2]%block[2] == 0 {
            if corner[0]%block[0] == 0 && corner[1]%block[1] == 0 && corner[2]%block[2] == 0 {
                Ok(())
            } else {
                Err(SubImageError::NotBlockAligned(corner, block))
            }
        } else {
            Err(SubImageError::NotBlockAligned(offset, block))
        }

    } else {
        Err(SubImageError::OutOfBounds(corner, img_dim))
    }
}

impl<I:ImageSrc> ClientSubImage<I> {

    pub unsafe fn new_unchecked(offset:[usize;3], dim:[usize;3], image: I) -> Self {
        ClientSubImage { offset, dim, image }
    }

    pub fn try_new(offset:[usize;3], dim:[usize;3], img: I) -> Result<Self, SubImageError> {
        check_bounds(offset, dim, img.image().dim(), img.image().block_dim()).map(
            |_| unsafe { ClientSubImage::new_unchecked(offset, dim, img) }
        )
    }

    pub fn new(offset:[usize;3], dim:[usize;3], img: I) -> Self {
        Self::try_new(offset, dim, img).unwrap()
    }

}

impl<I:?Sized> ClientSubImage<I> {

    pub fn offset(&self) -> [usize; 3] { self.offset }

    pub fn offset_x(&self) -> usize { self.offset()[0] }
    pub fn offset_y(&self) -> usize { self.offset()[1] }
    pub fn offset_z(&self) -> usize { self.offset()[2] }

    pub fn dim(&self) -> [usize; 3] { self.dim }

    pub fn width(&self) -> usize { self.dim()[0] }
    pub fn height(&self) -> usize { self.dim()[1] }
    pub fn depth(&self) -> usize { self.dim()[2] }

}

trait InnerImg {
    fn _base_dim(&self) -> [usize; 3];
    fn _block_dim(&self) -> [usize; 3];
}

impl<I:ImageSrc+?Sized> InnerImg for ClientSubImage<I> {
    default fn _base_dim(&self) -> [usize; 3] { self.image()._base_dim() }
    default fn _block_dim(&self) -> [usize; 3] { self.image()._block_dim() }
}

impl<P:PixelSrc+?Sized> InnerImg for ClientSubImage<ClientImage<P>> where ClientImage<P>: ImageSrc {
    fn _base_dim(&self) -> [usize; 3] { self.image.dim() }
    fn _block_dim(&self) -> [usize; 3] { self.image.block_dim() }
}

impl<I:ImageSrc+?Sized> ClientSubImage<I> {

    pub fn base_dim(&self) -> [usize; 3] { self._base_dim() }

    pub fn base_width(&self) -> usize { self.base_dim()[0] }
    pub fn base_height(&self) -> usize { self.base_dim()[1] }
    pub fn base_depth(&self) -> usize { self.base_dim()[2] }

    pub fn block_dim(&self) -> [usize; 3] { self._block_dim() }

    pub fn block_width(&self) -> usize { self.block_dim()[0] }
    pub fn block_height(&self) -> usize { self.block_dim()[1] }
    pub fn block_depth(&self) -> usize { self.block_dim()[2] }

}


impl<I:UncompressedImage+?Sized> UncompressedImage for ClientSubImage<I> { type Pixel = I::Pixel; }
impl<I:CompressedImage+?Sized> CompressedImage for ClientSubImage<I> { type Format = I::Format; }

impl<I:ImageSrc+?Sized> ImageSrc for ClientSubImage<I> {
    type Pixels = I::Pixels;
    type GL = I::GL;
    fn image(&self) -> ImageRef<Self::Pixels,Self::GL> {
        let img = self.image();
        let (offset1, dim, offset2) = (self.offset(), self.dim(), img.offset());
        let offset = [offset1[0]+offset2[0], offset1[1]+offset2[1], offset1[2]+offset2[2]];
        check_bounds(offset, dim, img.dim(), img.block_dim()).map(
            |_| ClientSubImage {
                offset, dim: img.dim(), image: img.image
            }
        ).unwrap()
    }
}

impl<I:ImageDst+?Sized> ImageDst for ClientSubImage<I> {
    fn image_mut(&mut self) -> ImageMut<Self::Pixels,Self::GL> {
        let (offset1, dim) = (self.offset(), self.dim());
        let img = self.image_mut();
        let offset2 = img.offset();
        let offset = [offset1[0]+offset2[0], offset1[1]+offset2[1], offset1[2]+offset2[2]];
        check_bounds(offset, dim, img.dim(), img.block_dim()).map(
            |_| ClientSubImage {
                offset, dim: img.dim(), image: img.image
            }
        ).unwrap()
    }
}
