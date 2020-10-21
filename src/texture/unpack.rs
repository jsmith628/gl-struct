use super::*;

unsafe fn apply_unpacking<P:PixelData+?Sized>(img: &ImageRef<P,()>) {

    gl::PixelStorei(gl::UNPACK_SWAP_BYTES,   P::swap_bytes().into());

    gl::PixelStorei(gl::UNPACK_ROW_LENGTH,   img.base_width().try_into().unwrap());
    gl::PixelStorei(gl::UNPACK_IMAGE_HEIGHT, img.base_height().try_into().unwrap());

}

unsafe fn apply_compressed_unpacking<P:PixelData+?Sized>(img: &ImageRef<P,()>) {

    gl::PixelStorei(gl::UNPACK_COMPRESSED_BLOCK_SIZE, P::block_size().try_into().unwrap());
    gl::PixelStorei(gl::UNPACK_COMPRESSED_BLOCK_WIDTH, P::block_width().try_into().unwrap());
    gl::PixelStorei(gl::UNPACK_COMPRESSED_BLOCK_HEIGHT, P::block_height().try_into().unwrap());
    gl::PixelStorei(gl::UNPACK_COMPRESSED_BLOCK_DEPTH, P::block_depth().try_into().unwrap());

    apply_unpacking(img)

}

fn pixel_ptr_with_offset<P:PixelData+?Sized>(img: &ImageRef<P,()>)  -> *const GLvoid {
    //safe because safe creation of a ClientImage or ClientSubImage does the checks that make
    //the function safe
    unsafe {
        offset_img_ptr(
            img.base_image().pixels().void_ptr(),
            img.block_dim(),
            img.block_size(),
            img.offset(),
            img.base_dim()
        )
    }
}

impl<'a,F:InternalFormat,T:PixelTransferTarget<F>> TexImageMut<'a,F,T> {

    unsafe fn unpack<GL,P>(&self, gl:&GL, offset: Option<[usize;3]>, img: ImageRef<P,()>) where
        P:UncompressedPixelData<F::PixelLayout>+?Sized,
        GL: Supports<P::GL>
    {

        TEXTURE0.map_bind(self,
            |t| PIXEL_UNPACK_BUFFER.map_bind(&img,
                |img| {
                    let img = img.resource();
                    apply_unpacking(img);

                    let face = t.resource().face.into();
                    let lvl = t.resource().level() as _;

                    let [w,h,d] = img.dim();
                    let [w,h,d] = [w.try_into().unwrap(), h.try_into().unwrap(), d.try_into().unwrap()];

                    //if the source is zero-sized, we don't need to do anything
                    if w==0 || h==0 || d==0 { return; }

                    let layout = P::layout(gl);
                    let (fmt, ty) = (layout.fmt().into(), layout.fmt().into());

                    let ptr = pixel_ptr_with_offset(img);

                    match offset {

                        None => {
                            let int_fmt = F::glenum() as GLint;
                            match T::Dim::dim() {
                                1 => gl::TexImage1D(face, lvl, int_fmt, w, 0, fmt, ty, ptr),
                                2 => gl::TexImage2D(face, lvl, int_fmt, w, h, 0, fmt, ty, ptr),
                                3 => gl::TexImage3D(face, lvl, int_fmt, w, h, d, 0, fmt, ty, ptr),
                                n => panic!("{}D Textures not supported", n)
                            }

                        }

                        Some([x,y,z]) => {
                            let [x,y,z] = [x.try_into().unwrap(), y.try_into().unwrap(), z.try_into().unwrap()];
                            match T::Dim::dim() {
                                1 => gl::TexSubImage1D(face, lvl, x, w, fmt, ty, ptr),
                                2 => gl::TexSubImage2D(face, lvl, x,y, w,h, fmt, ty, ptr),
                                3 => gl::TexSubImage3D(face, lvl, x,y,z, w,h,d, fmt, ty, ptr),
                                n => panic!("{}D Textures not supported", n)
                            }

                        }

                    }
                }

            )
        );


    }

    //makes sure it is valid to create an image level with these dimensions
    fn check_image_init(&self, [w,h,d]: [usize;3]) {
        if w==0 || h==0 || d==0 { panic!("Attempted to create a zero-sized texture image"); }
        if T::glenum() == gl::TEXTURE_CUBE_MAP_ARRAY && d%6 != 0 {
            panic!("Attempted to make a cube-map array with a depth not divisible by 6 ");
        }
        if T::Dim::dim()==1 && h!=1 { panic!("Attempted to create a 1D texture from a 2D image"); }
        if T::Dim::dim()==1 && d!=1  { panic!("Attempted to create a 1D texture from a 3D image"); }
        if T::Dim::dim()==2 && d!=1  { panic!("Attempted to create a 2D texture from a 3D image"); }
    }

    pub(super) unsafe fn image_init<GL,P>(&mut self, gl: &GL, img: ImageRef<P,()>) where
        P:UncompressedPixelData<F::PixelLayout>+?Sized,
        GL: Supports<P::GL>
    {
        self.check_image_init(img.dim());
        self.unpack(gl, None, img);
    }

    pub fn image<GL, I:ImageSrc>(&mut self, gl: &GL, img: I) where
        I::Pixels: UncompressedPixelData<F::PixelLayout>,
        GL: Supports<I::GL> + Supports<<I::Pixels as UncompressedPixelData<F::PixelLayout>>::GL>
    {
        //get the current dimensions
        let current_dim = self.dim();

        //get the pixels and unlock
        let img = img.image().unlock(gl);

        //check if this image hasn't been initialized yet
        if current_dim.pixels()==0 {
            //if so, run glTexImage*D
            let dim = self.base_dim().minimized(self.level());
            size_check(dim, &img);
            unsafe { self.image_init(gl, img) }
        } else {
            //else, run glTexSubImage*D
            size_check(current_dim, &img);
            unsafe { self.unpack(gl, Some([0,0,0]), img) }
        }
    }

    pub fn sub_image<GL, I:ImageSrc>(&mut self, gl: &GL, offset: T::Dim, img: I) where
        I::Pixels: UncompressedPixelData<F::PixelLayout>,
        GL: Supports<I::GL> + Supports<<I::Pixels as UncompressedPixelData<F::PixelLayout>>::GL>
    {
        source_size_check(offset, self.dim(), &img);
        unsafe { self.unpack(gl, Some(offset.into_array()), img.image().unlock(gl)) }
    }

    pub fn get_image<GL,I:ImageDst>(&self, gl:&GL, img:I) where
        I::Pixels: UncompressedPixelData<F::PixelLayout>,
        GL: Supports<I::GL> + Supports<<I::Pixels as UncompressedPixelData<F::PixelLayout>>::GL>
    {
        self.as_immut().get_image(gl, img);
    }

}

impl<'a,F:SpecificCompressed,T:CompressedTransferTarget<F>> TexImageMut<'a,F,T> {

    unsafe fn unpack_compressed<P:CompressedPixelData<Format=F>+?Sized>(
        &self, offset: Option<[usize;3]>, img: ImageRef<P,()>
    ) {

        TEXTURE0.map_bind(self,
            |t| PIXEL_UNPACK_BUFFER.map_bind(&img,
                |img| {
                    let img = img.resource();
                    apply_compressed_unpacking(img);

                    let face = t.resource().face.into();
                    let lvl = t.resource().level() as _;

                    let [w,h,d] = img.dim();
                    let [w,h,d] = [w.try_into().unwrap(), h.try_into().unwrap(), d.try_into().unwrap()];

                    //if the source is zero-sized, we don't need to do anything
                    if w==0 || h==0 || d==0 { return; }

                    let size = img.base_image().pixels().size().try_into().unwrap();
                    let ptr = pixel_ptr_with_offset(img);

                    let int_fmt = F::glenum() as _;

                    match offset {

                        None => {
                            match T::Dim::dim() {
                                1 => gl::CompressedTexImage1D(face, lvl, int_fmt, w,     0, size, ptr),
                                2 => gl::CompressedTexImage2D(face, lvl, int_fmt, w,h,   0, size, ptr),
                                3 => gl::CompressedTexImage3D(face, lvl, int_fmt, w,h,d, 0, size, ptr),
                                n => panic!("{}D Textures not supported", n)
                            }

                        }

                        Some([x,y,z]) => {
                            //TODO: double check that the "format" parameter is the same as the
                            //internal format since it seems weird we'd have to specify it here
                            let [x,y,z] = [x.try_into().unwrap(), y.try_into().unwrap(), z.try_into().unwrap()];
                            match T::Dim::dim() {
                                1 => gl::CompressedTexSubImage1D(face, lvl, x,     w,     int_fmt, size, ptr),
                                2 => gl::CompressedTexSubImage2D(face, lvl, x,y,   w,h,   int_fmt, size, ptr),
                                3 => gl::CompressedTexSubImage3D(face, lvl, x,y,z, w,h,d, int_fmt, size, ptr),
                                n => panic!("{}D Textures not supported", n)
                            }

                        }

                    }
                }

            )
        );


    }

    pub(super) unsafe fn compressed_image_init<P:CompressedPixelData<Format=F>+?Sized>(
        &self, img: ImageRef<P,()>
    ) {
        self.check_image_init(img.dim());
        self.unpack_compressed(None, img)
    }

    pub fn compressed_image<GL, I:ImageSrc>(&mut self, gl: &GL, img: I) where
        I::Pixels: CompressedPixelData<Format=F>,
        GL: Supports<I::GL>
    {
        //get the current dimensions
        let current_dim = self.dim();

        //get the pixels and unlock
        let img = img.image().unlock(gl);

        //check if this image hasn't been initialized yet
        if current_dim.pixels()==0 {
            //if so, run glCompressedTexImage*D
            let dim = self.base_dim().minimized(self.level());
            size_check(dim, &img);
            unsafe { self.compressed_image_init(img) }
        } else {
            //else, run glCompressedTexSubImage*D
            size_check(current_dim, &img);
            unsafe { self.unpack_compressed(Some([0,0,0]), img) }
        }
    }

    pub fn compressed_sub_image<GL, I:ImageSrc>(&mut self, gl: &GL, offset: T::Dim, img: I) where
        I::Pixels: CompressedPixelData<Format=F>, GL: Supports<I::GL>
    {
        let img = img.image().unlock(gl);
        source_size_check(offset, self.dim(), &img);
        unsafe { self.unpack_compressed(Some(offset.into_array()), img) }
    }

    pub fn get_compressed_image<GL,I:ImageDst>(&self, gl:&GL, img:I) where
        I::Pixels: CompressedPixelData<Format=F>,
        GL: Supports<I::GL>
    {
        self.as_immut().get_compressed_image(gl, img);
    }

}
