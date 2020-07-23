use super::*;
use crate::context::*;
use crate::pixel::*;

use std::mem::*;

pub use self::fragment::*;

mod fragment;
mod bitplane;

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

impl<'a, DS, F: Fragment<'a>> Framebuffer<'a,DS,F> {

    pub fn id(&self) -> GLuint { self.id }
    pub fn gl(&self) -> GL30 { unsafe {assume_supported()} }

    pub fn delete(self) { drop(self); }
    pub fn delete_framebuffers(rb: Box<[Self]>) {
        if rb.len()==0 {return;}
        unsafe {
            let ids: Box<[GLuint]> = transmute(rb);
            gl::DeleteFramebuffers(ids.len() as GLsizei, &ids[0])
        }
    }

}

impl<'a, DS, F:'a> Drop for Framebuffer<'a,DS,F> {
    fn drop(&mut self) { unsafe { gl::DeleteFramebuffers(1, &self.id)} }
}
