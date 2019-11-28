use super::*;

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub struct Level<'a,F,T:TextureTarget<F>> {
    id: GLuint,
    level: GLuint,
    tex: PhantomData<&'a Texture<F,T>>
}

pub struct LevelMut<'a,F,T:TextureTarget<F>> {
    id: GLuint,
    level: GLuint,
    tex: PhantomData<&'a mut Texture<F,T>>
}

impl<'a,F,T:TextureTarget<F>> !Sync for Level<'a,F,T> {}
impl<'a,F,T:TextureTarget<F>> !Send for Level<'a,F,T> {}
impl<'a,F,T:TextureTarget<F>> !Sync for LevelMut<'a,F,T> {}
impl<'a,F,T:TextureTarget<F>> !Send for LevelMut<'a,F,T> {}

impl<'a,'b:'a,F,T:TextureTarget<F>> From<&'a Level<'b,F,T>> for Level<'a,F,T> {
    #[inline] fn from(lvl: &'a Level<'b,F,T>) -> Self {*lvl}
}

impl<'a,F,T:TextureTarget<F>> From<LevelMut<'a,F,T>> for Level<'a,F,T> {
    #[inline] fn from(lvl: LevelMut<'a,F,T>) -> Self { Level{id:lvl.id, level:lvl.level, tex:PhantomData} }
}

impl<'a,'b:'a,F,T:TextureTarget<F>> From<&'a LevelMut<'b,F,T>> for Level<'a,F,T> {
    #[inline] fn from(lvl: &'a LevelMut<'b,F,T>) -> Self { Level{id:lvl.id, level:lvl.level, tex:PhantomData} }
}

impl<'a,'b:'a,F,T:TextureTarget<F>> From<&'a mut LevelMut<'b,F,T>> for LevelMut<'a,F,T> {
    #[inline] fn from(lvl: &'a mut LevelMut<'b,F,T>) -> Self { LevelMut{id:lvl.id, level:lvl.level, tex:PhantomData} }
}

impl<'a,F,T:TextureTarget<F>> From<&'a Texture<F,T>> for Level<'a,F,T> {
    #[inline] fn from(lvl: &'a Texture<F,T>) -> Self { Level{id:lvl.id, level:0, tex:PhantomData} }
}

impl<'a,F,T:TextureTarget<F>> From<&'a mut Texture<F,T>> for LevelMut<'a,F,T> {
    #[inline] fn from(lvl: &'a mut Texture<F,T>) -> Self { LevelMut{id:lvl.id, level:0, tex:PhantomData} }
}

impl<'a,F,T:TextureTarget<F>> Level<'a,F,T> {
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

impl<'a,F,T:TextureTarget<F>> LevelMut<'a,F,T> {
    pub fn id(&self) -> GLuint { self.id }
    pub fn level(&self) -> GLuint { self.level }

    pub fn as_immut(&self) -> Level<F,T> { Level::from(self) }
    pub fn as_mut(&mut self) -> LevelMut<F,T> { LevelMut::from(self) }

    pub fn width(&self) -> usize { self.as_immut().width() }
    pub fn height(&self) -> usize { self.as_immut().height() }
    pub fn depth(&self) -> usize { self.as_immut().depth() }

    pub fn dim(&self) -> T::Dim { self.as_immut().dim() }

}

impl<'a,F:InternalFormat,T:ImageTarget<F>> LevelMut<'a,F,T> {
    pub(super) unsafe fn image_dim<P:PixelData<F::ClientFormat>>(&mut self, dim:T::Dim, data: &P) {

        size_check::<_,F,_>(dim, data);
        apply_unpacking_settings(data);

        let mut target = BufferTarget::PixelUnpackBuffer.as_loc();
        let pixels = data.pixels();
        let (binding, ptr, client_format) = match pixels {
            Pixels::Slice(f, slice) => (None, slice as *const [P::Pixel] as *const GLvoid, f),
            Pixels::Buffer(f, ref slice) => (Some(target.bind(slice)), slice.offset() as *const GLvoid, f),
        };

        let (format, ty) = client_format.format_type().into();
        let (internal, format, ty) = (F::glenum() as GLint, format.into(), ty.into());
        let (w, h, d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);

        T::bind_loc_level_mut().map_bind(self,
            |b| match T::Dim::dim() {
                1 => gl::TexImage1D(b.target_id(), 0, internal, w, 0, format, ty, ptr),
                2 => gl::TexImage2D(b.target_id(), 0, internal, w, h, 0, format, ty, ptr),
                3 => gl::TexImage3D(b.target_id(), 0, internal, w, h, d, 0, format, ty, ptr),
                n => panic!("{}D Textures not supported", n)
            }
        );

        drop(binding);
    }
}
