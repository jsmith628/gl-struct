use super::*;

impl<'a,F:InternalFormat,T:PixelTransferTarget<F>> TexImage<'a,F,T> {

    unsafe fn pack<P:?Sized, GL:FnOnce(GLenum, GLsizei, *mut GLvoid)>(
        &self, settings:PixelStore, pixels: PixelPtrMut<P>, gl:GL
    ) {

        settings.apply_packing();

        let (id, ptr) = match pixels {
            PixelPtrMut::Slice(slice) => (None, slice as *mut GLvoid),
            PixelPtrMut::Buffer(buf, offset) => (Some(buf), offset as *mut GLvoid),
        };

        if let Some(i) = id { gl::BindBuffer(gl::PIXEL_PACK_BUFFER, i) };
        TEXTURE0.map_bind(self,
            |_| gl(self.face.into(), self.level() as GLsizei, ptr)
        );
        if id.is_some() { gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0) };

    }

    unsafe fn pack_pixels<P:Pixel<F::ClientFormat>>(&self, mut settings:PixelStore, pixels: PixelPtrMut<[P]>) {
        settings.swap_bytes ^= P::swap_bytes();
        settings.lsb_first ^= P::lsb_first();
        let fmt = P::format();
        self.pack(
            settings, pixels,
            |f, lvl, ptr| gl::GetTexImage(f,lvl,fmt.fmt().into(),fmt.ty().into(),ptr)
        );
    }

    pub fn get_image<I:TexImageDst<F>>(&self, mut data: I) {
        dest_size_check(self.dim(), &data);
        unsafe { self.pack_pixels(PixelStore::from(&data), data.pixels_mut()); }
    }

    pub fn into_image<I:OwnedTexImage<F>>(&self, gl:&I::GL, hint:I::Hint) -> I {
        unsafe { I::from_gl(gl, hint, self.dim().into_array(), |s, ptr| self.pack_pixels(s, ptr)) }
    }

    pub fn try_into_image<I:OwnedTexImage<F>>(&self, hint:I::Hint) -> Result<I,GLVersionError> {
        Ok(self.into_image(&upgrade_to(&self.gl())?, hint))
    }

}

impl<'a,F:SpecificCompressed,T:CompressedTransferTarget<F>> TexImage<'a,F,T> {

    unsafe fn pack_compressed_pixels(
        &self, settings:PixelStore, pixels: PixelPtrMut<CompressedPixels<F>>
    ) where F:SpecificCompressed {

        //since these are specific to the format, these are set independent of PixelStore::apply_packing()
        gl::PixelStorei(gl::PACK_COMPRESSED_BLOCK_SIZE, F::block_size().try_into().unwrap());
        gl::PixelStorei(gl::PACK_COMPRESSED_BLOCK_WIDTH, F::block_width().into());
        gl::PixelStorei(gl::PACK_COMPRESSED_BLOCK_HEIGHT, F::block_height().into());
        gl::PixelStorei(gl::PACK_COMPRESSED_BLOCK_DEPTH, F::block_depth().into());

        self.pack(settings, pixels, |f, lvl, ptr| gl::GetCompressedTexImage(f, lvl, ptr));
    }

    pub fn get_compressed_image<I:CompressedImageDst<Format=F>>(&self, mut data: I) {
        dest_size_check(self.dim(), &data);
        unsafe { self.pack_compressed_pixels(PixelStore::from(&data), data.pixels_mut()); }
    }

    pub fn into_compressed_image<I:OwnedCompressedImage<Format=F>>(&self, gl:&I::GL, hint:I::Hint) -> I {
        let dim = self.dim();
        compressed_block_check::<_,F>(dim);
        unsafe {
            I::from_gl(gl, hint, dim.into_array(), |s, ptr| self.pack_compressed_pixels(s, ptr))
        }
    }

    pub fn try_into_compressed_image<I:OwnedCompressedImage<Format=F>>(&self, hint:I::Hint) -> Result<I,GLVersionError> {
        Ok(self.into_compressed_image(&upgrade_to(&self.gl())?, hint))
    }


}
