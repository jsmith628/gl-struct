use super::*;
use std::convert::TryInto;

glenum!{
    pub enum CubeMapFace {
        [PositiveX TEXTURE_CUBE_MAP_POSITIVE_X "Cube-map Positive X"],
        [NegativeX TEXTURE_CUBE_MAP_NEGATIVE_X "Cube-map Negative X"],
        [PositiveY TEXTURE_CUBE_MAP_POSITIVE_Y "Cube-map Positive Y"],
        [NegativeY TEXTURE_CUBE_MAP_NEGATIVE_Y "Cube-map Negative Y"],
        [PositiveZ TEXTURE_CUBE_MAP_POSITIVE_Z "Cube-map Positive Z"],
        [NegativeZ TEXTURE_CUBE_MAP_NEGATIVE_Z "Cube-map Negative Z"]
    }
}

impl CubeMapFace {
    pub(super) fn faces() -> &'static [CubeMapFace] {
        use self::CubeMapFace::*;
        &[PositiveX, NegativeX, PositiveY, NegativeY, PositiveZ, NegativeZ]
    }
}

impl Default for CubeMapFace { fn default() -> Self { Self::PositiveX } }

pub(super) trait ImageSelector: TextureType { type Selection: GLEnum + Default; }
impl<T: TextureType> ImageSelector for T { default type Selection = Self; }
impl<T: BaseImage> ImageSelector for T { type Selection = Self; }
impl ImageSelector for TEXTURE_CUBE_MAP { type Selection = CubeMapFace; }


#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub struct TexImage<'a,F,T:TextureTarget<F>> {
    pub(super) id: GLuint,
    pub(super) level: GLuint,
    pub(super) face: <T as ImageSelector>::Selection,
    pub(super) tex: PhantomData<&'a Texture<F,T>>
}

pub struct TexImageMut<'a,F,T:TextureTarget<F>> {
    pub(super) id: GLuint,
    pub(super) level: GLuint,
    pub(super) face: <T as ImageSelector>::Selection,
    pub(super) tex: PhantomData<&'a mut Texture<F,T>>
}

pub trait TexImageSrc<F:InternalFormat> = ImageSrc where <Self as ImageSrc>::Pixel: Pixel<<F as InternalFormat>::ClientFormat>;
pub trait TexImageDst<F:InternalFormat> = ImageDst + TexImageSrc<F>;
pub trait OwnedTexImage<F:InternalFormat> = OwnedImage + TexImageSrc<F>;

impl<'a,F,T:TextureTarget<F>> !Sync for TexImage<'a,F,T> {}
impl<'a,F,T:TextureTarget<F>> !Send for TexImage<'a,F,T> {}
impl<'a,F,T:TextureTarget<F>> !Sync for TexImageMut<'a,F,T> {}
impl<'a,F,T:TextureTarget<F>> !Send for TexImageMut<'a,F,T> {}

impl<'a,'b:'a,F,T:TextureTarget<F>> From<&'a TexImage<'b,F,T>> for TexImage<'a,F,T> {
    #[inline] fn from(lvl: &'a TexImage<'b,F,T>) -> Self {*lvl}
}

impl<'a,F,T:TextureTarget<F>> From<TexImageMut<'a,F,T>> for TexImage<'a,F,T> {
    #[inline] fn from(lvl: TexImageMut<'a,F,T>) -> Self {
        TexImage{id:lvl.id, level:lvl.level, face:lvl.face, tex:PhantomData}
    }
}

impl<'a,'b:'a,F,T:TextureTarget<F>> From<&'a TexImageMut<'b,F,T>> for TexImage<'a,F,T> {
    #[inline] fn from(lvl: &'a TexImageMut<'b,F,T>) -> Self {
        TexImage{id:lvl.id, level:lvl.level, face:lvl.face, tex:PhantomData}
    }
}

impl<'a,'b:'a,F,T:TextureTarget<F>> From<&'a mut TexImageMut<'b,F,T>> for TexImageMut<'a,F,T> {
    #[inline] fn from(lvl: &'a mut TexImageMut<'b,F,T>) -> Self {
        TexImageMut{id:lvl.id, level:lvl.level, face:lvl.face, tex:PhantomData}
    }
}

impl<'a,F,T:TextureTarget<F>+BaseImage> From<&'a Texture<F,T>> for TexImage<'a,F,T> {
    #[inline] fn from(tex: &'a Texture<F,T>) -> Self {
        TexImage{id:tex.id, level:0, face:T::default(), tex:PhantomData}
    }
}

impl<'a,F,T:TextureTarget<F>+BaseImage> From<&'a mut Texture<F,T>> for TexImageMut<'a,F,T> {
    #[inline] fn from(tex: &'a mut Texture<F,T>) -> Self {
        TexImageMut{id:tex.id, level:0, face:T::default(), tex:PhantomData}
    }
}

impl<'a,F,T:TextureTarget<F>> TexImage<'a,F,T> {
    pub fn id(&self) -> GLuint { self.id }
    pub fn gl(&self) -> T::GL { unsafe {assume_supported()} }
    pub fn level(&self) -> GLuint { self.level }

    #[inline]
    unsafe fn get_level_parameter_iv(&self, lvl:GLint, pname:GLenum) -> GLint {
        let mut param = MaybeUninit::uninit();
        if T::glenum()!=gl::TEXTURE_CUBE_MAP && gl::GetTextureLevelParameteriv::is_loaded() {
            gl::GetTextureLevelParameteriv(self.id(), lvl, pname, param.as_mut_ptr());
        } else {
            T::bind_loc_level().map_bind(self,
                |_| gl::GetTexLevelParameteriv(self.face.into(), lvl, pname, param.as_mut_ptr())
            );
        }
        param.assume_init()
    }

    #[inline]
    unsafe fn get_parameter_iv(&self, pname:GLenum) -> GLint {
        self.get_level_parameter_iv(self.level() as GLint, pname)
    }

    pub fn width(&self) -> usize { unsafe {self.get_parameter_iv(gl::TEXTURE_WIDTH) as usize} }
    pub fn height(&self) -> usize { unsafe {self.get_parameter_iv(gl::TEXTURE_HEIGHT) as usize} }
    pub fn depth(&self) -> usize { unsafe {self.get_parameter_iv(gl::TEXTURE_DEPTH) as usize} }

    pub fn dim(&self) -> T::Dim {
        let coords = T::Dim::dim();
        T::Dim::new(
            if coords>0 {self.width()} else {1},
            if coords>1 {self.height()} else {1},
            if coords>2 {self.depth()} else {1},
        )
    }

    pub fn base_width(&self) -> usize { unsafe {self.get_level_parameter_iv(0, gl::TEXTURE_WIDTH) as usize} }
    pub fn base_height(&self) -> usize { unsafe {self.get_level_parameter_iv(0, gl::TEXTURE_HEIGHT) as usize} }
    pub fn base_depth(&self) -> usize { unsafe {self.get_level_parameter_iv(0, gl::TEXTURE_DEPTH) as usize} }

    pub fn base_dim(&self) -> T::Dim {
        let coords = T::Dim::dim();
        T::Dim::new(
            if coords>0 {self.base_width()} else {1},
            if coords>1 {self.base_height()} else {1},
            if coords>2 {self.base_depth()} else {1},
        )
    }

}

impl<'a,F:InternalFormat,T:PixelTransferTarget<F>> TexImage<'a,F,T> {

    unsafe fn pack<P:Pixel<F::ClientFormat>>(
        &self, settings:PixelStoreSettings, pixels: PixelPtrMut<[P]>
    ) {

        settings.apply_packing();

        let (fmt, ty) = P::format().format_type();
        let (id, ptr) = match pixels {
            PixelPtrMut::Slice(slice) => (None, slice as *mut GLvoid),
            PixelPtrMut::Buffer(buf, offset) => (Some(buf), offset as *mut GLvoid),
        };

        id.map(|i| gl::BindBuffer(gl::PIXEL_PACK_BUFFER, i));
        T::bind_loc_level().map_bind(self,
            |_| gl::GetTexImage(self.face.into(), self.level() as GLsizei, fmt.into(), ty.into(), ptr)
        );
        id.map(|_| gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0));

    }

    pub fn get_image<I:TexImageDst<F>>(&self, data: &mut I) {
        dest_size_check(self.dim(), data);
        unsafe { self.pack(data.settings(), data.pixels_mut()); }
    }

    pub fn into_image<I:OwnedTexImage<F>>(&self, hint:I::Hint) -> I where T::GL: Supports<I::GL> {
        unsafe {
            let dim = self.dim();
            I::from_gl(
                &assume_supported(), hint,
                [dim.width(), dim.height(), dim.depth()],
                |settings, ptr| self.pack(settings, ptr)
            )
        }
    }

    pub fn try_into_image<I:OwnedTexImage<F>>(&self, hint:I::Hint) -> Result<I,GLError> {
        unsafe {
            let dim = self.dim();
            let gl = upgrade_to(&self.gl())?;
            Ok(
                I::from_gl(
                    &gl, hint,
                    [dim.width(), dim.height(), dim.depth()],
                    |settings, ptr| self.pack(settings, ptr)
                )
            )
        }
    }

}


impl<'a,F,T:TextureTarget<F>> TexImageMut<'a,F,T> {
    pub fn id(&self) -> GLuint { self.id }
    pub fn gl(&self) -> T::GL { unsafe {assume_supported()} }
    pub fn level(&self) -> GLuint { self.level }

    pub fn as_immut(&self) -> TexImage<F,T> { TexImage::from(self) }
    pub fn as_mut(&mut self) -> TexImageMut<F,T> { TexImageMut::from(self) }

    pub fn width(&self) -> usize { self.as_immut().width() }
    pub fn height(&self) -> usize { self.as_immut().height() }
    pub fn depth(&self) -> usize { self.as_immut().depth() }

    pub fn dim(&self) -> T::Dim { self.as_immut().dim() }

    pub fn base_width(&self) -> usize { self.as_immut().base_width() }
    pub fn base_height(&self) -> usize { self.as_immut().base_height() }
    pub fn base_depth(&self) -> usize { self.as_immut().base_depth() }

    pub fn base_dim(&self) -> T::Dim { self.as_immut().base_dim() }

}

impl<'a,F:InternalFormat,T:PixelTransferTarget<F>> TexImageMut<'a,F,T> {

    unsafe fn unpack<
        GL:FnOnce(GLenum, [GLsizei;3], GLenum, GLenum, *const GLvoid),
        I:TexImageSrc<F>
    >(&self, data: &I, gl:GL) {
        data.settings().apply_unpacking();


        let (format, ty) = I::Pixel::format().format_type();
        let (format, ty) = (format.into(), ty.into());

        let (w, h, d) = (data.width(), data.height(), data.depth());
        let dim = [w.try_into().unwrap(), h.try_into().unwrap(), d.try_into().unwrap()];

        let (id, ptr) = match data.pixels() {
            PixelPtr::Slice(slice) => (None, slice as *const GLvoid),
            PixelPtr::Buffer(buf, offset) => (Some(buf), offset as *const GLvoid),
        };

        id.map(|i| gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, i));
        T::bind_loc_level_mut().map_bind(self, |_| gl(self.face.into(), dim, format, ty, ptr));
        id.map(|_| gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0));
    }

    pub(super) unsafe fn image_unchecked<I:TexImageSrc<F>>(&mut self, data: &I) {

        if data.pixel_count()==0 { panic!("Attempted to create a zero-sized texture image"); }
        if T::glenum() == gl::TEXTURE_CUBE_MAP_ARRAY && data.depth()%6 != 0 {
            panic!("Attempted to make a cube-map array with a depth not divisible by 6 ");
        }

        if T::Dim::dim()==1 && data.height()!=1 { panic!("Attempted to create a 1D texture from a 2D image"); }
        if T::Dim::dim()==1 && data.depth()!=1  { panic!("Attempted to create a 1D texture from a 3D image"); }
        if T::Dim::dim()==2 && data.depth()!=1  { panic!("Attempted to create a 2D texture from a 3D image"); }

        self.unpack(
            data,
            |face, [w,h,d], fmt, ty, ptr| {
                match T::Dim::dim() {
                    1 => gl::TexImage1D(face, 0, F::glenum() as GLint, w, 0, fmt, ty, ptr),
                    2 => gl::TexImage2D(face, 0, F::glenum() as GLint, w, h, 0, fmt, ty, ptr),
                    3 => gl::TexImage3D(face, 0, F::glenum() as GLint, w, h, d, 0, fmt, ty, ptr),
                    n => panic!("{}D Textures not supported", n)
                }
            }
        )
    }

    unsafe fn sub_image_unchecked<I:TexImageSrc<F>>(&mut self, offset:T::Dim, data: &I) {

        if data.pixel_count()==0 { return; }
        let (x, y, z) = (offset.width() as GLsizei, offset.height() as GLsizei, offset.depth() as GLsizei);

        self.unpack(
            data,
            |face, [w,h,d], fmt, ty, ptr| {
                match T::Dim::dim() {
                    1 => gl::TexSubImage1D(face, x, w, 0, fmt, ty, ptr),
                    2 => gl::TexSubImage2D(face, x,y, w,h, 0, fmt, ty, ptr),
                    3 => gl::TexSubImage3D(face, x,y,z, w,h,d, 0, fmt, ty, ptr),
                    n => panic!("{}D Textures not supported", n)
                }
            }
        )

    }

    pub fn image<I:TexImageSrc<F>>(&mut self, data: &I) {
        //get the current dimensions
        let current_dim = self.dim();

        //check if this image hasn't been initialized yet
        if current_dim.pixels()==0 {
            //if so, run glTexImage*D
            let dim = self.base_dim().minimized(self.level());
            size_check(dim, data);
            unsafe { self.image_unchecked(data) }
        } else {
            //else, run glTexSubImage*D
            size_check(current_dim, data);
            unsafe { self.sub_image_unchecked(T::Dim::new(0,0,0), data) }
        }
    }

    pub fn sub_image<I:TexImageSrc<F>>(&mut self, offset:T::Dim, data: &I) {
        source_size_check(offset, self.dim(), data);
        unsafe { self.sub_image_unchecked(offset, data) }
    }

    pub fn get_image<I:TexImageDst<F>>(&self, data: &mut I) {
        self.as_immut().get_image(data);
    }

    pub fn into_image<I:OwnedTexImage<F>>(&self, hint:I::Hint) -> I where T::GL: Supports<I::GL> {
        self.as_immut().into_image(hint)
    }

    pub fn try_into_image<I:OwnedTexImage<F>>(&self, hint:I::Hint) -> Result<I,GLError> where {
        self.as_immut().try_into_image(hint)
    }

}
