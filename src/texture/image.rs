
use super::*;

glenum! {
    pub enum TexLevelParameteriv {
        [Width TEXTURE_WIDTH "Width"],
        [Height TEXTURE_HEIGHT "Height"],
        [Depth TEXTURE_DEPTH "Depth"],
        [Samples TEXTURE_SAMPLES "Samples"],
        [FixedSampleLocations TEXTURE_FIXED_SAMPLE_LOCATIONS "Fixed Sample Locations"],

        [InternalFormat TEXTURE_INTERNAL_FORMAT "Width"],

        [RedType TEXTURE_RED_TYPE "Red Type"],
        [GreenType TEXTURE_GREEN_TYPE "Green Type"],
        [BlueType TEXTURE_BLUE_TYPE "Blue Type"],
        [AlphaType TEXTURE_ALPHA_TYPE "Alpha Type"],
        [DepthType TEXTURE_DEPTH_TYPE "Depth Type"],

        [RedSize TEXTURE_RED_SIZE "Red Size"],
        [GreenSize TEXTURE_GREEN_SIZE "Green Size"],
        [BlueSize TEXTURE_BLUE_SIZE "Blue Size"],
        [AlphaSize TEXTURE_ALPHA_SIZE "Alpha Size"],
        [DepthSize TEXTURE_DEPTH_SIZE "Depth Size"],
        [StencilSize TEXTURE_STENCIL_SIZE "Stencil Size"],
        [SharedSize TEXTURE_SHARED_SIZE "Shared Size"],

        [Compressed TEXTURE_COMPRESSED "Compressed"],
        [CompressedImageSize TEXTURE_COMPRESSED_IMAGE_SIZE "Compressed Image Size"],

        [BufferDataStoreBinding TEXTURE_BUFFER_DATA_STORE_BINDING "Buffer Data Store Binding"],
        [BufferOffset TEXTURE_BUFFER_OFFSET "Buffer Offset"],
        [BufferSize TEXTURE_BUFFER_SIZE "Buffer Size"]
    }
}

pub unsafe trait Image: Sized {
    type InternalFormat: InternalFormat<FormatType=Self::PixelFormat>;
    type PixelFormat: PixelFormatType;
    type Dim: TexDim;

    const TARGET: TextureTarget;

    fn id(&self) -> GLuint;
    fn level(&self) -> GLuint;
    fn dim(&self) -> Self::Dim;

    fn image<P:PixelData<Self::PixelFormat>+?Sized>(&mut self, data:&P);
    fn sub_image<P:PixelData<Self::PixelFormat>+?Sized>(&mut self, offset:Self::Dim, dim:Self::Dim, data:&P);

    fn clear_image<P:PixelType<Self::PixelFormat>>(&mut self, data:P);
    fn clear_sub_image<P:PixelType<Self::PixelFormat>>(&mut self, offset:Self::Dim, dim:Self::Dim, data:P);

    fn get_image<P:PixelDataMut<Self::PixelFormat>+?Sized>(&self, data: &mut P);

    fn into_box<P:PixelType<Self::PixelFormat>>(&self) -> Box<[P]> {
        let size = size_of::<P>()*self.dim().pixels();
        let mut dest = Vec::with_capacity(size);
        unsafe { dest.set_len(size) };
        self.get_image(dest.as_mut_slice());
        dest.into_boxed_slice()
    }

}

pub struct MipmapLevel<'a, T:Texture+PixelTransfer> {
    pub(super) tex: &'a mut T,
    pub(super) level: GLuint
}

pub(super) fn get_level_parameter_iv<T:Texture>(tex:&T, level:GLuint, pname: TexLevelParameteriv) -> GLint {
    unsafe {
        let mut params = ::std::mem::uninitialized::<GLint>();
        let mut target = T::TARGET.as_loc();
        let binding = target.bind_raw(tex.id()).unwrap();
        gl::GetTexLevelParameteriv(
            binding.target_id(), level as GLint, pname as GLenum, &mut params as *mut GLint
        );
        params
    }

}

impl<'a, T:Texture+PixelTransfer> MipmapLevel<'a,T> {

    #[inline] pub fn get_parameter_iv(&self, pname: TexLevelParameteriv) -> GLint {
        get_level_parameter_iv(self.tex, self.level, pname)
    }

    #[inline]
    unsafe fn prepare_transfer<'b, P:PixelData<T::PixelFormat>+?Sized>(
        &self, data:&'b P, loc: &'b mut BindingLocation<UninitBuf>
    ) -> Option<Binding<'b, UninitBuf>> {
        let dim = self.dim();
        if data.count() != dim.pixels() {
            panic!("Invalid pixel count");
        } else {
            match loc.target() {
                BufferTarget::PixelUnpackBuffer => apply_unpacking_settings(data),
                BufferTarget::PixelPackBuffer => apply_packing_settings(data),
                _ => panic!("Invalid target for pixel buffer transfer")
            }
            data.bind_pixel_buffer(loc)
        }
    }
}

unsafe impl<'a, T:Texture+PixelTransfer> Image for MipmapLevel<'a, T> {
    type InternalFormat = T::InternalFormat;
    type PixelFormat = T::PixelFormat;
    type Dim = T::Dim;

    const TARGET: TextureTarget = T::TARGET;

    #[inline] fn id(&self) -> GLuint {self.tex.id()}
    #[inline] fn level(&self) -> GLuint {self.level}
    #[inline] fn dim(&self) -> Self::Dim { self.tex.dim().minimized(self.level)}

    fn image<P:PixelData<Self::PixelFormat>+?Sized>(&mut self, data:&P) {
        unsafe {
            let mut pixel_buf = BufferTarget::PixelUnpackBuffer.as_loc();
            let _buf = self.prepare_transfer(data, &mut pixel_buf);
            let (fmt, ty) = data.format_type().format_type();
            tex_image::<T>(self.id(),self.level(),self.dim(),fmt.into(),ty.into(),data.pixels());
        }
    }

    fn sub_image<P:PixelData<Self::PixelFormat>+?Sized>(&mut self, offset:Self::Dim, dim:Self::Dim, data:&P) {
        unsafe {
            let mut pixel_buf = BufferTarget::PixelUnpackBuffer.as_loc();
            let _buf = self.prepare_transfer(data, &mut pixel_buf);

            let mut target = T::TARGET.as_loc();
            let binding = target.bind_raw(self.id()).unwrap();
            let level = self.level() as GLint;

            //TODO add error checking for dimensions
            let (x,y,z) = (offset.width() as GLsizei, offset.height() as GLsizei, offset.depth() as GLsizei);
            let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);
            let (fmt, ty) = data.format_type().format_type();

            match T::Dim::dim() {
                1 => gl::TexSubImage1D(binding.target_id(), level, x, w, fmt.into(), ty.into(), data.pixels()),
                2 => gl::TexSubImage2D(binding.target_id(), level, x,y, w,h, fmt.into(), ty.into(), data.pixels()),
                3 => gl::TexSubImage3D(binding.target_id(), level, x,y,z, w,h,d, fmt.into(), ty.into(), data.pixels()),
                _ => panic!("{}D Textures not currently supported", T::Dim::dim())
            }
        }
    }

    fn clear_image<P:PixelType<Self::PixelFormat>>(&mut self, data:P) {
        //TODO: provide some sort of fallback for if glClearTexImage isn't available
        unsafe {
            let (fmt, ty) = P::format_type().format_type();
            gl::ClearTexImage(self.id(), self.level() as GLint, fmt.into(), ty.into(), &data as *const P as *const GLvoid);
        }
    }
    fn clear_sub_image<P:PixelType<Self::PixelFormat>>(&mut self, offset:Self::Dim, dim:Self::Dim, data:P) {
        //TODO: provide some sort of fallback for if glClearTexImage isn't available
        unsafe {
            let (fmt, ty) = P::format_type().format_type();
            let (x,y,z) = (offset.width() as GLsizei, offset.height() as GLsizei, offset.depth() as GLsizei);
            let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);
            gl::ClearTexSubImage(self.id(), self.level() as GLint, x,y,z, w,h,d, fmt.into(), ty.into(), &data as *const P as *const GLvoid);
        }
    }

    fn get_image<P:PixelDataMut<Self::PixelFormat>+?Sized>(&self, data: &mut P) {
        unsafe {
            let (fmt, ty) = data.format_type().format_type();

            if gl::GetTextureImage::is_loaded() {
                gl::GetTextureImage(self.id(), self.level() as GLint, fmt.into(), ty.into(), data.size() as GLsizei, data.pixels_mut());
            } else {
                let mut target = T::TARGET.as_loc();
                let binding = target.bind_raw(self.id()).unwrap();
                gl::GetTexImage(binding.target_id(), self.level() as GLint, fmt.into(), ty.into(), data.pixels_mut());
            }

        }
    }

}
