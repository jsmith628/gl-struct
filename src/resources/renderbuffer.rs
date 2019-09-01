use super::*;
use format::*;

use std::marker::PhantomData;

glenum! {

    pub enum RenderbufferTarget {
        [Renderbuffer RENDERBUFFER "Renderbuffer"]
    }

    pub enum RenderbufferParameter {
        [Width RENDERBUFFER_WIDTH "Width"],
        [Height RENDERBUFFER_HEIGHT "Height"],
        [InternalFormat RENDERBUFFER_INTERNAL_FORMAT "Internal Format"],
        [Samples RENDERBUFFER_SAMPLES "Samples"],
        [RedSize RENDERBUFFER_RED_SIZE "Red Size"],
        [GreenSize RENDERBUFFER_GREEN_SIZE "Green Size"],
        [BlueSize RENDERBUFFER_BLUE_SIZE "Blue Size"],
        [AlphaSize RENDERBUFFER_ALPHA_SIZE "Alpha Size"],
        [DepthSize RENDERBUFFER_DEPTH_SIZE "Depth Size"],
        [StencilSize RENDERBUFFER_STENCIL_SIZE "Stencil Size"]
    }

}

gl_resource!{
    pub struct RawRenderbuffer {
        gl = GL30,
        target = RenderbufferTarget,
        gen = GenRenderbuffers,
        bind = BindRenderbuffer,
        is = IsRenderbuffer,
        delete = DeleteRenderbuffers
    }
}

static mut TARGET: BindingLocation<RawRenderbuffer> = BindingLocation(RenderbufferTarget::Renderbuffer);

pub struct Renderbuffer<T:InternalFormat> {
    raw: RawRenderbuffer,
    fmt: PhantomData<T>
}

impl<T:InternalFormat> Renderbuffer<T> {

    pub fn parameter(&self, pname: RenderbufferParameter) -> GLint {
        unsafe {
            let mut params = ::std::mem::MaybeUninit::uninit();

            if gl::GetNamedRenderbufferParameteriv::is_loaded() {
                gl::GetNamedRenderbufferParameteriv(self.raw.id(), pname as GLenum, params.as_mut_ptr());
            } else {
                let binding = TARGET.bind(&self.raw);
                gl::GetRenderbufferParameteriv(binding.target_id(), pname as GLenum, params.as_mut_ptr());
            }

            params.assume_init()
        }
    }

    pub fn storage(renderbuffer: RawRenderbuffer, width: usize, height: usize) -> Self {
        let (w,h) = (width as GLint, height as GLint);
        unsafe {
            if gl::NamedRenderbufferStorage::is_loaded() {
                gl::NamedRenderbufferStorage(renderbuffer.id(), T::glenum(), w, h);
            } else {
                let binding = TARGET.bind(&renderbuffer);
                gl::RenderbufferStorage(binding.target_id(), T::glenum(), w, h);
            }
        }
        Renderbuffer { raw: renderbuffer, fmt: PhantomData }
    }

    pub fn storage_multisample(renderbuffer: RawRenderbuffer, samples:usize, width: usize, height: usize) -> Self {
        let (s,w,h) = (samples as GLsizei, width as GLint, height as GLint);
        unsafe {
            if gl::NamedRenderbufferStorageMultisample::is_loaded() {
                gl::NamedRenderbufferStorageMultisample(renderbuffer.id(), s, T::glenum(), w, h);
            } else {
                let binding = TARGET.bind(&renderbuffer);
                gl::RenderbufferStorageMultisample(binding.target_id(), s, T::glenum(), w, h);
            }
        }
        Renderbuffer { raw: renderbuffer, fmt: PhantomData }
    }

    #[inline] pub fn width(&self) -> GLuint { self.parameter(RenderbufferParameter::Width) as GLuint }
    #[inline] pub fn height(&self) -> GLuint { self.parameter(RenderbufferParameter::Height) as GLuint }

    #[inline] pub fn samples(&self) -> GLuint { self.parameter(RenderbufferParameter::Samples) as GLuint }

    #[inline] pub fn red_size(&self) -> GLuint { self.parameter(RenderbufferParameter::RedSize) as GLuint }
    #[inline] pub fn green_size(&self) -> GLuint { self.parameter(RenderbufferParameter::GreenSize) as GLuint }
    #[inline] pub fn blue_size(&self) -> GLuint { self.parameter(RenderbufferParameter::BlueSize) as GLuint }
    #[inline] pub fn alpha_size(&self) -> GLuint { self.parameter(RenderbufferParameter::AlphaSize) as GLuint }
    #[inline] pub fn depth_size(&self) -> GLuint { self.parameter(RenderbufferParameter::DepthSize) as GLuint }
    #[inline] pub fn stencil_size(&self) -> GLuint { self.parameter(RenderbufferParameter::StencilSize) as GLuint }

}

impl<T:InternalFormat> !Send for Renderbuffer<T> {}
impl<T:InternalFormat> !Sync for Renderbuffer<T> {}
