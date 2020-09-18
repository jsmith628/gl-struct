use super::*;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Default,Debug)]
pub struct PixelStore {
    pub swap_bytes: bool,
    pub lsb_first: bool,
    pub row_alignment: PixelRowAlignment,
    pub skip_pixels: usize,
    pub skip_rows: usize,
    pub skip_images: usize,
    pub row_length: usize,
    pub image_height: usize,
}

impl PixelStore {

    // pub fn from<I:ImageSrc>(img:&I) -> Self {
    //     PixelStore {
    //         swap_bytes: img.swap_bytes(),
    //         lsb_first: img.lsb_first(),
    //         row_alignment: img.row_alignment(),
    //         skip_pixels: img.skip_pixels(),
    //         skip_rows: img.skip_rows(),
    //         skip_images: img.skip_images(),
    //         row_length: img.row_length(),
    //         image_height: img.image_height(),
    //     }
    // }

    pub unsafe fn apply_unpacking(self) {
        gl::PixelStorei(gl::UNPACK_SWAP_BYTES,   self.swap_bytes.into());
        gl::PixelStorei(gl::UNPACK_LSB_FIRST,    self.lsb_first.into());
        gl::PixelStorei(gl::UNPACK_ALIGNMENT,    self.row_alignment.0.into());
        gl::PixelStorei(gl::UNPACK_SKIP_PIXELS,  self.skip_pixels.try_into().unwrap());
        gl::PixelStorei(gl::UNPACK_SKIP_ROWS,    self.skip_rows.try_into().unwrap());
        gl::PixelStorei(gl::UNPACK_SKIP_IMAGES,  self.skip_images.try_into().unwrap());
        gl::PixelStorei(gl::UNPACK_ROW_LENGTH,   self.row_length.try_into().unwrap());
        gl::PixelStorei(gl::UNPACK_IMAGE_HEIGHT, self.image_height.try_into().unwrap());
    }

    pub unsafe fn apply_packing(self) {
        gl::PixelStorei(gl::PACK_SWAP_BYTES,   self.swap_bytes.into());
        gl::PixelStorei(gl::PACK_LSB_FIRST,    self.lsb_first.into());
        gl::PixelStorei(gl::PACK_ALIGNMENT,    self.row_alignment.0.into());
        gl::PixelStorei(gl::PACK_SKIP_PIXELS,  self.skip_pixels.try_into().unwrap());
        gl::PixelStorei(gl::PACK_SKIP_ROWS,    self.skip_rows.try_into().unwrap());
        gl::PixelStorei(gl::PACK_SKIP_IMAGES,  self.skip_images.try_into().unwrap());
        gl::PixelStorei(gl::PACK_ROW_LENGTH,   self.row_length.try_into().unwrap());
        gl::PixelStorei(gl::PACK_IMAGE_HEIGHT, self.image_height.try_into().unwrap());
    }

}
