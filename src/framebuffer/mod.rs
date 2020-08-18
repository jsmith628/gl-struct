use super::*;
use crate::version::*;
use crate::pixel::*;
use crate::glsl::*;
use crate::texture::*;
use crate::renderbuffer::*;

use std::mem::*;

pub use self::fragment::*;
pub use self::attachment::*;
pub use self::image::*;

use self::fragment::Layered;
use self::fragment::Multisampled;

mod fragment;
mod attachment;
mod image;

glenum! {
    #[non_exhaustive]
    pub enum FramebufferTarget {
        [DrawFramebuffer DRAW_FRAMEBUFFER "Draw Framebuffer"],
        [ReadFramebuffer READ_FRAMEBUFFER "Read Framebuffer"]
    }
}

impl<'a, DS, F:'a> Target<Framebuffer<'a,DS,F>> for FramebufferTarget {
    fn target_id(self) -> GLenum { self.into() }
    unsafe fn bind(self, fb: &Framebuffer<'a,DS,F>) { gl::BindFramebuffer(self.into(), fb.id()) }
    unsafe fn unbind(self) { gl::BindFramebuffer(self.into(), 0) }
}

impl<'a,'b,F:'b> Target<FramebufferAttachment<'a,'b,F>> for FramebufferTarget {
    fn target_id(self) -> GLenum { self.into() }
    unsafe fn bind(self, fb: &FramebufferAttachment<'a,'b,F>) { gl::BindFramebuffer(self.into(), fb.id()) }
    unsafe fn unbind(self) { gl::BindFramebuffer(self.into(), 0) }
}

impl<'a,'b,F:'b> Target<FramebufferAttachmentMut<'a,'b,F>> for FramebufferTarget {
    fn target_id(self) -> GLenum { self.into() }
    unsafe fn bind(self, fb: &FramebufferAttachmentMut<'a,'b,F>) { gl::BindFramebuffer(self.into(), fb.id()) }
    unsafe fn unbind(self) { gl::BindFramebuffer(self.into(), 0) }
}

static mut READ_FRAMEBUFFER: BindingLocation<FramebufferTarget> = unsafe {
    BindingLocation::new(FramebufferTarget::DrawFramebuffer)
};

static mut DRAW_FRAMEBUFFER: BindingLocation<FramebufferTarget> = unsafe {
    BindingLocation::new(FramebufferTarget::ReadFramebuffer)
};

pub struct Framebuffer<'a, DS, F:'a> {
    id: GLuint,
    attachments: PhantomData<&'a mut (DS, F)>
}

impl<'a> Framebuffer<'a,!,!> {

    pub fn gen(#[allow(unused_variables)] gl: &GL30) -> GLuint {
        unsafe {
            let mut fb = MaybeUninit::uninit();
            gl::GenFramebuffers(1, fb.as_mut_ptr());
            fb.assume_init()
        }
    }

    pub fn gen_framebuffers(#[allow(unused_variables)] gl: &GL30, n: GLuint) -> Box<[GLuint]> {
        if n==0 { return Box::new([]); }
        unsafe {
            let mut fb = Box::new_uninit_slice(n as usize);
            gl::GenFramebuffers(fb.len().try_into().unwrap(), MaybeUninit::first_ptr_mut(&mut *fb));
            fb.assume_init()
        }
    }

    pub fn create(#[allow(unused_variables)] gl: &GL30) -> Self {
        let mut fb: MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe {
            if gl::CreateRenderbuffers::is_loaded() {
                gl::CreateFramebuffers(1, fb.as_mut_ptr() as *mut GLuint);
            } else {
                gl::GenFramebuffers(1, fb.as_mut_ptr() as *mut GLuint);
                gl::BindFramebuffer(gl::FRAMEBUFFER, fb.get_mut().id());
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            }
            fb.assume_init()
        }
    }

    pub fn create_framebuffers(#[allow(unused_variables)] gl: &GL30, n: GLuint) -> Box<[Self]> {
        if n==0 { return Box::new([]); }
        let mut fb:Box<[MaybeUninit<Self>]> = Box::new_uninit_slice(n as usize);
        unsafe {
            if gl::CreateFramebuffers::is_loaded() {
                gl::CreateFramebuffers(fb.len().try_into().unwrap(), fb[0].as_mut_ptr() as *mut GLuint);
            } else {
                gl::GenFramebuffers(fb.len().try_into().unwrap(), fb[0].as_mut_ptr() as *mut GLuint);
                for t in fb.iter_mut() { gl::BindFramebuffer(gl::FRAMEBUFFER, t.get_mut().id()) }
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            }
            fb.assume_init()
        }
    }
}

impl<'a, DS, F:'a> Framebuffer<'a,DS,F> {

    pub fn id(&self) -> GLuint { self.id }
    pub fn gl(&self) -> GL30 { unsafe {assume_supported()} }

    pub fn delete(self) { drop(self); }
    pub fn delete_framebuffers(rb: Box<[Self]>) {
        if rb.is_empty() {return;}
        unsafe {
            let ids: Box<[GLuint]> = transmute(rb);
            gl::DeleteFramebuffers(ids.len() as GLsizei, &ids[0])
        }
    }

}

impl<'a, DS, F:'a> Drop for Framebuffer<'a,DS,F> {
    fn drop(&mut self) { unsafe { gl::DeleteFramebuffers(1, &self.id)} }
}
