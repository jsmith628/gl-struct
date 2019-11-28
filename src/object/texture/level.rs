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


}
