use super::*;

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
pub struct Image<'a,F,T:TextureTarget<F>> {
    pub(super) id: GLuint,
    pub(super) level: GLuint,
    pub(super) face: <T as ImageSelector>::Selection,
    pub(super) tex: PhantomData<&'a Texture<F,T>>
}

pub struct ImageMut<'a,F,T:TextureTarget<F>> {
    pub(super) id: GLuint,
    pub(super) level: GLuint,
    pub(super) face: <T as ImageSelector>::Selection,
    pub(super) tex: PhantomData<&'a mut Texture<F,T>>
}

impl<'a,F,T:TextureTarget<F>> !Sync for Image<'a,F,T> {}
impl<'a,F,T:TextureTarget<F>> !Send for Image<'a,F,T> {}
impl<'a,F,T:TextureTarget<F>> !Sync for ImageMut<'a,F,T> {}
impl<'a,F,T:TextureTarget<F>> !Send for ImageMut<'a,F,T> {}

impl<'a,'b:'a,F,T:TextureTarget<F>> From<&'a Image<'b,F,T>> for Image<'a,F,T> {
    #[inline] fn from(lvl: &'a Image<'b,F,T>) -> Self {*lvl}
}

impl<'a,F,T:TextureTarget<F>> From<ImageMut<'a,F,T>> for Image<'a,F,T> {
    #[inline] fn from(lvl: ImageMut<'a,F,T>) -> Self {
        Image{id:lvl.id, level:lvl.level, face:lvl.face, tex:PhantomData}
    }
}

impl<'a,'b:'a,F,T:TextureTarget<F>> From<&'a ImageMut<'b,F,T>> for Image<'a,F,T> {
    #[inline] fn from(lvl: &'a ImageMut<'b,F,T>) -> Self {
        Image{id:lvl.id, level:lvl.level, face:lvl.face, tex:PhantomData}
    }
}

impl<'a,'b:'a,F,T:TextureTarget<F>> From<&'a mut ImageMut<'b,F,T>> for ImageMut<'a,F,T> {
    #[inline] fn from(lvl: &'a mut ImageMut<'b,F,T>) -> Self {
        ImageMut{id:lvl.id, level:lvl.level, face:lvl.face, tex:PhantomData}
    }
}

impl<'a,F,T:TextureTarget<F>+BaseImage> From<&'a Texture<F,T>> for Image<'a,F,T> {
    #[inline] fn from(tex: &'a Texture<F,T>) -> Self {
        Image{id:tex.id, level:0, face:T::default(), tex:PhantomData}
    }
}

impl<'a,F,T:TextureTarget<F>+BaseImage> From<&'a mut Texture<F,T>> for ImageMut<'a,F,T> {
    #[inline] fn from(tex: &'a mut Texture<F,T>) -> Self {
        ImageMut{id:tex.id, level:0, face:T::default(), tex:PhantomData}
    }
}

impl<'a,F,T:TextureTarget<F>> Image<'a,F,T> {
    pub fn id(&self) -> GLuint { self.id }
    pub fn level(&self) -> GLuint { self.level }

    #[inline]
    unsafe fn get_parameter_iv(&self, pname:GLenum) -> GLint {
        let mut param = MaybeUninit::uninit();
        if gl::GetTextureLevelParameteriv::is_loaded() {
            gl::GetTextureLevelParameteriv(self.id(), self.level() as GLint, pname, param.as_mut_ptr());
        } else {
            T::bind_loc_level().map_bind(self,
                |b| {
                    let target = if b.target_id()==gl::TEXTURE_CUBE_MAP {
                        gl::TEXTURE_CUBE_MAP_POSITIVE_X
                    } else {
                        b.target_id()
                    };
                    gl::GetTexLevelParameteriv(target, self.level() as GLint, pname, param.as_mut_ptr())
                }
            );
        }
        param.assume_init()
    }

    pub fn width(&self) -> usize { unsafe {self.get_parameter_iv(gl::TEXTURE_WIDTH) as usize} }
    pub fn height(&self) -> usize { unsafe {self.get_parameter_iv(gl::TEXTURE_HEIGHT) as usize} }
    pub fn depth(&self) -> usize { unsafe {self.get_parameter_iv(gl::TEXTURE_DEPTH) as usize} }

    pub fn dim(&self) -> T::Dim {
        let coords = T::Dim::dim();
        T::Dim::new(
            if coords>0 {self.width()} else {0},
            if coords>1 {self.height()} else {0},
            if coords>2 {self.depth()} else {0},
        )
    }

}

impl<'a,F:InternalFormat,T:PixelTransferTarget<F>> Image<'a,F,T> {

    pub fn get_image<I:ImageDst<F::ClientFormat>>(&self, data: &mut I) {
        unsafe {

            size_check::<_,F,_>(self.dim(), data);
            data.settings().apply_packing();

            let (id, ptr, client_format) = match data.pixels_mut() {
                PixelPtrMut::Slice(f, slice) => (None, slice, f),
                PixelPtrMut::Buffer(f, buf, offset) => (Some(buf), offset, f),
            };
            id.map(|i| gl::BindBuffer(gl::PIXEL_PACK_BUFFER, i));

            let (format, ty) = client_format.format_type();

            T::bind_loc_level().map_bind(self,
                |_| gl::GetTexImage(self.face.into(), self.level() as GLsizei, format.into(), ty.into(), ptr)
            );

            id.map(|_| gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0));

        }
    }

}


impl<'a,F,T:TextureTarget<F>> ImageMut<'a,F,T> {
    pub fn id(&self) -> GLuint { self.id }
    pub fn level(&self) -> GLuint { self.level }

    pub fn as_immut(&self) -> Image<F,T> { Image::from(self) }
    pub fn as_mut(&mut self) -> ImageMut<F,T> { ImageMut::from(self) }

    pub fn width(&self) -> usize { self.as_immut().width() }
    pub fn height(&self) -> usize { self.as_immut().height() }
    pub fn depth(&self) -> usize { self.as_immut().depth() }

    pub fn dim(&self) -> T::Dim { self.as_immut().dim() }

}

impl<'a,F:InternalFormat,T:PixelTransferTarget<F>> ImageMut<'a,F,T> {
    pub(super) unsafe fn image_unchecked<I:ImageSrc<F::ClientFormat>>(&mut self, data: &I) {

        data.settings().apply_unpacking();

        let (id, ptr, client_format) = match data.pixels() {
            PixelPtr::Slice(f, slice) => (None, slice, f),
            PixelPtr::Buffer(f, buf, offset) => (Some(buf), offset, f),
        };
        id.map(|i| gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, i));

        let (format, ty) = client_format.format_type().into();
        let (internal, format, ty) = (F::glenum() as GLint, format.into(), ty.into());

        let (w, h, d) = (usize::from(data.width()), usize::from(data.height()), usize::from(data.depth()));
        let (w, h, d) = (w as GLsizei, h as GLsizei, d as GLsizei);

        T::bind_loc_level_mut().map_bind(self,
            |_| match T::Dim::dim() {
                1 => gl::TexImage1D(self.face.into(), 0, internal, w, 0, format, ty, ptr),
                2 => gl::TexImage2D(self.face.into(), 0, internal, w, h, 0, format, ty, ptr),
                3 => gl::TexImage3D(self.face.into(), 0, internal, w, h, d, 0, format, ty, ptr),
                n => panic!("{}D Textures not supported", n)
            }
        );

        id.map(|_| gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0));
    }
}
