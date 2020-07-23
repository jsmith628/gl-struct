use super::*;
use crate::pixel::*;
use crate::context::*;

use std::marker::PhantomData;
use std::mem::*;

glenum! {

    enum RenderbufferTarget {
        [Renderbuffer RENDERBUFFER "Renderbuffer"]
    }

    #[non_exhaustive]
    enum RenderbufferParameter {
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

impl<F,MS> Target<Renderbuffer<F,MS>> for RenderbufferTarget {
    fn target_id(self) -> GLenum { self.into() }
    unsafe fn bind(self, rb: &Renderbuffer<F,MS>) { gl::BindRenderbuffer(self.into(), rb.id()) }
    unsafe fn unbind(self) { gl::BindRenderbuffer(self.into(), 0) }
}

static mut RENDERBUFFER: BindingLocation<RenderbufferTarget> = unsafe {
    BindingLocation::new(RenderbufferTarget::Renderbuffer)
};

pub struct Renderbuffer<F, MS=MS0> {
    id: GLuint,
    fmt: PhantomData<(F,MS, *const ())>
}

pub type UninitRenderbuffer = Renderbuffer<!,!>;

impl UninitRenderbuffer {

    pub fn gen(#[allow(unused_variables)] gl: &GL30) -> GLuint {
        unsafe {
            let mut rb = MaybeUninit::uninit();
            gl::GenRenderbuffers(1, rb.as_mut_ptr());
            rb.assume_init()
        }
    }

    pub fn gen_renderbuffers(#[allow(unused_variables)] gl: &GL30, n: GLuint) -> Box<[GLuint]> {
        if n==0 { return Box::new([]); }
        unsafe {
            let mut rb = Box::new_uninit_slice(n as usize);
            gl::GenRenderbuffers(rb.len().try_into().unwrap(), MaybeUninit::first_ptr_mut(&mut *rb));
            rb.assume_init()
        }
    }

    pub fn create(#[allow(unused_variables)] gl: &GL30) -> Self {
        let mut rb: MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe {
            if gl::CreateRenderbuffers::is_loaded() {
                gl::CreateRenderbuffers(1, rb.as_mut_ptr() as *mut GLuint);
            } else {
                gl::GenRenderbuffers(1, rb.as_mut_ptr() as *mut GLuint);
                gl::BindRenderbuffer(gl::RENDERBUFFER, rb.get_mut().id());
                gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
            }
            rb.assume_init()
        }
    }

    pub fn create_renderbuffers(#[allow(unused_variables)] gl: &GL30, n: GLuint) -> Box<[Self]> {
        if n==0 { return Box::new([]); }
        let mut rb:Box<[MaybeUninit<Self>]> = Box::new_uninit_slice(n as usize);
        unsafe {
            if gl::CreateRenderbuffers::is_loaded() {
                gl::CreateRenderbuffers(rb.len().try_into().unwrap(), rb[0].as_mut_ptr() as *mut GLuint);
            } else {
                gl::GenRenderbuffers(rb.len().try_into().unwrap(), rb[0].as_mut_ptr() as *mut GLuint);
                for t in rb.iter_mut() { gl::BindRenderbuffer(gl::RENDERBUFFER, t.get_mut().id()) }
                gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
            }
            rb.assume_init()
        }
    }

    #[allow(unused_variables)]
    pub fn storage<F:ReqRenderBuffer>(
        self, gl:&F::GL, width: usize, height: usize
    ) -> Renderbuffer<F> {

        //get the id
        let id = self.id;

        //allocate the storage
        let (w,h) = (width as GLint, height as GLint);
        unsafe {
            if gl::NamedRenderbufferStorage::is_loaded() {
                gl::NamedRenderbufferStorage(id, F::glenum(), w, h);
            } else {
                RENDERBUFFER.map_bind(&self,
                    |b| gl::RenderbufferStorage(b.target_id(), F::glenum(), w, h)
                );
            }
        }

        //forget self so that we don't accidentally drop the renderbuffer
        forget(self);

        //return
        Renderbuffer { id: id, fmt: PhantomData }

    }

    #[allow(unused_variables)]
    pub fn storage_multisample<F:ReqRenderBuffer, MS:RenderbufferMSFormat>(
        self, gl:&F::GL, width: usize, height: usize
    ) -> Renderbuffer<F,MS> {

        //get the id
        let id = self.id;

        //allocate the storage
        let (w,h,samples) = (width as GLint, height as GLint, MS::SAMPLES.try_into().unwrap());
        unsafe {
            if gl::NamedRenderbufferStorageMultisample::is_loaded() {
                gl::NamedRenderbufferStorageMultisample(id, samples, F::glenum(), w, h);
            } else {
                RENDERBUFFER.map_bind(&self,
                    |b| gl::RenderbufferStorageMultisample(b.target_id(), samples, F::glenum(), w, h)
                );
            }
        }

        //forget self so that we don't accidentally drop the renderbuffer
        forget(self);

        //return
        Renderbuffer { id: id, fmt: PhantomData }

    }

}

impl<F,MS> Renderbuffer<F,MS> {

    pub fn id(&self) -> GLuint { self.id }
    pub fn gl(&self) -> GL30 { unsafe {assume_supported()} }

    fn parameter(&self, pname: RenderbufferParameter) -> GLint {
        unsafe {
            let mut params = ::std::mem::MaybeUninit::uninit();

            if gl::GetNamedRenderbufferParameteriv::is_loaded() {
                gl::GetNamedRenderbufferParameteriv(self.id(), pname as GLenum, params.as_mut_ptr());
            } else {
                RENDERBUFFER.map_bind(self,
                    |b| gl::GetRenderbufferParameteriv(
                        b.target_id(), pname as GLenum, params.as_mut_ptr()
                    )
                );
            }

            params.assume_init()
        }
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

    pub fn delete(self) { drop(self); }
    pub fn delete_renderbuffers(rb: Box<[Self]>) {
        if rb.len()==0 {return;}
        unsafe {
            let ids: Box<[GLuint]> = transmute(rb);
            gl::DeleteRenderbuffers(ids.len() as GLsizei, &ids[0])
        }
    }

}

impl<F,MS> Drop for Renderbuffer<F,MS> {
    fn drop(&mut self) { unsafe { gl::DeleteRenderbuffers(1, &self.id); } }
}
