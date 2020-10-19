use super::*;


//TODO: change the typing on the methods used here so that we don't have to do as much conversion

//TODO: create a struct and static thread-local variable that keeps track of this
//state so that we can potentiall do some optimizations and encapsulation
unsafe fn apply_packing<P:PixelData+?Sized>(img: &ImageMut<P,()>) {

    gl::PixelStorei(gl::PACK_SWAP_BYTES,   P::swap_bytes().into());
    // gl::PixelStorei(gl::PACK_LSB_FIRST,    self.lsb_first.into());
    // gl::PixelStorei(gl::PACK_ALIGNMENT,    self.row_alignment.0.into());

    //TODO: do this manually using pointer offsets since it makes this step compiler-optimizable
    //and reduces the number of OpenGL calls
    gl::PixelStorei(gl::PACK_SKIP_PIXELS,  img.offset_x().try_into().unwrap());
    gl::PixelStorei(gl::PACK_SKIP_ROWS,    img.offset_y().try_into().unwrap());
    gl::PixelStorei(gl::PACK_SKIP_IMAGES,  img.offset_z().try_into().unwrap());

    //TODO: try benchmarking some tests to avoid these calls
    gl::PixelStorei(gl::PACK_ROW_LENGTH,   img.base_width().try_into().unwrap());
    gl::PixelStorei(gl::PACK_IMAGE_HEIGHT, img.base_height().try_into().unwrap());

}

unsafe fn apply_compressed_packing<P:PixelData+?Sized>(img: &ImageMut<P,()>) {

    gl::PixelStorei(gl::PACK_COMPRESSED_BLOCK_SIZE, P::block_size().try_into().unwrap());
    gl::PixelStorei(gl::PACK_COMPRESSED_BLOCK_WIDTH, P::block_width().try_into().unwrap());
    gl::PixelStorei(gl::PACK_COMPRESSED_BLOCK_HEIGHT, P::block_height().try_into().unwrap());
    gl::PixelStorei(gl::PACK_COMPRESSED_BLOCK_DEPTH, P::block_depth().try_into().unwrap());

    apply_packing(img)

}

impl<'a,F:InternalFormat,T:PixelTransferTarget<F>> TexImage<'a,F,T> {

    unsafe fn pack<GL, P>(&self, gl:&GL, mut img: ImageMut<P,()>) where
        P:UncompressedPixelData<F::PixelLayout>+?Sized,
        GL: Supports<P::GL>
    {

        apply_packing(&img);

        TEXTURE0.map_bind(self,
            |t| PIXEL_PACK_BUFFER.map_bind_mut(&mut img,
                |mut img| gl::GetTexImage(
                    t.resource().face.into(),
                    t.resource().level().try_into().unwrap(),
                    P::layout(gl).fmt().into(),
                    P::layout(gl).ty().into(),
                    img.resource_mut().base_image_mut().pixels_mut().void_ptr_mut()
                )
            )
        );

    }

    pub fn get_image<GL,I:ImageDst>(&self, gl:&GL, mut img:I) where
        I::Pixels: UncompressedPixelData<F::PixelLayout>,
        GL: Supports<I::GL> + Supports<<I::Pixels as UncompressedPixelData<F::PixelLayout>>::GL>
    {
        let mut img = img.image_mut();
        dest_size_check(self.dim(), &img);
        unsafe { self.pack(gl, img.unlock(gl)); }
    }

}

impl<'a,F:SpecificCompressed,T:CompressedTransferTarget<F>> TexImage<'a,F,T> {

    unsafe fn pack_compressed<P:CompressedPixelData<Format=F>+?Sized>(&self, mut img: ImageMut<P,()>) {
        apply_compressed_packing(&img);
        TEXTURE0.map_bind(self,
            |t| PIXEL_PACK_BUFFER.map_bind_mut(&mut img,
                |mut img| gl::GetCompressedTexImage(
                    t.resource().face.into(),
                    t.resource().level().try_into().unwrap(),
                    img.resource_mut().base_image_mut().pixels_mut().void_ptr_mut()
                )
            )
        );
    }

    pub fn get_compressed_image<GL,I:ImageDst>(&self, gl:&GL, mut data:I) where
        I::Pixels: CompressedPixelData<Format=F>,
        GL: Supports<I::GL>
    {
        let img = data.image_mut();
        dest_size_check(self.dim(), &img);
        unsafe { self.pack_compressed(img.unlock(gl)); }
    }


}
