use super::*;
use format::pixel::*;
use format::image::*;

use std::mem::*;
use std::ops::*;
use std::convert::*;

pub use self::target::*;
pub use self::dim::*;
pub use self::uninit::*;
pub use self::image::*;
pub use self::pack::*;
pub use self::mipmap::*;

pub mod target;
mod dim;
mod uninit;
mod mipmap;
mod image;
mod pack;
mod unpack;

pub struct Texture<F, T:TextureTarget<F>> {
    id: GLuint,
    phantom: PhantomData<(F,T)>
}

impl<F,T:TextureTarget<F>> !Sync for Texture<F,T> {}
impl<F,T:TextureTarget<F>> !Send for Texture<F,T> {}

impl<F, T:TextureTarget<F>> Texture<F,T> {
    pub fn id(&self) -> GLuint { self.id }
    pub fn gl(&self) -> T::GL { unsafe {assume_supported()} }

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

    fn _base_image(&self) -> TexImage<F,T> { self._level(0, <T as ImageSelector>::Selection::default()) }

    fn _level(&self, layer:GLuint, face:<T as ImageSelector>::Selection) -> TexImage<F,T> {
        if layer!=0 && layer>=self.max_levels() { panic!("Mipmap level out of range"); }
        TexImage{id:self.id(), level:0, face:face, tex:PhantomData}
    }

    fn _level_mut(&mut self, layer:GLuint, face:<T as ImageSelector>::Selection) -> TexImageMut<F,T> {
        if layer!=0 && layer>=self.max_levels() { panic!("Mipmap level out of range"); }
        TexImageMut{id:self.id(), level:0, face:face, tex:PhantomData}
    }

    pub fn width(&self) -> usize { self._base_image().width() }
    pub fn height(&self) -> usize { self._base_image().height() }
    pub fn depth(&self) -> usize { self._base_image().depth() }

    pub fn dim(&self) -> T::Dim { self._base_image().dim() }

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

}

impl<F, T:TextureTarget<F>> Drop for Texture<F,T> {
    fn drop(&mut self) { unsafe { gl::DeleteTextures(1, &self.id) } }
}
