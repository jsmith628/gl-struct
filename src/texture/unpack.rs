use super::*;

impl<'a,F:InternalFormat,T:PixelTransferTarget<F>> TexImageMut<'a,F,T> {

    unsafe fn unpack<
        GL:FnOnce(GLenum, GLint, [GLsizei;3], *const GLvoid), I:ImageSrc
    >(&mut self, settings:PixelStore, data: I, gl:GL) {

        settings.apply_unpacking();

        let (w, h, d) = (data.width(), data.height(), data.depth());
        let dim = [w.try_into().unwrap(), h.try_into().unwrap(), d.try_into().unwrap()];

        let (id, ptr) = match data.pixels() {
            PixelPtr::Slice(slice) => (None, slice as *const GLvoid),
            PixelPtr::Buffer(buf, offset) => (Some(buf), offset as *const GLvoid),
        };

        if let Some(i) = id { gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, i) };
        TEXTURE0.map_bind(self, |_| gl(self.face.into(), self.level() as GLint, dim, ptr));
        if id.is_some() { gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0) };
    }

    unsafe fn unpack_pixels<
        GL:FnOnce(GLenum, GLint, [GLsizei;3], GLenum, GLenum, *const GLvoid), I:TexImageSrc<F>
    >(&mut self, data: I, gl:GL) {

        let mut settings = PixelStore::from(&data);
        settings.swap_bytes ^= I::Pixel::swap_bytes();
        settings.lsb_first ^= I::Pixel::lsb_first();

        let fmt = I::Pixel::layout(unimplemented!());
        self.unpack(settings, data, |f,l,d,p| gl(f,l,d,fmt.fmt().into(),fmt.ty().into(),p))

    }

    fn image_dim_check<I:ImageSrc>(&self, data: &I) {
        if data.width()==0 || data.height()==0 || data.depth()==0 {
            panic!("Attempted to create a zero-sized texture image");
        }
        if T::glenum() == gl::TEXTURE_CUBE_MAP_ARRAY && data.depth()%6 != 0 {
            panic!("Attempted to make a cube-map array with a depth not divisible by 6 ");
        }
        if T::Dim::dim()==1 && data.height()!=1 { panic!("Attempted to create a 1D texture from a 2D image"); }
        if T::Dim::dim()==1 && data.depth()!=1  { panic!("Attempted to create a 1D texture from a 3D image"); }
        if T::Dim::dim()==2 && data.depth()!=1  { panic!("Attempted to create a 2D texture from a 3D image"); }
    }

    pub(super) unsafe fn image_unchecked<I:TexImageSrc<F>>(&mut self, data: I) {
        self.image_dim_check(&data);
        self.unpack_pixels(
            data,
            |face, lvl, [w,h,d], fmt, ty, ptr| {
                match T::Dim::dim() {
                    1 => gl::TexImage1D(face, lvl, F::glenum() as GLint, w, 0, fmt, ty, ptr),
                    2 => gl::TexImage2D(face, lvl, F::glenum() as GLint, w, h, 0, fmt, ty, ptr),
                    3 => gl::TexImage3D(face, lvl, F::glenum() as GLint, w, h, d, 0, fmt, ty, ptr),
                    n => panic!("{}D Textures not supported", n)
                }
            }
        )
    }

    unsafe fn sub_image_unchecked<I:TexImageSrc<F>>(&mut self, offset:T::Dim, data: I) {

        if data.width()==0 || data.height()==0 || data.depth()==0 { return; }
        let (x, y, z) = (offset.width() as GLsizei, offset.height() as GLsizei, offset.depth() as GLsizei);

        self.unpack_pixels(
            data,
            |face, lvl, [w,h,d], fmt, ty, ptr| {
                match T::Dim::dim() {
                    1 => gl::TexSubImage1D(face, lvl, x, w, fmt, ty, ptr),
                    2 => gl::TexSubImage2D(face, lvl, x,y, w,h, fmt, ty, ptr),
                    3 => gl::TexSubImage3D(face, lvl, x,y,z, w,h,d, fmt, ty, ptr),
                    n => panic!("{}D Textures not supported", n)
                }
            }
        )

    }

    pub fn image<I:TexImageSrc<F>>(&mut self, data: I) {
        //get the current dimensions
        let current_dim = self.dim();

        //check if this image hasn't been initialized yet
        if current_dim.pixels()==0 {
            //if so, run glTexImage*D
            let dim = self.base_dim().minimized(self.level());
            size_check(dim, &data);
            unsafe { self.image_unchecked(data) }
        } else {
            //else, run glTexSubImage*D
            size_check(current_dim, &data);
            unsafe { self.sub_image_unchecked(T::Dim::new(0,0,0), data) }
        }
    }

    pub fn sub_image<I:TexImageSrc<F>>(&mut self, offset:T::Dim, data: I) {
        source_size_check(offset, self.dim(), &data);
        unsafe { self.sub_image_unchecked(offset, data) }
    }

    pub fn get_image<I:TexImageDst<F>>(&self, data: I) {
        self.as_immut().get_image(data);
    }

    pub fn into_image<I:OwnedTexImage<F>>(&self, gl:&<I as OwnedImage>::GL, hint:I::Hint) -> I {
        self.as_immut().into_image(gl, hint)
    }

    pub fn try_into_image<I:OwnedTexImage<F>>(&self, hint:I::Hint) -> Result<I,GLVersionError> {
        self.as_immut().try_into_image(hint)
    }

}

impl<'a,F:SpecificCompressed,T:CompressedTransferTarget<F>> TexImageMut<'a,F,T> {

    unsafe fn unpack_compressed_pixels<
        GL:FnOnce(GLenum, GLint, [GLsizei;3], GLsizei, *const GLvoid), I:CompressedImageSrc<Format=F>
    >(&mut self, data: I, gl:GL) {

        gl::PixelStorei(gl::UNPACK_COMPRESSED_BLOCK_SIZE, F::block_size().try_into().unwrap());
        gl::PixelStorei(gl::UNPACK_COMPRESSED_BLOCK_WIDTH, F::block_width().into());
        gl::PixelStorei(gl::UNPACK_COMPRESSED_BLOCK_HEIGHT, F::block_height().into());
        gl::PixelStorei(gl::UNPACK_COMPRESSED_BLOCK_DEPTH, F::block_depth().into());

        let size = data.pixels().size();

        self.unpack(PixelStore::from(&data), data, |f,l,d,p| gl(f,l,d,size.try_into().unwrap(),p))

    }

    pub(super) unsafe fn compressed_image_unchecked<I:CompressedImageSrc<Format=F>>(&mut self, data:I) {
        self.image_dim_check(&data);
        self.unpack_compressed_pixels(data,
            |face, lvl, [w,h,d], size, ptr| {
                match T::Dim::dim() {
                    1 => gl::CompressedTexImage1D(face, lvl, F::glenum(), w, 0, size, ptr),
                    2 => gl::CompressedTexImage2D(face, lvl, F::glenum(), w, h, 0, size, ptr),
                    3 => gl::CompressedTexImage3D(face, lvl, F::glenum(), w, h, d, 0, size, ptr),
                    n => panic!("{}D Textures not supported", n)
                }
            }
        )
    }

    pub(super) unsafe fn compressed_sub_image_unchecked<I:CompressedImageSrc<Format=F>>(
        &mut self, offset:T::Dim, data:I
    ) {

        if data.width()==0 || data.height()==0 || data.depth()==0 { return; }
        let (x, y, z) = (offset.width() as GLsizei, offset.height() as GLsizei, offset.depth() as GLsizei);

        self.unpack_compressed_pixels(data,
            |face, lvl, [w,h,d], size, ptr| {
                match T::Dim::dim() {
                    1 => gl::CompressedTexSubImage1D(face, lvl, x, w, F::glenum(), size, ptr),
                    2 => gl::CompressedTexSubImage2D(face, lvl, x,y, w,h, F::glenum(), size, ptr),
                    3 => gl::CompressedTexSubImage3D(face, lvl, x,y,z, w,h,d, F::glenum(), size, ptr),
                    n => panic!("{}D Textures not supported", n)
                }
            }
        )
    }

    pub fn compressed_image<I:CompressedImageSrc<Format=F>>(&mut self, data: I) {
        //get the current dimensions
        let current_dim = self.dim();

        //check if this image hasn't been initialized yet
        if current_dim.pixels()==0 {
            //if so, run glCompressedTexImage*D
            let dim = self.base_dim().minimized(self.level());
            size_check(dim, &data);
            unsafe { self.compressed_image_unchecked(data) }
        } else {
            //else, run glCompressedTexSubImage*D
            size_check(current_dim, &data);
            unsafe { self.compressed_sub_image_unchecked(T::Dim::new(0,0,0), data) }
        }
    }

    pub fn compressed_sub_image<I:CompressedImageSrc<Format=F>>(&mut self, offset:T::Dim, data: I) {
        source_size_check(offset, self.dim(), &data);
        unsafe { self.compressed_sub_image_unchecked(offset, data) }
    }

    pub fn get_compressed_image<I:CompressedImageDst<Format=F>>(&self, data: I) {
        self.as_immut().get_compressed_image(data);
    }

    pub fn into_compressed_image<I:OwnedCompressedImage<Format=F>>(&self, gl:&I::GL, hint:I::Hint) -> I {
        self.as_immut().into_compressed_image(gl, hint)
    }

    pub fn try_into_compressed_image<I:OwnedCompressedImage<Format=F>>(&self, hint:I::Hint) -> Result<I,GLVersionError> {
        self.as_immut().try_into_compressed_image(hint)
    }

}
