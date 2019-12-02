use super::*;

pub struct ClientSubImage<I:ImageSrc> {
    offset: [usize;3],
    dim: [usize;3],
    image: I
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SubImageOutOfBounds {
    offset: [usize;3],
    dim: [usize;3],
    bounds: [usize;3]
}

impl<I:ImageSrc> ClientSubImage<I> {

    pub unsafe fn new_unchecked(offset:[usize;3], dim:[usize;3], img: I) -> Self {
        ClientSubImage { offset: offset, dim: dim, image: img }
    }

    pub fn try_new(offset:[usize;3], dim:[usize;3], img: I) -> Result<Self, SubImageOutOfBounds> {
        let corner = [offset[0]+dim[0], offset[1]+dim[1], offset[2]+dim[2]];
        if corner[0]<img.width() && corner[1]<img.height() && corner[2]<img.depth() {
            unsafe {
                Ok(ClientSubImage::new_unchecked(offset, dim, img))
            }
        } else {
            Err(SubImageOutOfBounds { offset: offset, dim: dim, bounds: img.dim() } )
        }
    }

    pub fn new(offset:[usize;3], dim:[usize;3], img: I) -> Self {
        Self::try_new(offset, dim, img).unwrap()
    }

}

unsafe impl<I:ImageSrc> ImageSrc for ClientSubImage<I> {

    type Pixel = I::Pixel;

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

    fn dim(&self) -> [usize; 3] { self.dim }

    fn pixels(&self) -> PixelPtr<[Self::Pixel]> { self.image.pixels() }

}

unsafe impl<I:ImageDst> ImageDst for ClientSubImage<I> {
    fn pixels_mut(&mut self) -> PixelPtrMut<[Self::Pixel]> { self.image.pixels_mut() }
}
