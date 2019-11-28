use super::*;

use std::convert::TryInto;

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
        [Never NEVER "Never"],
        [Always ALWAYS "Always"],
        [Less LESS "Less"],
        [LEqual LEQUAL "Less or Equal"],
        [Equal EQUAL "Equal"],
        [NotEqual NOTEQUAL "Not Equal"],
        [GEqual GEQUAL "Greater or Equal"],
        [Greater GREATER "Greater"]
    }

    pub enum Wrapping {
        [Repeat REPEAT "Repeat"],
        [MirroredRepeat MIRRORED_REPEAT "Mirrored Repeat"],
        [ClampToEdge CLAMP_TO_EDGE "Clamp To Edge"],
        [ClampToBorder CLAMP_TO_BORDER "Clamp To Border"],
        [MirrorClampToEdge MIRROR_CLAMP_TO_EDGE "Mirror Clamp To Edge"]
    }

}

gl_resource!{
    pub struct Sampler {
        gl = GL33,
        target = !,
        ident = Sampler,
        gen = GenSamplers,
        // bind = BindSampler,
        is = IsSampler,
        delete = DeleteSamplers
    }
}

macro_rules! sampler_params {
    ($set:ident $get:ident $enum:ident $gl:ident; $($rest:tt)*) => {
        #[inline]
        pub fn $set(&mut self, param: $enum) {
            unsafe {
                gl::SamplerParameteri(self.id(), gl::$gl, param as GLint)
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


impl !Send for Sampler {}
impl !Sync for Sampler {}
