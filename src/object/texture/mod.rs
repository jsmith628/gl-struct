use super::*;
use format::image_format::*;

use std::mem::*;

pub use self::target::*;
pub use self::dim::*;
pub use self::uninit::*;

mod target;
mod dim;
mod uninit;

pub struct Texture<F, T:TextureTarget> {
    id: GLuint,
    phantom: PhantomData<(F,T)>
}

impl<F, T:TextureTarget> Texture<F,T> {
    pub fn id(&self) -> GLuint { self.id }

    pub fn delete(self) { drop(self); }
    pub fn delete_textures(tex: Box<[Self]>) {
        if tex.len()==0 {return;}
        unsafe {
            let ids: Box<[GLuint]> = transmute(tex);
            gl::DeleteTextures(ids.len() as GLsizei, &ids[0])
        }
    }


    unsafe fn parameter_iv(&mut self, pname:GLenum, params: *const GLint) {
        if gl::TextureParameteriv::is_loaded() {
            gl::TextureParameteriv(self.id(), pname, params);
        } else {
            T::bind_loc().map_bind(self, |b| gl::TexParameteriv(b.target_id(), pname, params));
        }
    }

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

impl<F:InternalFormat, T:TextureTarget> Texture<F,T> {

    pub fn immutable_format(&self) -> bool {
        unsafe { self.get_parameter_iv(gl::TEXTURE_IMMUTABLE_FORMAT)!=0 }
    }

    pub fn immutable_levels(&self) -> GLuint {
        unsafe { self.get_parameter_iv(gl::TEXTURE_IMMUTABLE_LEVELS) as GLuint }
    }

}



impl<F, T:TextureTarget> Drop for Texture<F,T> {
    fn drop(&mut self) { unsafe { gl::DeleteTextures(1, &self.id) } }
}
