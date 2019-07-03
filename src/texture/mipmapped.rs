
use super::*;
use image_format::pixel_data::{apply_packing_settings, apply_unpacking_settings};

fn clamp_range<T:MipmappedTexture, R:RangeBounds<GLuint>>(t:&T, r:&R) -> (GLuint, GLuint) {
    (
        match r.start_bound() {
            Bound::Included(m) => *m,
            Bound::Excluded(m) => *m + 1,
            Bound::Unbounded => 0
        },
        match r.end_bound() {
            Bound::Included(m) => *m,
            Bound::Excluded(m) => *m - 1,
            Bound::Unbounded => t.dim().max_levels()
        }
    )
}

pub unsafe trait MipmappedTexture: PixelTransfer {

    #[inline] fn immutable_levels(&self) -> GLuint {unsafe{get_tex_parameter_iv(self, gl::TEXTURE_IMMUTABLE_LEVELS) as GLuint}}
    #[inline] fn base_level(&self) -> GLuint {unsafe{get_tex_parameter_iv(self, gl::TEXTURE_BASE_LEVEL) as GLuint}}
    #[inline] fn max_level(&self) -> GLuint {unsafe{get_tex_parameter_iv(self, gl::TEXTURE_MAX_LEVEL) as GLuint}}

    #[inline] fn set_base_level(&mut self, level: GLuint) {
        if cfg!(debug_assertions) && level > self.max_level() { panic!("Base level higher than current max level"); }
        unsafe { tex_parameter_iv(self, gl::TEXTURE_BASE_LEVEL, &(level as GLint) as *const GLint) }
    }

    #[inline] fn set_max_level(&mut self, level: GLuint) {
        if cfg!(debug_assertions) {
            if level < self.base_level() { panic!("Max level lower than max level"); }
            if self.immutable_storage() && level >= self.immutable_levels() {
                panic!("Base level higher than allocated immutable storage");
            }
        }
        unsafe { tex_parameter_iv(self, gl::TEXTURE_MAX_LEVEL, &(level as GLint) as *const GLint) }
    }

    #[inline]
    unsafe fn alloc_mipmaps(gl:&Self::GL, levels:GLuint, dim:Self::Dim) -> Self {
        let raw = RawTex::gen(gl);
        if let Ok(gl43) = gl.try_as_gl43() {
            if_sized!(
                helper()(_gl:&GL43,tex:RawTex<T::Target>,l:GLuint,d:T::Dim) -> T
                    {unsafe{T::image_mipmaps(tex, l, d)}}
                    {unsafe{T::storage_mipmaps(_gl, tex, l, d)}}
                where T:MipmappedTexture
            );
            Self::InternalFormat::helper(&gl43, raw, levels, dim)
        } else {
            Self::image_mipmaps(raw, levels, dim)
        }
    }

    #[inline]
    unsafe fn image_mipmaps(raw:RawTex<Self::Target>, levels:GLuint, dim:Self::Dim) -> Self {
        let mut tex = Self::image(raw, dim);
        tex.set_max_level(levels);
        // tex.gen_mipmaps(0..levels);
        tex
    }

    #[inline]
    unsafe fn storage_mipmaps(gl:&GL43, raw:RawTex<Self::Target>, levels:GLuint, dim:Self::Dim) -> Self
    where Self::InternalFormat: SizedInternalFormat{
        tex_storage(gl, raw, levels, dim, None)
    }

    fn from_mipmaps<P:PixelData<Self::ClientFormat>+?Sized>(
        gl:&Self::GL, levels:GLuint, dim:Self::Dim, base:&P, mipmaps: Option<HashMap<GLuint, &P>>
    ) -> Self {
        let raw = RawTex::gen(gl);
        if let Ok(gl43) = gl.try_as_gl43() {
            if_sized!(
                helper(P:PixelData<T::ClientFormat>+?Sized)(
                    _gl:&GL43, tex:RawTex<T::Target>, l:GLuint, d:T::Dim, b:&P, m: Option<HashMap<GLuint, &P>>
                ) -> T
                    {T::image_from_mipmaps(tex, l, d, b, m)}
                    {T::storage_from_mipmaps(_gl, tex, l, d, b, m)}
                where T:MipmappedTexture
            );
            Self::InternalFormat::helper(&gl43, raw, levels, dim, base, mipmaps)
        } else {
            Self::image_from_mipmaps(raw, levels, dim, base, mipmaps)
        }
    }

    fn image_from_mipmaps<P:PixelData<Self::ClientFormat>+?Sized>(
        raw:RawTex<Self::Target>, levels:GLuint, dim:Self::Dim, base:&P, mipmaps: Option<HashMap<GLuint, &P>>
    ) -> Self {
        let mut tex = Self::image_from_pixels(raw, dim, base);
        tex.set_max_level(levels);
        match mipmaps {
            None => tex.gen_mipmaps(0..levels),
            Some(map) => {
                for (level, pixels) in map.into_iter() {
                    tex.level(level).image(pixels);
                }
            }
        }
        tex
    }

    fn storage_from_mipmaps<P>(
        gl:&GL43, raw:RawTex<Self::Target>, levels:GLuint, dim:Self::Dim, base:&P, mipmaps: Option<HashMap<GLuint, &P>>
    ) -> Self where P:PixelData<Self::ClientFormat>+?Sized, Self::InternalFormat:SizedInternalFormat {
        let mut tex = unsafe { Self::storage_mipmaps(gl, raw, levels, dim) };
        tex.set_max_level(levels);
        tex.level(0).image(base);
        match mipmaps {
            None => tex.gen_mipmaps(0..levels),
            Some(map) => {
                let mut levels = Vec::with_capacity(map.len());
                for (level, pixels) in map.into_iter() {
                    //load the mipmap image
                    tex.level(level).image(pixels);

                    //insert the level and sort
                    let mut i = levels.len();
                    levels.push(level);
                    while i>0 {
                        if levels[i-1] > levels[i] { levels.swap(i-1, i) }
                        i -= 1;
                    }
                }

                //generate the empty levels
                for i in 0..levels.len()-1 {
                    if levels[i]+1 < levels[i+1] {
                        tex.gen_mipmaps(levels[i] .. levels[i+1]);
                    }
                }
            }
        }
        tex
    }

    fn gen_mipmaps<R:RangeBounds<GLuint>>(&mut self, range: R) {

        let (min, max) = clamp_range(self, &range);
        if max >= min {
            return;
        } else {
            let (prev_base, prev_max) = (self.base_level(), self.max_level());
            self.set_base_level(min);
            self.set_max_level(max);

            unsafe {
                let mut target = <Self as Texture>::Target::binding_location();
                let binding = target.bind(self.raw());
                gl::GenerateMipmap(binding.target_id());
            }

            self.set_base_level(prev_base);
            self.set_max_level(prev_max);
        }

    }

    #[inline] fn level(&mut self, level: GLuint) -> MipmapLevel<Self> {MipmapLevel{tex:self, level:level}}
    #[inline] fn level_range<R:RangeBounds<GLuint>>(&mut self, range: R) -> Box<[MipmapLevel<Self>]> {
        let (min, max) = clamp_range(self, &range);
        unsafe {
            let ptr = self as *mut Self;
            (min..=max).map(|i| (&mut *ptr).level(i)).collect::<Box<[_]>>()
        }
    }

}

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

pub struct MipmapLevel<T:Texture+PixelTransfer> {
    pub(super) tex: *mut T,
    pub(super) level: GLuint
}

pub(super) fn get_level_parameter_iv<T:Texture>(tex:&T, level:GLuint, pname: TexLevelParameteriv) -> GLint {
    unsafe {
        let mut params = ::std::mem::uninitialized::<GLint>();
        let mut target = T::Target::binding_location();
        let binding = target.bind(tex.raw());
        gl::GetTexLevelParameteriv(
            binding.target_id(), level as GLint, pname as GLenum, &mut params as *mut GLint
        );
        params
    }

}

impl<T:Texture+PixelTransfer> MipmapLevel<T> {

    #[inline] pub fn get_parameter_iv(&self, pname: TexLevelParameteriv) -> GLint {
        get_level_parameter_iv(unsafe {&*self.tex}, self.level, pname)
    }

}

unsafe impl<T:Texture+PixelTransfer> Image for MipmapLevel<T> {
    type InternalFormat = T::InternalFormat;
    type ClientFormat = T::ClientFormat;
    type Dim = T::Dim;
    type Target = T::Target;

    #[inline] fn level(&self) -> GLuint {self.level}
    #[inline] fn dim(&self) -> Self::Dim {  unsafe {&*self.tex}.dim().minimized(self.level)}
    #[inline] fn raw(&self) -> &RawTex<T::Target> { unsafe { (&*self.tex).raw() } }

    #[inline] fn image<P:PixelData<Self::ClientFormat>+?Sized>(&mut self, data:&P) {
        unsafe { tex_image_data::<T,P>(self.raw().id(),self.level(),self.dim(),data); }
    }

    fn sub_image<P:PixelData<Self::ClientFormat>+?Sized>(&mut self, offset:Self::Dim, dim:Self::Dim, data:&P) {
        unsafe {
            //TODO index checking
            apply_unpacking_settings(data);
            let mut pixel_buf = BufferTarget::PixelUnpackBuffer.as_loc();
            let (_buf, ptr) = data.pixels(&mut pixel_buf);

            let mut target = T::Target::binding_location();
            let binding = target.bind(self.raw());
            let level = self.level() as GLint;

            //TODO add error checking for dimensions
            let (x,y,z) = (offset.width() as GLsizei, offset.height() as GLsizei, offset.depth() as GLsizei);
            let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);
            let (fmt, ty) = data.format_type().format_type();
            let coords = T::Dim::dim();

            match coords {
                1 => gl::TexSubImage1D(binding.target_id(), level, x, w, fmt.into(), ty.into(), ptr),
                2 => gl::TexSubImage2D(binding.target_id(), level, x,y, w,h, fmt.into(), ty.into(), ptr),
                3 => gl::TexSubImage3D(binding.target_id(), level, x,y,z, w,h,d, fmt.into(), ty.into(), ptr),
                _ => panic!("{}D Textures not currently supported", coords)
            }
        }
    }

    fn clear_image<P:PixelType<Self::ClientFormat>>(&mut self, data:P) {
        //TODO: provide some sort of fallback for if glClearTexImage isn't available
        unsafe {
            let (fmt, ty) = P::format_type().format_type();
            gl::ClearTexImage(self.raw().id(), self.level() as GLint, fmt.into(), ty.into(), &data as *const P as *const GLvoid);
        }
    }
    fn clear_sub_image<P:PixelType<Self::ClientFormat>>(&mut self, offset:Self::Dim, dim:Self::Dim, data:P) {
        //TODO: provide some sort of fallback for if glClearTexImage isn't available
        unsafe {
            let (fmt, ty) = P::format_type().format_type();
            let (x,y,z) = (offset.width() as GLsizei, offset.height() as GLsizei, offset.depth() as GLsizei);
            let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);
            gl::ClearTexSubImage(self.raw().id(), self.level() as GLint, x,y,z, w,h,d, fmt.into(), ty.into(), &data as *const P as *const GLvoid);
        }
    }

    fn get_image<P:PixelDataMut<Self::ClientFormat>+?Sized>(&self, data: &mut P) {
        unsafe {
            //TODO index checking
            let (fmt, ty) = data.format_type().format_type();
            let size = data.size() as GLsizei;

            apply_packing_settings(data);
            let mut pixel_buf = BufferTarget::PixelUnpackBuffer.as_loc();
            let (_buf, ptr) = data.pixels_mut(&mut pixel_buf);

            if gl::GetTextureImage::is_loaded() {
                gl::GetTextureImage(self.raw().id(), self.level() as GLint, fmt.into(), ty.into(), size, ptr);
            } else {
                let mut target = T::Target::binding_location();
                let binding = target.bind(self.raw());
                gl::GetTexImage(binding.target_id(), self.level() as GLint, fmt.into(), ty.into(), ptr);
            }

        }
    }

}
