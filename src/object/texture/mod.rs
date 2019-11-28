use super::*;
use format::image_format::*;

use std::mem::*;
use std::ops::*;

pub use self::target::*;
pub use self::dim::*;
pub use self::uninit::*;
pub use self::level::*;

mod target;
mod dim;
mod uninit;
mod level;

pub struct Texture<F, T:TextureTarget<F>> {
    id: GLuint,
    phantom: PhantomData<(F,T)>
}

impl<F,T:TextureTarget<F>> !Sync for Texture<F,T> {}
impl<F,T:TextureTarget<F>> !Send for Texture<F,T> {}

impl<F, T:TextureTarget<F>> Texture<F,T> {
    pub fn id(&self) -> GLuint { self.id }

    pub fn delete(self) { drop(self); }
    pub fn delete_textures(tex: Box<[Self]>) {
        if tex.len()==0 {return;}
        unsafe {
            let ids: Box<[GLuint]> = transmute(tex);
            gl::DeleteTextures(ids.len() as GLsizei, &ids[0])
        }
    }

    #[inline]
    unsafe fn parameter_iv(&mut self, pname:GLenum, params: *const GLint) {
        if gl::TextureParameteriv::is_loaded() {
            gl::TextureParameteriv(self.id(), pname, params);
        } else {
            T::bind_loc().map_bind(self, |b| gl::TexParameteriv(b.target_id(), pname, params));
        }
    }

    #[inline]
    unsafe fn get_parameter_iv(&self, pname:GLenum) -> GLint {
        let mut param = MaybeUninit::uninit();
        if gl::GetTextureParameteriv::is_loaded() {
            gl::GetTextureParameteriv(self.id(), pname, param.as_mut_ptr());
        } else {
            T::bind_loc().map_bind(self, |b| gl::GetTexParameteriv(b.target_id(), pname, param.as_mut_ptr()));
        }
        param.assume_init()
    }

}

impl<F:InternalFormat, T:TextureTarget<F>> Texture<F,T> {

    pub fn immutable_format(&self) -> bool {
        unsafe { self.get_parameter_iv(gl::TEXTURE_IMMUTABLE_FORMAT)!=0 }
    }

    pub fn immutable_levels(&self) -> GLuint {
        unsafe { self.get_parameter_iv(gl::TEXTURE_IMMUTABLE_LEVELS) as GLuint }
    }

    pub fn width(&self) -> usize { self.base_image().width() }
    pub fn height(&self) -> usize { self.base_image().height() }
    pub fn depth(&self) -> usize { self.base_image().depth() }

    pub fn dim(&self) -> T::Dim { self.base_image().dim() }

    pub fn max_levels(&self) -> GLuint {
        if T::mipmapped() {
            if self.immutable_format() {
                self.immutable_levels()
            } else {
                self.dim().max_levels()
            }
        } else {
            1
        }
    }

    pub fn base_image(&self) -> Level<F,T> { Level::from(self) }
    pub fn base_image_mut(&mut self) -> LevelMut<F,T> { LevelMut::from(self) }

}

impl<F:InternalFormat, T:MipmappedTarget<F>> Texture<F,T> {

    pub fn base_level(&self) -> GLuint {unsafe{self.get_parameter_iv(gl::TEXTURE_BASE_LEVEL) as GLuint}}
    pub fn max_level(&self) -> GLuint {unsafe{self.get_parameter_iv(gl::TEXTURE_MAX_LEVEL) as GLuint}}
    pub fn level_range(&self) -> RangeInclusive<GLuint> {
        self.base_level()..=self.max_level()
    }

    fn check_levels(&self, base:GLuint, max:GLuint) {
        if max < base { panic!("Max level lower than current base level"); }
        if max >= self.max_levels() {
            if self.immutable_format() {
                panic!("Max level higher than allocated immutable storage");
            } else {
                panic!("Max level higher than maximum allowable mipmap levels");
            }
        }
    }

    pub fn set_base_level(&mut self, level: GLuint) {
        if level > self.max_level() { panic!("Base level higher than current max level"); }
        unsafe { self.parameter_iv(gl::TEXTURE_BASE_LEVEL, &(level as GLint)) }
    }

    pub fn set_max_level(&mut self, level: GLuint) {
        self.check_levels(self.base_level(), level);
        unsafe { self.parameter_iv(gl::TEXTURE_MAX_LEVEL, &(level as GLint)) }
    }

    pub fn set_level_range<R:RangeBounds<GLuint>>(&mut self, range:R) {
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

        self.check_levels(base, max);

        unsafe {
            self.parameter_iv(gl::TEXTURE_BASE_LEVEL, &(base as GLint));
            self.parameter_iv(gl::TEXTURE_MAX_LEVEL, &(max as GLint));
        }
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

        self.set_level_range(r);
        self.generate_mipmap();

        unsafe {
            self.parameter_iv(gl::TEXTURE_BASE_LEVEL, &(*levels.start() as GLint));
            self.parameter_iv(gl::TEXTURE_MAX_LEVEL, &(*levels.end() as GLint));
        }
    }

}



impl<F, T:TextureTarget<F>> Drop for Texture<F,T> {
    fn drop(&mut self) { unsafe { gl::DeleteTextures(1, &self.id) } }
}
