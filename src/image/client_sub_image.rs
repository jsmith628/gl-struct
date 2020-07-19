use super::*;

#[derive(Clone,Copy)]
pub struct ClientSubImage<I:ImageSrc> {
    offset: [usize;3],
    dim: [usize;3],
    image: I
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SubImageError {
    OutOfBounds,
    NotBlockAligned
}

impl<I:ImageSrc> ClientSubImage<I> {

    pub unsafe fn new_unchecked(offset:[usize;3], dim:[usize;3], img: I) -> Self {
        ClientSubImage { offset: offset, dim: dim, image: img }
    }

    pub fn try_new(offset:[usize;3], dim:[usize;3], img: I) -> Result<Self, SubImageError> {

        trait BlockSize { fn _block_dim() -> [usize;3]; }
        impl<I> BlockSize for I { default fn _block_dim() -> [usize;3] {[1;3]} }
        impl<I:CompressedImageSrc> BlockSize for I {
            fn _block_dim() -> [usize;3] {
                [I::Format::block_width().into(), I::Format::block_height().into(), I::Format::block_depth().into()]
            }
        }

        let corner = [offset[0]+dim[0], offset[1]+dim[1], offset[2]+dim[2]];
        if corner[0]<img.width() && corner[1]<img.height() && corner[2]<img.depth() {
            let block = I::_block_dim();
            if dim[0]%block[0] == 0 && dim[1]%block[1] == 0 && dim[2]%block[2] == 0 &&
                offset[0]%block[0] == 0 && offset[1]%block[1] == 0 && offset[2]%block[2] == 0
            {
                unsafe {
                    Ok(ClientSubImage::new_unchecked(offset, dim, img))
                }
            } else {
                Err(SubImageError::NotBlockAligned)
            }
        } else {
            Err(SubImageError::OutOfBounds)
        }
    }

    pub fn new(offset:[usize;3], dim:[usize;3], img: I) -> Self {
        Self::try_new(offset, dim, img).unwrap()
    }

}


impl<I:UncompressedImage> UncompressedImage for ClientSubImage<I> { type Pixel = I::Pixel; }
impl<I:CompressedImage> CompressedImage for ClientSubImage<I> { type Format = I::Format; }

unsafe impl<I:ImageSrc> ImageSrc for ClientSubImage<I> {

    type Pixels = I::Pixels;

    fn swap_bytes(&self) -> bool { self.image.swap_bytes() }
    fn lsb_first(&self) -> bool { self.image.lsb_first() }
    fn row_alignment(&self) -> PixelRowAlignment { self.image.row_alignment() }

    fn skip_pixels(&self) -> usize { self.offset[0] }
    fn skip_rows(&self) -> usize { self.offset[1] }
    fn skip_images(&self) -> usize { self.offset[2] }

    fn row_length(&self) -> usize { if self.image.row_length()==0 {self.width()} else {self.image.row_length()} }
    fn image_height(&self) -> usize { if self.image.image_height()==0 {self.height()} else {self.image.image_height()} }

    fn width(&self) -> usize { self.dim[0] }
    fn height(&self) -> usize {  self.dim[1] }
    fn depth(&self) -> usize {  self.dim[2] }

    fn pixels(&self) -> PixelPtr<Self::Pixels> { self.image.pixels() }

}

unsafe impl<I:ImageDst> ImageDst for ClientSubImage<I> {
    fn pixels_mut(&mut self) -> PixelPtrMut<Self::Pixels> { self.image.pixels_mut() }
}
