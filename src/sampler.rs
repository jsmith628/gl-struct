use super::*;
use crate::version::*;

use std::convert::TryInto;
use std::mem::*;

glenum! {

    pub enum MagFilter {
        [Linear LINEAR "Linear"],
        [Nearest NEAREST "Nearest"]
    }

    pub enum MinFilter {
        [Linear LINEAR "Linear"],
        [Nearest NEAREST "Nearest"],
        [NearestMipmapNearest NEAREST_MIPMAP_NEAREST "Nearest Mipmap Nearest"],
        [LinearMipmapNearest LINEAR_MIPMAP_NEAREST "Linear Mipmap Nearest"],
        [NearestMipmapLinear NEAREST_MIPMAP_LINEAR "Nearest Mipmap Linear"],
        [LinearMipmapLinear LINEAR_MIPMAP_LINEAR "Linear Mipmap Linear"]
    }

    pub enum CompareMode {
        [None NONE "None"],
        [CompareRefToTexture COMPARE_REF_TO_TEXTURE "Compare Ref to Texture"]
    }

    pub enum CompareFunc {
        [Never(GL15) NEVER "Never"],
        [Always(GL15) ALWAYS "Always"],
        [Less(GL15) LESS "Less"],
        [LEqual LEQUAL "Less or Equal"],
        [Equal(GL15) EQUAL "Equal"],
        [NotEqual(GL15) NOTEQUAL "Not Equal"],
        [GEqual GEQUAL "Greater or Equal"],
        [Greater(GL15) GREATER "Greater"]
    }

    pub enum Wrapping {
        [Repeat REPEAT "Repeat"],
        [ClampToEdge(GL12) CLAMP_TO_EDGE "Clamp To Edge"],
        [ClampToBorder(GL13) CLAMP_TO_BORDER "Clamp To Border"],
        [MirroredRepeat(GL14) MIRRORED_REPEAT "Mirrored Repeat"],
        [MirrorClampToEdge(GL44) MIRROR_CLAMP_TO_EDGE "Mirror Clamp To Edge"]
    }

    impl Default for Wrapping {
        fn default() -> Self { Wrapping::Repeat }
    }

}

#[repr(transparent)]
pub struct Sampler(GLuint);

impl Sampler {

    pub fn gen(#[allow(unused_variables)] gl: &GL33) -> GLuint {
        unsafe {
            let mut s = MaybeUninit::uninit();
            gl::GenSamplers(1, s.as_mut_ptr());
            s.assume_init()
        }
    }

    pub fn gen_samplers(#[allow(unused_variables)] gl: &GL33, n: GLuint) -> Box<[GLuint]> {
        if n==0 { return Box::new([]); }
        unsafe {
            let mut s = Box::new_uninit_slice(n as usize);
            gl::GenSamplers(s.len().try_into().unwrap(), MaybeUninit::first_ptr_mut(&mut *s));
            s.assume_init()
        }
    }

    pub fn create(#[allow(unused_variables)] gl: &GL33) -> Self {
        let mut s: MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe {
            if gl::CreateSamplers::is_loaded() {
                gl::CreateSamplers(1, s.as_mut_ptr() as *mut GLuint);
            } else {
                gl::GenSamplers(1, s.as_mut_ptr() as *mut GLuint);
                gl::BindSampler(0, s.get_mut().id());
                gl::BindSampler(0, 0);
            }
            s.assume_init()
        }
    }

    pub fn create_renderbuffers(#[allow(unused_variables)] gl: &GL30, n: GLuint) -> Box<[Self]> {
        if n==0 { return Box::new([]); }
        let mut s:Box<[MaybeUninit<Self>]> = Box::new_uninit_slice(n as usize);
        unsafe {
            if gl::CreateSamplers::is_loaded() {
                gl::CreateSamplers(s.len().try_into().unwrap(), s[0].as_mut_ptr() as *mut GLuint);
            } else {
                gl::GenSamplers(s.len().try_into().unwrap(), s[0].as_mut_ptr() as *mut GLuint);
                for t in s.iter_mut() { gl::BindSampler(0, t.get_mut().id()) }
                gl::BindSampler(0, 0);
            }
            s.assume_init()
        }
    }

    pub fn id(&self) -> GLuint { self.0 }
    pub fn gl(&self) -> GL33 { unsafe { assume_supported() } }

    pub fn delete(self) { drop(self) }
    pub fn delete_samplers(s: Box<[Self]>) {
        if s.is_empty() {return;}
        unsafe {
            let ids: Box<[GLuint]> = transmute(s);
            gl::DeleteSamplers(ids.len() as GLsizei, &ids[0])
        }
    }

}

impl Drop for Sampler {
    fn drop(&mut self) { unsafe { gl::DeleteSamplers(1, &self.0) } }
}

macro_rules! sampler_params {
    ($set:ident $get:ident $enum:ident $gl:ident; $($rest:tt)*) => {
        #[inline]
        pub fn $set(&mut self, param: $enum) {
            unsafe {
                gl::SamplerParameteri(self.id(), gl::$gl, GLenum::from(param) as GLint)
            }
        }

        #[inline]
        pub fn $get(&self) -> $enum {
            unsafe {
                let mut param = ::std::mem::MaybeUninit::uninit();
                gl::GetSamplerParameteriv(self.id(), gl::$gl, param.as_mut_ptr());
                (param.assume_init() as GLenum).try_into().unwrap()
            }
        }

        sampler_params!($($rest)*);
    };

    ($set:ident $get:ident $gl:ident; $($rest:tt)*) => {
        #[inline]
        pub fn $set(&mut self, param: GLfloat) {
            unsafe {
                gl::SamplerParameterf(self.id(), gl::$gl, param)
            }
        }

        #[inline]
        pub fn $get(&self) -> GLfloat {
            unsafe {
                let mut param = ::std::mem::MaybeUninit::uninit();
                gl::GetSamplerParameterfv(self.id(), gl::$gl, param.as_mut_ptr());
                param.assume_init()
            }
        }

        sampler_params!($($rest)*);
    };

    () => {}
}

impl Sampler {

    sampler_params! {
        min_filtering get_min_filtering MinFilter TEXTURE_MIN_FILTER;
        mag_filtering get_mag_filtering MagFilter TEXTURE_MAG_FILTER;
        // max_anisotropy get_max_anisotropy TEXTURE_MAX_ANISOTROPY;
        min_lod get_min_lod TEXTURE_MIN_LOD;
        max_lod get_max_lod TEXTURE_MAX_LOD;
        lod_bias get_lod_bias TEXTURE_LOD_BIAS;
        compare_mode get_compare_mode CompareMode TEXTURE_COMPARE_MODE;
        compare_func get_compare_func CompareFunc TEXTURE_COMPARE_FUNC;
        wrap_s get_wrap_s Wrapping TEXTURE_WRAP_S;
        wrap_t get_wrap_t Wrapping TEXTURE_WRAP_T;
        wrap_r get_wrap_r Wrapping TEXTURE_WRAP_R;
    }

    #[inline]
    pub fn cube_map_seamless(&mut self, param: bool) {
        unsafe {
            gl::SamplerParameteri(self.id(), gl::TEXTURE_CUBE_MAP_SEAMLESS, param as GLint)
        }
    }

    #[inline]
    pub fn get_cube_map_seamless(&mut self) -> bool {
        unsafe {
            let mut dest = ::std::mem::MaybeUninit::uninit();
            gl::GetSamplerParameteriv(self.id(), gl::TEXTURE_CUBE_MAP_SEAMLESS, dest.as_mut_ptr());
            dest.assume_init() != 0
        }
    }


}
