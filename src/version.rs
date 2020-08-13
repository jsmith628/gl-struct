
use super::*;

use std::mem::*;
use std::str::*;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::collections::HashSet;

unsafe fn get_integerv(name: GLenum) -> GLint {
    let mut dest = MaybeUninit::uninit();
    gl::GetIntegerv(name, dest.get_mut());
    dest.assume_init()
}

unsafe fn get_string(name: GLenum) -> &'static CStr {
    CStr::from_ptr(gl::GetString(name) as *const c_char)
}

unsafe fn get_string_i(name: GLenum, index: GLuint) -> &'static CStr {
    CStr::from_ptr(gl::GetStringi(name, index) as *const c_char)
}

fn get_major_version() -> GLuint {
    if gl::GetIntegerv::is_loaded() {
        unsafe { get_integerv(gl::MAJOR_VERSION) as GLuint }
    } else {
        0
    }
}

fn get_minor_version() -> GLuint {
    if gl::GetIntegerv::is_loaded() {
        unsafe { get_integerv(gl::MINOR_VERSION) as GLuint }
    } else {
        0
    }
}

fn get_version() -> (GLuint, GLuint) {
    (get_major_version(), get_minor_version())
}

enum ExtensionsIter {
    String(SplitWhitespace<'static>),
    Stringi(usize, usize),
}

impl Iterator for ExtensionsIter {
    type Item = &'static str;

    fn next(&mut self) -> Option<&'static str> {
        match self {
            Self::String(iter) => iter.next(),
            Self::Stringi(count, index) => {
                if index < count {
                    *index += 1;
                    unsafe {
                        Some(get_string_i(gl::EXTENSIONS, (*index-1) as GLuint).to_str().unwrap())
                    }
                } else {
                    None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::String(iter) => iter.size_hint(),
            Self::Stringi(count, index) => (count - index, Some(count - index))
        }
    }

}


fn get_extensions() -> ExtensionsIter {
    unsafe {
        if gl::GetStringi::is_loaded() {
            //for GL30 onwards, we want to use gl::GetStringi and loop through that way
            ExtensionsIter::Stringi(get_integerv(gl::NUM_EXTENSIONS).max(0) as usize, 0)
        } else if gl::GetString::is_loaded() {
            //else, we use glGetString to get a space-separated list of extensions
            ExtensionsIter::String(get_string(gl::EXTENSIONS).to_str().unwrap().split_whitespace())
        } else {
            //else, the GL isn't loaded, so we just return nothing
            ExtensionsIter::Stringi(0, 0)
        }
    }
}


#[inline]
pub unsafe fn assume_supported<GL:GLVersion>() -> GL {
    MaybeUninit::zeroed().assume_init()
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum GLVersionError {
    Version(GLuint, GLuint),
    Extension(&'static str)
}

impl ::std::fmt::Display for GLVersionError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Self::Version(maj, min) => write!(
                f,"The current OpenGL context does not support GL version {}.{}", maj, min
            ),
            Self::Extension(ex) => write!(
                f,"The current OpenGL context does not support the extension {}", ex
            ),
        }
    }
}



pub fn supported<GL:GLVersion>() -> Result<GL,GLVersionError> {

    //since the version methods take &self, we need to construct an instance
    let target: GL = unsafe { assume_supported() };

    let req_version = (target.req_major_version(), target.req_minor_version());
    let mut req_extensions = target.req_extensions();

    //if the required version is satisfied
    if req_version == (0,0) || get_version() >= req_version {

        //make sure that every extra required extension is supported
        if req_extensions.len() == 0 { return Ok(target); }
        for e in get_extensions() {
            req_extensions.remove(e);
            if req_extensions.len() == 0 { return Ok(target); }
        }

        Err(GLVersionError::Extension(req_extensions.into_iter().next().unwrap()))

    } else {
        Err(GLVersionError::Version(req_version.0, req_version.1))
    }

}

#[inline]
pub fn supports<Test:GLVersion+?Sized, Version:GLVersion+Sized>(
    #[allow(unused_variables)] gl: &Test
) -> bool {

    //use specialization and a helper trait to determine if Test supports Version
    trait Helper<GL> { fn _supports() -> bool; }
    impl<T:?Sized,U> Helper<U> for T { default fn _supports() -> bool {false} }
    impl<T:Supports<U>+?Sized,U:GLVersion> Helper<U> for T {
        fn _supports() -> bool {true}
    }

    <Test as Helper<Version>>::_supports()

}

pub fn upgrade_to<Test:GLVersion+?Sized, Version:GLVersion+Sized>(gl: &Test) -> Result<Version,GLVersionError> {
    if supports::<Test,Version>(gl) {
        Ok(unsafe { assume_supported() } )
    } else {
        supported::<Version>()
    }
}

pub fn downgrade_to<Test:Supports<Version>, Version:GLVersion>(gl: &Test) -> Version {
    unsafe { assume_supported() }
}

pub unsafe trait GLVersion {

    fn req_version(&self) -> (GLuint, GLuint);
    #[inline(always)] fn req_major_version(&self) -> GLuint { self.req_version().0 }
    #[inline(always)] fn req_minor_version(&self) -> GLuint { self.req_version().1 }
    fn req_extensions(&self) -> HashSet<&'static str>;

    fn version(&self) -> (GLuint, GLuint);
    #[inline(always)] fn major_version(&self) -> GLuint { self.version().0 }
    #[inline(always)] fn minor_version(&self) -> GLuint { self.version().1 }

    fn supports_version(&self, v: (GLuint, GLuint)) -> bool { v <= self.version() }
    fn supports_extension(&self, ex: &str) -> bool;

    #[inline(always)] fn as_gl10(&self) -> GL10 {GL10 {_private:PhantomData}}

    #[inline(always)] fn try_as_gl11(&self) -> Result<GL11,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl12(&self) -> Result<GL12,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl13(&self) -> Result<GL13,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl14(&self) -> Result<GL14,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl15(&self) -> Result<GL15,GLVersionError> {upgrade_to(self)}

    #[inline(always)] fn try_as_gl20(&self) -> Result<GL20,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl21(&self) -> Result<GL21,GLVersionError> {upgrade_to(self)}

    #[inline(always)] fn try_as_gl30(&self) -> Result<GL30,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl31(&self) -> Result<GL31,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl32(&self) -> Result<GL32,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl33(&self) -> Result<GL33,GLVersionError> {upgrade_to(self)}

    #[inline(always)] fn try_as_gl40(&self) -> Result<GL40,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl41(&self) -> Result<GL41,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl42(&self) -> Result<GL42,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl43(&self) -> Result<GL43,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl44(&self) -> Result<GL44,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl45(&self) -> Result<GL45,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl46(&self) -> Result<GL46,GLVersionError> {upgrade_to(self)}

}

///Signifies that a given [GLVersion] object is a superset of another
#[marker] pub unsafe trait Supports<V:GLVersion>: GLVersion {}
unsafe impl<G:GLVersion> Supports<G> for G {}

///Signifies that a given [GLVersion] object supports all versions before [2.1](GL21)
pub unsafe trait GL2:
    Supports<GL10> + Supports<GL11> + Supports<GL12> + Supports<GL13> + Supports<GL14> +
    Supports<GL15> + Supports<GL20>
{
    #[inline(always)] fn as_gl11(&self) -> GL11 {GL11 {_private:PhantomData}}
    #[inline(always)] fn as_gl12(&self) -> GL12 {GL12 {_private:PhantomData}}
    #[inline(always)] fn as_gl13(&self) -> GL13 {GL13 {_private:PhantomData}}
    #[inline(always)] fn as_gl14(&self) -> GL14 {GL14 {_private:PhantomData}}
    #[inline(always)] fn as_gl15(&self) -> GL15 {GL15 {_private:PhantomData}}
}

unsafe impl<V> GL2 for V where V:
    Supports<GL10> + Supports<GL11> + Supports<GL12> + Supports<GL13> + Supports<GL14> +
    Supports<GL15> + Supports<GL20> {}

///Signifies that a given [GLVersion] object supports all versions before [3.1](GL31)
pub unsafe trait GL3: GL2 + Supports<GL21> + Supports<GL30> {
    #[inline(always)] fn as_gl20(&self) -> GL20 {GL20 {_private:PhantomData}}
    #[inline(always)] fn as_gl21(&self) -> GL21 {GL21 {_private:PhantomData}}
}

unsafe impl<V> GL3 for V where V: GL2 + Supports<GL21> + Supports<GL30> {}

///Signifies that a given [GLVersion] object supports all versions before [4.1](GL41)
pub unsafe trait GL4: GL3 + Supports<GL31> + Supports<GL32> + Supports<GL33> + Supports<GL40> {
    #[inline(always)] fn as_gl30(&self) -> GL30 {GL30 {_private:PhantomData}}
    #[inline(always)] fn as_gl31(&self) -> GL31 {GL31 {_private:PhantomData}}
    #[inline(always)] fn as_gl32(&self) -> GL32 {GL32 {_private:PhantomData}}
    #[inline(always)] fn as_gl33(&self) -> GL33 {GL33 {_private:PhantomData}}
}

unsafe impl<V> GL4 for V where V: GL3 + Supports<GL31> + Supports<GL32> + Supports<GL33> + Supports<GL40> {}

macro_rules! version_struct {

    ({$($prev:ty),*} $gl:ident $maj:literal $min:literal, $($rest:tt)*) => {
        version_struct!({$($prev)*} $gl $maj $min {}, $($rest)*);
    };

    (
        {$($prev:ty),*}
        $gl:ident $maj:literal $min:literal
        {$($ex:ident {$($ex_deps:ident),*}),*},
        $($rest:tt)*
    ) => {

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct $gl { _private: ::std::marker::PhantomData<*const ()> }

        unsafe impl GLVersion for $gl {
            fn req_version(&self) -> (GLuint, GLuint) {($maj, $min)}
            fn req_extensions(&self) -> HashSet<&'static str> { HashSet::new() }

            fn version(&self) -> (GLuint, GLuint) {($maj, $min)}
            fn supports_extension(&self, ex: &str) -> bool {
                //check all supported extensions for a match
                $(if downgrade_to::<_,$ex>(self).supports_extension(ex) {return true;})*

                //check all previous versions for a match
                $(if downgrade_to::<_,$prev>(self).supports_extension(ex) {return true;})*

                return false;
            }

        }

        $(unsafe impl<G:GLVersion> Supports<G> for $gl where $prev:Supports<G> {})*
        $(unsafe impl<G:GLVersion> Supports<G> for $gl where $ex:Supports<G> {})*

        extension_struct!($($ex {$($ex_deps),*},)*);

        version_struct!({$gl} $($rest)*);
    };

    ({$($prev:ident)*} ) => {}
}

macro_rules! extension_struct {

    ($ex:ident {$($deps:ident),*}, $($rest:tt)*) => {

        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct $ex { _private: ::std::marker::PhantomData<*const ()> }

        unsafe impl GLVersion for $ex {
            fn req_version(&self) -> (GLuint, GLuint) {(0, 0)}
            fn req_extensions(&self) -> HashSet<&'static str> {
                let mut ex = HashSet::new();
                ex.insert(stringify!($ex));
                ex
            }

            fn version(&self) -> (GLuint, GLuint) {
                //gets the max supported version from this extensions dependencies
                downgrade_to::<_,($($deps,)*)>(self).version()
            }

            fn supports_extension(&self, ex: &str) -> bool {
                //if the string matches this type's name, we have a hit
                if ex == stringify!($ex) { return true; }

                //else, check all of the dependencies
                $(if downgrade_to::<_,$deps>(self).supports_extension(ex) {return true;})*

                return false;
            }

        }

        $(unsafe impl<G:GLVersion> Supports<G> for $ex where $deps:Supports<G> {})*
        extension_struct!($($rest)*);
    };

    () => {}
}

//NOTE: contrary to expectation, when ever ambiguous, the dependencies listed here
//are **under**-estimated, as the purpose of them is guarrantee support if a dependent
//extension is supported and NOT to actually _check_ for support; glGetString() or glGetStringi()
//should always properly do that for us

//NOTE: the following information was taken from the Khronos registry at: https://www.khronos.org/registry/OpenGL/index_gl.php
//The determination of what extensions each core version supports was determined by looking at the
//version history section of each core spec, whereas the extension dependecies were taken from
//the 'dependecies' section of each extension spec where effort was taken to mirror notes from the
//specs in comments whenever potentially relevant

version_struct!{ {()}

    GL10 1 0,

    GL11 1 1 {
        GL_EXT_vertex_array {GL10},
        GL_EXT_polygon_offset {GL10},
        GL_EXT_blend_logic_op {GL10},
        GL_EXT_texture {GL10},
        GL_EXT_copy_texture {GL_EXT_texture},
        GL_EXT_subtexture {GL_EXT_texture},
        GL_EXT_texture_object {GL10}
    },

    GL12 1 2 {
        GL_EXT_texture3D {GL_EXT_texture},
        GL_EXT_bgra {GL10},
        GL_EXT_packed_pixels {GL10},
        GL_EXT_rescale_normal {GL10},
        GL_EXT_separate_specular_color {GL10},
        GL_SGIS_texture_edge_clamp {GL10},
        GL_SGIS_texture_lod {GL_EXT_texture},
        GL_EXT_draw_range_elements {GL10},
        GL_EXT_color_table {GL10}, //no registry info on this, basing off of GL_SGIS_color_table instead
        GL_EXT_color_subtable {GL10},
        GL_EXT_convolution {GL_EXT_texture},
        GL_HP_convolution_border_modes {GL_EXT_convolution},
        GL_SGI_color_matrix {GL10},
        GL_EXT_histogram {GL_EXT_texture},
        GL_EXT_blend_color {GL10},
        GL_EXT_blend_minmax {GL10},
        GL_EXT_blend_subtract {GL10}
    },

    GL13 1 3 {
        GL_ARB_texture_compression {GL11, GL_ARB_texture_cube_map},
        GL_ARB_texture_cube_map {GL10},
        GL_ARB_multisample {WGL_EXT_extensions_string, WGL_EXT_pixel_format},
        GL_ARB_multitexture {GL10}, //the docs for this extension leave out all info on dependencies, so using GL10 as dep to be safe
        GL_ARB_texture_env_add {GL10},
        GL_ARB_texture_env_combine {GL11, GL_ARB_multitexture},
        GL_ARB_texture_env_dot3 {GL11, GL_ARB_multitexture, GL_ARB_texture_env_combine},
        GL_ARB_texture_border_clamp {GL10},
        GL_ARB_transpose_matrix {GL10},

        //brought in by GL_ARB_multisample:
        WGL_EXT_extensions_string {GL10},
        WGL_EXT_pixel_format {WGL_EXT_extensions_string}

    },

    GL14 1 4 {
        GL_SGIS_generate_mipmap {GL_EXT_texture},
        GL_NV_blend_square {GL10},
        GL_ARB_depth_texture {GL11},
        GL_ARB_shadow {GL11, GL_ARB_depth_texture},
        GL_EXT_fog_coord {GL11},
        GL_EXT_multi_draw_arrays {GL11},
        GL_ARB_point_parameters {GL10},
        GL_EXT_secondary_color {GL_EXT_separate_specular_color},
        GL_EXT_blend_func_separate {GL10},
        GL_EXT_stencil_wrap {GL10},
        GL_ARB_texture_env_crossbar {GL11, GL_ARB_multitexture, GL_ARB_texture_env_combine},
        GL_EXT_texture_lod_bias {GL10},
        GL_ARB_texture_mirrored_repeat {GL10},
        GL_ARB_window_pos {GL10}
    },

    GL15 1 5 {
        GL_ARB_vertex_buffer_object {GL10},
        GL_ARB_occlusion_query {GL10},
        GL_EXT_shadow_funcs {GL11, GL_ARB_depth_texture, GL_ARB_shadow}
    },

    GL20 2 0 {
        GL_ARB_shader_objects {GL10},
        GL_ARB_vertex_shader {GL_ARB_shader_objects},
        GL_ARB_fragment_shader {GL_ARB_shader_objects},
        GL_ARB_shading_language_100 {GL_ARB_shader_objects, GL_ARB_fragment_shader, GL_ARB_vertex_shader},
        GL_ARB_draw_buffers {GL13},
        GL_ARB_texture_non_power_of_two {GL10},
        GL_ARB_point_sprite {GL10},

        //technically, this required GL_EXT_blend_subtract or GL_EXT_blend_minmax, but we don't
        //know which, so we keep the deps at GL10 to be safe
        GL_EXT_blend_equation_separate {GL10},

        GL_ATI_separate_stencil {GL10},
        GL_EXT_stencil_two_side {GL10}
    },

    GL21 2 1 {
        GL_ARB_pixel_buffer_object {GL_ARB_vertex_buffer_object},
        GL_EXT_texture_sRGB {GL11}
    },

    GL30 3 0 {
        //whenever choosing between this or NV_gpu_shader4, we choose EXT_gpu_shader4
        //since they _should_ basically be synonyms
        GL_EXT_gpu_shader4 {GL20},

        GL_NV_conditional_render {GL_ARB_occlusion_query}, //for ES, we need EXT_occlusion_query_boolean
        GL_APPLE_flush_buffer_range {GL_ARB_vertex_buffer_object},
        GL_ARB_color_buffer_float {WGL_ARB_pixel_format}, //"will work with OpenGL 1.5 Specification"
        GL_NV_depth_buffer_float {
            GL20, GL_ARB_color_buffer_float, GL_EXT_packed_depth_stencil, GL_EXT_framebuffer_object
        },
        GL_ARB_texture_float {GL_EXT_texture}, //"will work with OpenGL 1.5 Specification"
        GL_EXT_packed_float {GL11, WGL_ARB_pixel_format}, //"WGL_ARB_pixel_format is required for use with WGL"
        GL_EXT_texture_shared_exponent {GL11},
        GL_EXT_framebuffer_object {GL11},
        GL_NV_half_float {GL11},
        GL_ARB_half_float_pixel {GL10}, //"will work with OpenGL 1.5 Specification"
        GL_EXT_framebuffer_multisample {GL_EXT_framebuffer_object, GL_EXT_framebuffer_blit},
        GL_EXT_framebuffer_blit {GL11, GL_EXT_framebuffer_object},
        GL_EXT_texture_integer {GL20, GL_EXT_gpu_shader4},
        GL_EXT_texture_array {GL10}, //written against 2.0 though
        GL_EXT_packed_depth_stencil {GL11, GL_EXT_framebuffer_object},
        GL_EXT_draw_buffers2 {GL20},
        GL_EXT_texture_compression_rgtc {GL_ARB_texture_compression},
        GL_EXT_transform_feedback {GL_ARB_shader_objects},
        GL_APPLE_vertex_array_object {GL10},
        GL_EXT_framebuffer_sRGB {GL11, WGL_EXT_extensions_string, WGL_EXT_pixel_format},

        //brought in by GL_EXT_packed_float and GL_ARB_color_buffer_float
        WGL_ARB_extensions_string {GL10},
        WGL_ARB_pixel_format {WGL_ARB_extensions_string},

        //Technically, the spec does NOT say that this extension has been incorperated into GL30,
        //as GL_ARB_framebuffer_object was created after OpenGL version 3.0. However, the
        //extension is designed _specfically_ as an encapsulation of the GL30 framebuffer
        //functionality, and thus, and context supporting GL30 should also support all of the
        //features of this extension
        GL_ARB_framebuffer_object {

            GL11,

            //Technically, the spec for this extension does not say that GL_ARB_framebuffer_object
            //_requires_ these extensions, but since it GL_ARB_framebuffer_object is _specfically_
            //designed to merge together these extensions, is very clear that any context supporting
            //GL_ARB_framebuffer_object supports these as well
            GL_EXT_framebuffer_object,
            GL_EXT_framebuffer_blit,
            GL_EXT_framebuffer_multisample,
            GL_EXT_packed_depth_stencil

        }

    },

    GL31 3 1 {
        GL_ARB_draw_instanced {GL20, GL_EXT_gpu_shader4},
        GL_ARB_copy_buffer {GL10},
        GL_NV_primitive_restart {GL10},
        GL_ARB_texture_buffer_object {GL20, GL_EXT_gpu_shader4},
        GL_ARB_texture_rectangle {GL11},
        GL_ARB_uniform_buffer_object {
            GL_ARB_shading_language_100, GL_ARB_shader_objects, GL_ARB_vertex_buffer_object
        }
    },

    GL32 3 2 {
        GL_ARB_vertex_array_bgra {GL11},
        GL_ARB_draw_elements_base_vertex {GL10},
        GL_ARB_fragment_coord_conventions {GL10},
        GL_ARB_provoking_vertex {GL10},
        GL_ARB_seamless_cube_map {GL11, GL_ARB_texture_cube_map},

        //does not say GL_EXT_texture_object is needed, so I guess we're just going to have to
        //add that as a dep for TEXTURE_2D_MULTISAMPLE
        GL_ARB_texture_multisample {GL10},

        GL_ARB_depth_clamp {GL10},
        GL_ARB_geometry_shader4 {GL11},
        GL_ARB_sync {GL31}
    },

    GL33 3 3 {
        GL_ARB_shader_bit_encoding {GL_ARB_shading_language_100}, //GLSL only
        GL_ARB_blend_func_extended {GL10, GL_ARB_fragment_shader, GL_EXT_gpu_shader4}, //doesn't say NV_gpu_shader4 can be used
        GL_ARB_explicit_attrib_location {GL_ARB_vertex_shader},
        GL_ARB_occlusion_query2 {GL10}, //"OpenGL 1.x is required."
        GL_ARB_sampler_objects {GL10},
        GL_ARB_texture_rgb10_a2ui {GL_EXT_texture_integer},
        GL_ARB_texture_swizzle {GL10}, //probably implies EXT_textures but it doesn't say
        GL_ARB_timer_query {GL10},
        GL_ARB_instanced_arrays {GL11},
        GL_ARB_vertex_type_2_10_10_10_rev {GL11}
    },

    GL40 4 0 {
        GL_ARB_texture_query_lod {GL30, GL_EXT_gpu_shader4, GL_EXT_texture_array}, //also requires GLSL 130
        GL_ARB_draw_buffers_blend {GL20, GL_EXT_draw_buffers2},
        GL_ARB_draw_indirect {GL31},
        GL_ARB_gpu_shader5 {GL32}, //also requires GLSL 150
        GL_ARB_gpu_shader_fp64 {GL32}, //also requires GLSL 150
        GL_ARB_sample_shading {GL20}, //also requires GLSL 130
        GL_ARB_shader_subroutine {GL_ARB_gpu_shader5},
        GL_ARB_tessellation_shader {GL32}, //also requires GLSL 150
        GL_ARB_texture_buffer_object_rgb32 {GL10},
        GL_ARB_texture_cube_map_array {GL10},
        GL_ARB_texture_gather {GL11}, //also requires GLSL 130
        GL_ARB_transform_feedback2 {
            GL_ARB_shading_language_100, GL_ARB_shader_objects,GL_EXT_transform_feedback //or NV_transform_feedback
        },
        GL_ARB_transform_feedback3 {GL20, GL_EXT_transform_feedback /*or NV_transform_feedback*/}
    },

    GL41 4 1 {
        GL_ARB_ES2_compatibility {GL10}, //later, we can (probably) add GLES20 here since this adds compatibility
        GL_ARB_get_program_binary {GL30},
        GL_ARB_separate_shader_objects {GL_ARB_shader_objects, GL_ARB_explicit_attrib_location},
        GL_ARB_shader_precision {GL40},
        GL_ARB_vertex_attrib_64bit {GL30, GL_ARB_gpu_shader_fp64}, //also requires GLSL 130
        GL_ARB_viewport_array {GL10, GL_ARB_geometry_shader4}
    },

    GL42 4 2 {
        GL_ARB_texture_compression_bptc {GL_ARB_texture_compression},
        GL_ARB_compressed_texture_pixel_storage {GL12},
        GL_ARB_shader_atomic_counters {GL30},
        GL_ARB_texture_storage {GL10}, //requires GL12, GLES10 or GLES20, but we can't know which
        GL_ARB_transform_feedback_instanced {GL_ARB_transform_feedback2, GL_ARB_draw_instanced},
        GL_ARB_base_instance {GL_ARB_draw_instanced},
        GL_ARB_shader_image_load_store {GL30}, //also requires GLSL 130
        GL_ARB_conservative_depth {GL30},
        GL_ARB_shading_language_420pack {GL10}, //also requires GLSL 130
        GL_ARB_internalformat_query {GL_ARB_framebuffer_object},
        GL_ARB_map_buffer_alignment {GL21}

    },

    GL43 4 3 {
        GL_ARB_arrays_of_arrays {GL10}, //GLSL only; requires GLSL120 also

        //eventually can (probably) add GLES30 to this
        GL_ARB_ES3_compatibility {
            GL33, GL_ARB_ES2_compatibility, GL_ARB_invalidate_subdata, GL_ARB_texture_storage
        },

        GL_ARB_clear_buffer_object {GL15 /*GL_EXT_direct_state_access*/},
        GL_ARB_compute_shader {GL42},
        GL_ARB_copy_image {GL11},
        GL_ARB_debug_output {GL11},
        GL_ARB_explicit_uniform_location {GL_ARB_explicit_attrib_location},
        GL_ARB_fragment_layer_viewport {GL30, GL_ARB_geometry_shader4, GL_ARB_viewport_array}, //GLSL only
        GL_ARB_framebuffer_no_attachments {GL_ARB_framebuffer_object}, //or GL30
        GL_ARB_internalformat_query2 {GL20, GL_ARB_internalformat_query},
        GL_ARB_invalidate_subdata {GL20},
        GL_ARB_multi_draw_indirect {GL_ARB_draw_indirect},
        GL_ARB_program_interface_query {GL20},
        GL_ARB_robust_buffer_access_behavior {GL_ARB_robustness},
        GL_ARB_shader_image_size {GL42}, //GLSL only; also requires GLSL420
        GL_ARB_shader_storage_buffer_object {GL40, GL_ARB_program_interface_query},
        GL_ARB_stencil_texturing {GL11, GL_ARB_depth_texture, GL_EXT_packed_depth_stencil},
        GL_ARB_texture_buffer_range {GL_ARB_texture_buffer_object /*GL_EXT_direct_state_access*/},
        GL_ARB_texture_query_levels {GL30}, //also requires GLSL130
        GL_ARB_texture_storage_multisample {GL_ARB_texture_storage},
        GL_ARB_texture_view {GL_ARB_texture_storage},
        GL_ARB_vertex_attrib_binding {GL10},
        GL_KHR_debug {GL11},

        //brought in by GL_ARB_robust_buffer_access_behavior
        GL_ARB_robustness {GL11} //there may be some extra requirements for different context creators

    },

    GL44 4 4 {
        GL_ARB_buffer_storage {GL10},
        GL_ARB_clear_texture {GL13},
        GL_ARB_enhanced_layouts {GL31}, //GLSL only; also requires GLSL140
        GL_ARB_multi_bind {GL30},
        GL_ARB_query_buffer_object {GL15},
        GL_ARB_texture_mirror_clamp_to_edge {GL14},
        GL_ARB_texture_stencil8 {GL10},
        GL_ARB_vertex_type_10f_11f_11f_rev {GL30, GL_ARB_vertex_attrib_binding, GL_ARB_vertex_type_2_10_10_10_rev}
    },

    GL45 4 5 {
        GL_ARB_clip_control {GL10},
        GL_ARB_cull_distance {GL30},
        GL_ARB_ES3_1_compatibility {GL44, GL_ARB_ES2_compatibility, GL_ARB_ES3_compatibility}, //could (probably) add GLES31 later
        GL_ARB_conditional_render_inverted {GL30},
        GL_KHR_context_flush_control {GL10}, //for certain context creators, other extensions are required
        GL_ARB_derivative_control {GL40}, //GLSL only; also requires GLSL 400
        GL_ARB_direct_state_access {GL20},
        GL_ARB_get_texture_sub_image {GL20},
        GL_KHR_robustness {GL32}, //or GL20ES
        GL_ARB_shader_texture_image_samples {GL_ARB_texture_multisample}, //or GLSL150
        GL_ARB_texture_barrier {GL10}
    },

    GL46 4 6 {
        GL_ARB_indirect_parameters {GL42},
        GL_ARB_pipeline_statistics_query {GL30},
        GL_ARB_polygon_offset_clamp {GL33},
        GL_KHR_no_error {GL20},
        GL_ARB_shader_atomic_counter_ops {GL_ARB_shader_atomic_counters}, //GLSL
        GL_ARB_shader_draw_parameters {GL31},
        GL_ARB_shader_group_vote {GL_ARB_compute_shader}, //GLSL
        GL_ARB_gl_spirv {GL33},
        GL_ARB_spirv_extensions {GL_ARB_gl_spirv},
        GL_ARB_texture_filter_anisotropic {GL12},
        GL_ARB_transform_feedback_overflow_query {GL30}
    },
}


macro_rules! impl_tuple_versions {

    ({$($T:ident:$t:ident)*} $Last:ident:$l:ident) => {

        unsafe impl<$($T:GLVersion,)* $Last:GLVersion+?Sized> GLVersion for ($($T,)* $Last,) {

            fn req_version(&self) -> (GLuint, GLuint) {
                //split into the separate vars
                let ($($t,)* $l,) = self;

                //find the maximum version
                $l.req_version()$(.max($t.req_version()))*
            }

            fn req_extensions(&self) -> HashSet<&'static str> {
                //split into the separate vars
                let ($($t,)* $l, ) = self;

                //merge all of the extension sets into one
                #[allow(unused_mut)]
                let mut ex = $l.req_extensions();
                $(ex = ex.union(&$t.req_extensions()).copied().collect();)*
                ex
            }

            fn version(&self) -> (GLuint, GLuint) {
                //split into the separate vars
                let ($($t,)* $l,) = self;

                //find the maximum version
                $l.version()$(.max($t.version()))*
            }

            fn supports_extension(&self, ex: &str) -> bool {
                //split into the separate vars
                let ($($t,)* $l,) = self;

                //NOTE: theoretically, one could conceive of a situation involving this code
                //whereby a particularly maverick developer concocts a conundrum creating an
                //infinite recursive loop. However, this situation can only occur if said vile
                //programmer disregarded the rust reference rules since it would require a self
                //reference and thus unsafe code.

                //check all the subversions for a match
                $(if $t.supports_extension(ex) {return true;})*
                if $l.supports_extension(ex) {return true;}

                return false;
            }

        }

        //
        //a tuple version supports another version if any of its members do
        //

        //implement Supports if the last member does
        unsafe impl<GL:GLVersion, $($T:GLVersion,)* $Last:GLVersion+?Sized> Supports<GL> for ($($T,)* $Last,)
        where $Last:Supports<GL> {}

        //recurse the Support implementation on the tuple containing the rest of the members
        unsafe impl<GL:GLVersion, $($T:GLVersion,)* $Last:GLVersion+?Sized> Supports<GL> for ($($T,)* $Last,)
        where ($($T,)*):Supports<GL> {}

        //
        //A version supports a tuple version if it supports all of its members
        //

        unsafe impl<GL:GLVersion, $($T:GLVersion,)* $Last:GLVersion> Supports<($($T,)* $Last,)> for GL
        where $(GL:Supports<$T>,)* GL:Supports<$Last> {}



    }

}

impl_tuple!(impl_tuple_versions @with_last);


//represents a context state where NO OpenGL is supported
unsafe impl GLVersion for () {
    fn req_version(&self) -> (GLuint, GLuint) {(0,0)}
    fn req_extensions(&self) -> HashSet<&'static str> { HashSet::new() }
    fn version(&self) -> (GLuint, GLuint) {(0,0)}
    fn supports_extension(&self, ex: &str) -> bool {false}
}

//Everything supports ()
unsafe impl<GL:GLVersion> Supports<()> for GL {}

//represents the hypothetical maximum OpenGL version that supports ALL versions and extensions
unsafe impl GLVersion for ! {
    fn req_version(&self) -> (GLuint, GLuint) {(!0, !0)}
    fn req_extensions(&self) -> HashSet<&'static str> { HashSet::new() }
    fn version(&self) -> (GLuint, GLuint) {(!0, !0)}
    fn supports_extension(&self, ex: &str) -> bool {true}
}

//`!` supports everything
unsafe impl<GL:GLVersion> Supports<GL> for ! {}

//TODO: add actual checking of if functions are loaded

impl GL10 {

    pub unsafe fn assume_loaded() -> GL10 { GL10 {_private:PhantomData}}

}
