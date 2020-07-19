use super::*;

impl<F:InternalFormat, T:TextureTarget<F>+BaseImage> Texture<F,T> {
    pub fn base_image(&self) -> TexImage<F,T> { TexImage::from(self) }
    pub fn base_image_mut(&mut self) -> TexImageMut<F,T> { TexImageMut::from(self) }
}

impl<F:InternalFormat, T:MipmappedTarget<F>+BaseImage> Texture<F,T> {
    pub fn level(&self, level:GLuint) -> TexImage<F,T> { self._level(level, T::default()) }
    pub fn level_mut(&mut self, level:GLuint) -> TexImageMut<F,T> { self._level_mut(level, T::default()) }
}

impl<F:InternalFormat> Texture<F,TEXTURE_CUBE_MAP> where TEXTURE_CUBE_MAP:TextureTarget<F> {
    pub fn face(&self, face:CubeMapFace, level:GLuint) -> TexImage<F,TEXTURE_CUBE_MAP> { self._level(level, face) }
    pub fn face_mut(&mut self, face:CubeMapFace, level:GLuint) -> TexImageMut<F,TEXTURE_CUBE_MAP> {
        self._level_mut(level, face)
    }
}

impl<F:InternalFormat, T:MipmappedTarget<F>> Texture<F,T> {

    pub fn base_level(&self) -> GLuint {unsafe{self.get_parameter_i(gl::TEXTURE_BASE_LEVEL) as GLuint}}
    pub fn max_level(&self) -> GLuint {unsafe{self.get_parameter_i(gl::TEXTURE_MAX_LEVEL) as GLuint}}
    pub fn level_range(&self) -> RangeInclusive<GLuint> {
        self.base_level()..=self.max_level()
    }

    //TODO: return a result
    fn check_levels(&self, base:GLuint, max:GLuint, check_complete:bool) {
        //make sure the interval is ordered correctly
        if max < base { panic!("Max level lower than current base level"); }

        //make sure we don't go out of range
        if max >= self.max_levels() {
            if self.immutable_format() {
                panic!("Max level higher than allocated immutable storage");
            } else {
                panic!("Max level higher than maximum allowable mipmap levels");
            }
        }

        //make sure the mipmap levels are complete.
        let prev = self.max_level();
        if check_complete && !self.immutable_format() && max > prev {
            T::bind_loc().map_bind(self,
                |b| unsafe {
                    //Note that since we check dimensions at upload time, we only need to check
                    //if the layers are initialized
                    let mut fmt = 0;
                    for level in (prev+1).min(base) ..= max {
                        if T::glenum() == gl::TEXTURE_CUBE_MAP {
                            for face in CubeMapFace::faces() {
                                gl::GetTexLevelParameteriv(
                                    *face as GLenum, level as GLint, gl::TEXTURE_INTERNAL_FORMAT, &mut fmt
                                );
                                if fmt==0 { panic!("Mipmap level not initialized"); }
                            }
                        } else {
                            gl::GetTexLevelParameteriv(
                                b.target_id(), level as GLint, gl::TEXTURE_INTERNAL_FORMAT, &mut fmt
                            );
                            if fmt==0 { panic!("Mipmap level not initialized"); }
                        }
                    }
                }
            );
        }
    }

    pub fn set_base_level(&mut self, level: GLuint) {
        if level > self.max_level() { panic!("Base level higher than current max level"); }
        unsafe { self.parameter_iv(gl::TEXTURE_BASE_LEVEL, &(level as GLint)) }
    }

    pub fn set_max_level(&mut self, level: GLuint) {
        self.check_levels(self.base_level(), level, true);
        unsafe { self.parameter_iv(gl::TEXTURE_MAX_LEVEL, &(level as GLint)) }
    }

    fn _set_level_range<R:RangeBounds<GLuint>>(&mut self, range:R, check_complete:bool) {
        let base = match range.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => x+1,
            Bound::Unbounded => 0
        };

        let max = match range.end_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => x-1,
            Bound::Unbounded => self.max_levels()-1
        };

        self.check_levels(base, max, check_complete);

        unsafe {
            self.parameter_iv(gl::TEXTURE_BASE_LEVEL, &(base as GLint));
            self.parameter_iv(gl::TEXTURE_MAX_LEVEL, &(max as GLint));
        }
    }

    pub fn set_level_range<R:RangeBounds<GLuint>>(&mut self, range:R) {
        self._set_level_range(range, true);
    }

    pub fn generate_mipmap(&mut self) {
        unsafe {
            if gl::GenerateTextureMipmap::is_loaded() {
                gl::GenerateTextureMipmap(self.id());
            } else {
                T::bind_loc().map_bind(self, |b| gl::GenerateMipmap(b.target_id()));
            }
        }
    }

    pub fn generate_mipmap_range<R:RangeBounds<GLuint>>(&mut self, r:R) {
        let levels = self.level_range();

        self._set_level_range(r, false);
        self.generate_mipmap();

        unsafe {
            self.parameter_iv(gl::TEXTURE_BASE_LEVEL, &(*levels.start() as GLint));
            self.parameter_iv(gl::TEXTURE_MAX_LEVEL, &(*levels.end() as GLint));
        }
    }

}
