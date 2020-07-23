use super::*;
use crate::context::*;
use crate::pixel::*;
use crate::image::*;
use crate::buffer::*;

use std::mem::*;
use std::ops::*;
use std::marker::*;

pub use self::target::*;
pub use self::dim::*;
pub use self::uninit::*;
pub use self::image::*;
pub use self::pack::*;
pub use self::mipmap::*;
pub use self::swizzle::*;
pub use self::params::*;

pub mod target;
mod dim;
mod uninit;
mod mipmap;
mod image;
mod pack;
mod unpack;
mod swizzle;
mod params;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(self) struct ActiveTexture(GLuint);

impl Display for ActiveTexture {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        write!(f, "Active Texture #{}", self.0)
    }
}

impl<F, T:TextureTarget<F>> Target<Texture<F,T>> for ActiveTexture {
    fn target_id(self) -> GLenum { T::glenum() }
    unsafe fn bind(self, tex: &Texture<F,T>) { gl::BindTexture(T::glenum(), tex.id()); }
    unsafe fn unbind(self) { gl::BindTexture(T::glenum(), 0); }
}

impl<'a, F, T:TextureTarget<F>> Target<TexImage<'a,F,T>> for ActiveTexture {
    fn target_id(self) -> GLenum { T::glenum() }
    unsafe fn bind(self, tex: &TexImage<'a,F,T>) { gl::BindTexture(T::glenum(), tex.id()); }
    unsafe fn unbind(self) { gl::BindTexture(T::glenum(), 0); }
}

impl<'a, F, T:TextureTarget<F>> Target<TexImageMut<'a,F,T>> for ActiveTexture {
    fn target_id(self) -> GLenum { T::glenum() }
    unsafe fn bind(self, tex: &TexImageMut<'a,F,T>) { gl::BindTexture(T::glenum(), tex.id()); }
    unsafe fn unbind(self) { gl::BindTexture(T::glenum(), 0); }
}

pub(self) static mut TEXTURE0: BindingLocation<ActiveTexture> = unsafe {
    BindingLocation::new(ActiveTexture(0))
};





#[repr(transparent)]
pub struct Texture<F, T:TextureTarget<F>> {
    id: GLuint,
    phantom: PhantomData<(F, T, *const ())>
}

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
    unsafe fn parameter<E:GLEnum>(&mut self, pname:GLenum, param: E) {
        if gl::TextureParameteri::is_loaded() {
            gl::TextureParameteri(self.id(), pname, Into::<GLenum>::into(param) as GLint);
        } else {
            TEXTURE0.map_bind(self,
                |b| gl::TexParameteri(b.target_id(), pname, Into::<GLenum>::into(param) as GLint)
            );
        }
    }

    #[inline]
    unsafe fn parameter_iv(&mut self, pname:GLenum, params: *const GLint) {
        if gl::TextureParameteriv::is_loaded() {
            gl::TextureParameteriv(self.id(), pname, params);
        } else {
            TEXTURE0.map_bind(self, |b| gl::TexParameteriv(b.target_id(), pname, params));
        }
    }

    #[inline]
    unsafe fn parameter_fv(&mut self, pname:GLenum, params: *const GLfloat) {
        if gl::TextureParameterfv::is_loaded() {
            gl::TextureParameterfv(self.id(), pname, params);
        } else {
            TEXTURE0.map_bind(self, |b| gl::TexParameterfv(b.target_id(), pname, params));
        }
    }

    #[inline]
    unsafe fn parameter_i_iv(&mut self, pname:GLenum, params: *const GLint) {
        if gl::TextureParameterIuiv::is_loaded() {
            gl::TextureParameterIiv(self.id(), pname, params);
        } else {
            TEXTURE0.map_bind(self, |b| gl::TexParameterIiv(b.target_id(), pname, params));
        }
    }

    #[inline]
    unsafe fn parameter_i_uiv(&mut self, pname:GLenum, params: *const GLuint) {
        if gl::TextureParameterIuiv::is_loaded() {
            gl::TextureParameterIuiv(self.id(), pname, params);
        } else {
            TEXTURE0.map_bind(self, |b| gl::TexParameterIuiv(b.target_id(), pname, params));
        }
    }

    #[inline]
    unsafe fn get_parameter_i(&self, pname:GLenum) -> GLint {
        let mut param = MaybeUninit::uninit();
        self.get_parameter_iv(pname, param.as_mut_ptr());
        param.assume_init()
    }

    #[inline]
    unsafe fn get_parameter_f(&self, pname:GLenum) -> GLfloat {
        let mut param = MaybeUninit::uninit();
        self.get_parameter_fv(pname, param.as_mut_ptr());
        param.assume_init()
    }

    #[inline]
    unsafe fn get_parameter<E:GLEnum>(&self, pname:GLenum) -> E {
        (self.get_parameter_i(pname) as GLenum).try_into().unwrap()
    }

    #[inline]
    unsafe fn get_parameter_iv(&self, pname:GLenum, param: *mut GLint) {
        if gl::GetTextureParameteriv::is_loaded() {
            gl::GetTextureParameteriv(self.id(), pname, param);
        } else {
            TEXTURE0.map_bind(self, |b| gl::GetTexParameteriv(b.target_id(), pname, param));
        }
    }

    #[inline]
    unsafe fn get_parameter_fv(&self, pname:GLenum, param: *mut GLfloat) {
        if gl::GetTextureParameterfv::is_loaded() {
            gl::GetTextureParameterfv(self.id(), pname, param);
        } else {
            TEXTURE0.map_bind(self, |b| gl::GetTexParameterfv(b.target_id(), pname, param));
        }
    }

    #[inline]
    unsafe fn get_parameter_i_iv(&self, pname:GLenum, param: *mut GLint) {
        if gl::GetTextureParameterIiv::is_loaded() {
            gl::GetTextureParameterIiv(self.id(), pname, param);
        } else {
            TEXTURE0.map_bind(self, |b| gl::GetTexParameterIiv(b.target_id(), pname, param));
        }
    }

    #[inline]
    unsafe fn get_parameter_i_uiv(&self, pname:GLenum, param: *mut GLuint) {
        if gl::GetTextureParameterIuiv::is_loaded() {
            gl::GetTextureParameterIuiv(self.id(), pname, param);
        } else {
            TEXTURE0.map_bind(self, |b| gl::GetTexParameterIuiv(b.target_id(), pname, param));
        }
    }

}

impl<F:InternalFormat, T:TextureTarget<F>> Texture<F,T> {

    pub fn immutable_format(&self) -> bool {
        unsafe { self.get_parameter_i(gl::TEXTURE_IMMUTABLE_FORMAT)!=0 }
    }

    pub fn immutable_levels(&self) -> GLuint {
        unsafe { self.get_parameter_i(gl::TEXTURE_IMMUTABLE_LEVELS) as GLuint }
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

//aliases for the different texture types

pub type Tex1D<F> = Texture<F, TEXTURE_1D>;
pub type Tex2D<F> = Texture<F, TEXTURE_2D>;
pub type Tex3D<F> = Texture<F, TEXTURE_3D>;
pub type Tex1DArray<F> = Texture<F, TEXTURE_1D_ARRAY>;
pub type Tex2DArray<F> = Texture<F, TEXTURE_2D_ARRAY>;
pub type TexRect<F> = Texture<F, TEXTURE_RECTANGLE>;
pub type TexBuf<'a,F> = Texture<F, TEXTURE_BUFFER<'a>>;
pub type TexBufMut<'a,F> = Texture<F, TEXTURE_BUFFER_MUT<'a>>;
pub type TexCubeMap<F> = Texture<F, TEXTURE_CUBE_MAP>;
pub type Tex2DMS<F,MS> = Texture<F, TEXTURE_2D_MULTISAMPLE<MS>>;
pub type Tex2DMSArray<F,MS> = Texture<F, TEXTURE_2D_MULTISAMPLE_ARRAY<MS>>;
