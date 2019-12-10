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


#[repr(C)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub struct TexImage<'a,F,T:TextureTarget<F>> {
    pub(super) id: GLuint,
    pub(super) level: GLuint,
    pub(super) face: <T as ImageSelector>::Selection,
    pub(super) tex: PhantomData<&'a Texture<F,T>>
}

#[repr(C)]
pub struct TexImageMut<'a,F,T:TextureTarget<F>> {
    pub(super) id: GLuint,
    pub(super) level: GLuint,
    pub(super) face: <T as ImageSelector>::Selection,
    pub(super) tex: PhantomData<&'a mut Texture<F,T>>
}

impl<'a,F,T:TextureTarget<F>> !Sync for TexImage<'a,F,T> {}
impl<'a,F,T:TextureTarget<F>> !Send for TexImage<'a,F,T> {}
impl<'a,F,T:TextureTarget<F>> !Sync for TexImageMut<'a,F,T> {}
impl<'a,F,T:TextureTarget<F>> !Send for TexImageMut<'a,F,T> {}

impl<'a,'b:'a,F,T:TextureTarget<F>> From<&'a TexImage<'b,F,T>> for TexImage<'b,F,T> {
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
