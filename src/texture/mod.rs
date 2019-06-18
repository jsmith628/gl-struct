use super::*;
use crate::gl;
use crate::Target;

use image_format::*;
use buffer_new::{UninitBuf, BufferTarget};

use std::convert::TryInto;
use std::marker::PhantomData;
use std::ops::{Bound,RangeBounds};
use std::collections::HashMap;
use std::mem::{size_of, uninitialized};

pub use self::pixel_format::*;
pub use self::pixel_data::*;
pub use self::image::*;
pub use self::dim::*;

pub mod pixel_format;
pub mod pixel_data;
mod image;
mod dim;

glenum! {
    pub enum TextureTarget {
        [Texture1D TEXTURE_1D "Texture 1D"],
        [Texture2D TEXTURE_2D "Texture 2D"],
        [Texture3D TEXTURE_3D "Texture 3D"],
        [Texture1DArray TEXTURE_1D_ARRAY "Texture 1D Array"],
        [Texture2DArray TEXTURE_2D_ARRAY "Texture 2D Array"],
        [TextureRectangle TEXTURE_RECTANGLE "Texture Rectangle"],
        [TextureBuffer TEXTURE_BUFFER "Texture Buffer"],
        [TextureCubemap TEXTURE_CUBE_MAP "Texture Cube Map"],
        [TextureCubemapArray TEXTURE_CUBE_MAP_ARRAY "Texture Cube Map Array"],
        [Texture2DMultisample TEXTURE_2D_MULTISAMPLE "Texture 2D Multisample"],
        [Texture2DMultisampleArray TEXTURE_2D_MULTISAMPLE_ARRAY "Texture 2D Multisample Array"]
    }

    pub enum TextureSwizzle {
        [Red RED "Red Component"],
        [Green GREEN "Green Component"],
        [Blue BLUE "Blue Component"],
        [Alpha ALPHA "Alpha Component"],
        [Zero ZERO "Zero"],
        [One ONE "One"]
    }

}

impl TextureTarget {
    #[inline]
    fn multisample(self) -> bool {
        match self {
            TextureTarget::Texture2DMultisample | TextureTarget::Texture2DMultisampleArray => true,
            _ => false
        }
    }
}

gl_resource!{
    pub struct RawTex {
        gl = GL2,
        target = TextureTarget,
        gen = GenTextures,
        bind = BindTexture,
        is = IsTexture,
        delete = DeleteTextures
    }
}

unsafe fn tex_storage<T:Texture>(
    _gl:&GL4, tex: &mut RawTex, levels: GLuint, dim: T::Dim, sampling: Option<(GLsizei,bool)>
) where T::InternalFormat: SizedInternalFormat {
    let mut target = T::TARGET.as_loc();
    let binding = target.bind(tex);
    let fmt = T::InternalFormat::glenum();
    let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);

    match sampling {
        Some((samples, fixed)) => match T::Dim::dim() {
            2 => gl::TexStorage2DMultisample(binding.target_id(), samples, fmt, w, h, fixed as GLboolean),
            3 => gl::TexStorage3DMultisample(binding.target_id(), samples, fmt, w, h, d, fixed as GLboolean),
            _ => panic!("{}D Multisample textures not currently supported", T::Dim::dim())
        },
        None => match T::Dim::dim() {
            1 => gl::TexStorage1D(binding.target_id(), levels as GLsizei, fmt, w),
            2 => gl::TexStorage2D(binding.target_id(), levels as GLsizei, fmt, w, h),
            3 => gl::TexStorage3D(binding.target_id(), levels as GLsizei, fmt, w, h, d),
            _ => panic!("{}D Textures not currently supported", T::Dim::dim())
        }
    }
}

unsafe fn tex_image_data<T:Texture, P:PixelData<T::PixelFormat>+?Sized>(
    tex: GLuint, level: GLuint, dim: T::Dim, data:&P
) {
    let (fmt,ty) = data.format_type().format_type();
    let mut pixel_buf = BufferTarget::PixelUnpackBuffer.as_loc();
    let _buf = data.bind_pixel_buffer(&mut pixel_buf);
    tex_image::<T>(tex, level, dim, fmt.into(), ty.into(), data.pixels());
    drop(_buf)
}

#[inline]
unsafe fn tex_image_null<T:Texture>(tex: GLuint, level: GLuint, dim: T::Dim) {
    tex_image::<T>(tex, level, dim, 0, 0, ::std::ptr::null());
}

unsafe fn tex_image<T:Texture>(
    tex: GLuint, level: GLuint, dim: T::Dim, fmt:GLenum, ty:GLenum, data:*const GLvoid
) {
    //bind the texture
    let mut target = T::TARGET.as_loc();
    let binding = target.bind_unchecked(tex);

    //convert and rename params
    let int_fmt = T::InternalFormat::glenum() as GLint;
    let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);

    //now, select the right function based on the dimensionality of the texture
    match T::Dim::dim() {
        1 => gl::TexImage1D(binding.target_id(), level as GLint, int_fmt, w, 0, fmt, ty, data),
        2 => gl::TexImage2D(binding.target_id(), level as GLint, int_fmt, w, h, 0, fmt, ty, data),
        3 => gl::TexImage3D(binding.target_id(), level as GLint, int_fmt, w, h, d, 0, fmt, ty, data),
        _ => panic!("{}D Textures not currently supported", T::Dim::dim())
    }
}

unsafe fn tex_image_multisample<T:Texture>(tex: &mut RawTex, dim: T::Dim, samples: GLuint, fixed: bool) {
    let mut target = T::TARGET.as_loc();
    let binding = target.bind(tex);
    let int_fmt = T::InternalFormat::glenum();
    let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);

    match T::Dim::dim() {
        2 => gl::TexImage2DMultisample(binding.target_id(), samples as GLsizei, int_fmt, w, h, fixed as GLboolean),
        3 => gl::TexImage3DMultisample(binding.target_id(), samples as GLsizei, int_fmt, w, h, d, fixed as GLboolean),
        _ => panic!("{}D Multisample Textures not currently supported", T::Dim::dim())
    }
}

unsafe fn tex_parameter_iv<T:Texture>(tex:&mut T, pname:GLenum, params: *const GLint) {
    if gl::TextureParameteriv::is_loaded() {
        gl::TextureParameteriv(tex.id(), pname, params);
    } else {
        let mut target = T::TARGET.as_loc();
        let binding = target.bind_unchecked(tex.id());
        gl::TexParameteriv(binding.target_id(), pname, params);
    }
}

unsafe fn get_tex_parameter_iv<T:Texture>(tex:&T, pname:GLenum) -> GLint {
    let mut param = ::std::mem::uninitialized();
    if gl::GetTextureParameteriv::is_loaded() {
        gl::GetTextureParameteriv(tex.id(), pname, &mut param as *mut GLint);
    } else {
        let mut target = T::TARGET.as_loc();
        let binding = target.bind_unchecked(tex.id());
        gl::GetTexParameteriv(binding.target_id(), pname, &mut param as *mut GLint);
    }
    param
}

#[inline]
unsafe fn get_swizzle_param<T:Texture>(tex:&T, pname:GLenum) -> TextureSwizzle {
    (get_tex_parameter_iv(tex, pname) as GLenum).try_into().unwrap()
}

#[inline]
unsafe fn swizzle_param<T:Texture>(tex:&mut T, pname:GLenum, param:TextureSwizzle) {
    tex_parameter_iv(tex, pname, &mut (param as GLint) as *mut GLint)
}

macro_rules! if_sized {
    ($name:ident($($gen:tt)*)($($param:ident: $ty:ty),*) -> $ret:ty {$($c1:tt)*} {$($c2:tt)*} where $($rest:tt)* ) => {
        trait Helper<T:Texture>: InternalFormat {
            fn $name<$($gen)*>($($param: $ty),*) -> $ret where $($rest)*;
        }

        impl<F,T> Helper<T> for F
        where F:InternalFormat, T:Texture<InternalFormat=F,PixelFormat=F::FormatType>
        {
            #[inline] default fn $name<$($gen)*>($($param: $ty),*) -> $ret where $($rest)* {$($c1)*}
        }

        impl<F,T> Helper<T> for F
        where F:SizedInternalFormat, T:Texture<InternalFormat=F,PixelFormat=F::FormatType>
        {
            #[inline] fn $name<$($gen)*>($($param: $ty),*) -> $ret where $($rest)* {$($c2)*}
        }
    }
}

pub unsafe trait Texture: Sized {
    type InternalFormat: InternalFormat<FormatType=Self::PixelFormat>;
    type PixelFormat: PixelFormatType;
    type Dim: TexDim;

    const TARGET: TextureTarget;

    fn id(&self) -> GLuint;
    fn dim(&self) -> Self::Dim;

    unsafe fn from_raw(raw:RawTex, dim:Self::Dim) -> Self;

    #[inline]
    fn immutable_storage(&self) -> bool {
        unsafe { get_tex_parameter_iv(self, gl::TEXTURE_IMMUTABLE_FORMAT) != 0 }
    }

    #[inline]
    unsafe fn alloc(gl:&GL2, dim:Self::Dim) -> Self {
        let raw = RawTex::gen(gl);
        if let Ok(gl4) = gl.upgrade().and_then(|gl3| gl3.upgrade()) {
            if_sized!(
                helper()(_gl:&GL4,tex:RawTex,d:T::Dim) -> T
                    {unsafe {T::image(tex, d)}}
                    {unsafe{T::storage(_gl, tex, d)}}
                where
            );
            Self::InternalFormat::helper(&gl4, raw, dim)
        } else {
            Self::image(raw, dim)
        }
    }

    #[inline]
    unsafe fn image(mut raw:RawTex, dim:Self::Dim) -> Self {
        if Self::TARGET.multisample() {
            tex_image_multisample::<Self>(&mut raw, dim, 0, false)
        } else {
            tex_image_null::<Self>(raw.id(), 0, dim)
        }
        Self::from_raw(raw, dim)
    }

    #[inline]
    unsafe fn storage(gl:&GL4, mut raw:RawTex, dim:Self::Dim) -> Self where Self::InternalFormat:SizedInternalFormat {
        let sampling = if Self::TARGET.multisample() {Some((0,false))} else {None};
        tex_storage::<Self>(gl, &mut raw, 1, dim, sampling);
        Self::from_raw(raw, dim)
    }

}

pub unsafe trait PixelTransfer: Texture {

    type BaseImage: Image<
        Dim = <Self as Texture>::Dim,
        PixelFormat = <Self as Texture>::PixelFormat,
        InternalFormat = <Self as Texture>::InternalFormat,
    >;

    #[inline] fn swizzle_r(&mut self, param:TextureSwizzle) where Self::InternalFormat: InternalFormatColor
        { unsafe{ swizzle_param(self, gl::TEXTURE_SWIZZLE_R, param) } }
    #[inline] fn swizzle_g(&mut self, param:TextureSwizzle) where Self::InternalFormat: InternalFormatColor
        { unsafe{ swizzle_param(self, gl::TEXTURE_SWIZZLE_G, param) } }
    #[inline] fn swizzle_b(&mut self, param:TextureSwizzle) where Self::InternalFormat: InternalFormatColor
        { unsafe{ swizzle_param(self, gl::TEXTURE_SWIZZLE_B, param) } }
    #[inline] fn swizzle_a(&mut self, param:TextureSwizzle) where Self::InternalFormat: InternalFormatColor
        { unsafe{ swizzle_param(self, gl::TEXTURE_SWIZZLE_A, param) } }

    fn swizzle_rgba(&mut self, red:TextureSwizzle, green:TextureSwizzle, blue:TextureSwizzle, alpha:TextureSwizzle)
    where Self::InternalFormat: InternalFormatColor
    {
        unsafe {
            let mut swizzle = [red as GLint, green as GLint, blue as GLint, alpha as GLint];
            if gl::TextureParameteriv::is_loaded() {
                gl::TextureParameteriv(self.id(), gl::TEXTURE_SWIZZLE_RGBA, &mut swizzle[0] as *mut GLint);
            } else {
                let mut target = Self::TARGET.as_loc();
                let binding = target.bind_unchecked(self.id());
                gl::TexParameteriv(binding.target_id(), gl::TEXTURE_SWIZZLE_RGBA, &mut swizzle[0] as *mut GLint);
            }
        }
    }

    #[inline] fn get_swizzle_r(&self) -> TextureSwizzle where Self::InternalFormat: InternalFormatColor
        { unsafe{ get_swizzle_param(self, gl::TEXTURE_SWIZZLE_R) } }
    #[inline] fn get_swizzle_g(&self) -> TextureSwizzle where Self::InternalFormat: InternalFormatColor
        { unsafe{ get_swizzle_param(self, gl::TEXTURE_SWIZZLE_G) } }
    #[inline] fn get_swizzle_b(&self) -> TextureSwizzle where Self::InternalFormat: InternalFormatColor
        { unsafe{ get_swizzle_param(self, gl::TEXTURE_SWIZZLE_B) } }
    #[inline] fn get_swizzle_a(&self) -> TextureSwizzle where Self::InternalFormat: InternalFormatColor
        { unsafe{ get_swizzle_param(self, gl::TEXTURE_SWIZZLE_A) } }

    fn get_swizzle_rgba(&self) -> [TextureSwizzle;4] where Self::InternalFormat: InternalFormatColor{
        unsafe {
            let mut swizzle = uninitialized::<[GLint;4]>();
            if gl::GetTextureParameteriv::is_loaded() {
                gl::GetTextureParameteriv(self.id(), gl::TEXTURE_SWIZZLE_RGBA, &mut swizzle[0] as *mut GLint);
            } else {
                let mut target = Self::TARGET.as_loc();
                let binding = target.bind_unchecked(self.id());
                gl::GetTexParameteriv(binding.target_id(), gl::TEXTURE_SWIZZLE_RGBA, &mut swizzle[0] as *mut GLint);
            }
            [
                (swizzle[0] as GLenum).try_into().unwrap(),
                (swizzle[1] as GLenum).try_into().unwrap(),
                (swizzle[2] as GLenum).try_into().unwrap(),
                (swizzle[3] as GLenum).try_into().unwrap(),
            ]
        }
    }

    fn from_pixels<P:PixelData<Self::PixelFormat>+?Sized>(gl:&GL2, dim:Self::Dim, data:&P) -> Self {
        let raw = RawTex::gen(gl);
        if let Ok(gl4) = gl.upgrade().and_then(|gl3| gl3.upgrade()) {
            if_sized!(
                helper(P:PixelData<T::PixelFormat>+?Sized)(_gl:&GL4,tex:RawTex,d:T::Dim,p:&P) -> T
                    {T::image_from_pixels(tex, d, p)}
                    {T::storage_from_pixels(_gl, tex, d, p)}
                where T:PixelTransfer
            );
            Self::InternalFormat::helper(&gl4, raw, dim, data)
        } else {
            Self::image_from_pixels(raw, dim, data)
        }

    }

    fn image_from_pixels<P:PixelData<Self::PixelFormat>+?Sized>(raw:RawTex, dim:Self::Dim, data:&P) -> Self {
        unsafe {
            tex_image_data::<Self, _>(raw.id(), 0, dim, data);
            Self::from_raw(raw, dim)
        }
    }

    fn storage_from_pixels<P>(gl:&GL4, raw:RawTex, dim:Self::Dim, data:&P) -> Self
        where P:PixelData<Self::PixelFormat>+?Sized, Self::InternalFormat:SizedInternalFormat
    {
        let mut tex = unsafe { Self::storage(gl, raw, dim) };
        tex.base_image().image(data);
        tex
    }

    fn base_image(&mut self) -> Self::BaseImage;
}

fn clamp_range<T:MipmappedTexture, R:RangeBounds<GLuint>>(t:&T, r:&R) -> (GLuint, GLuint) {
    (
        match r.start_bound() {
            Bound::Included(m) => *m,
            Bound::Excluded(m) => *m + 1,
            Bound::Unbounded => 0
        },
        match r.end_bound() {
            Bound::Included(m) => *m,
            Bound::Excluded(m) => *m - 1,
            Bound::Unbounded => t.dim().max_levels()
        }
    )
}

pub unsafe trait MipmappedTexture: PixelTransfer {

    #[inline] fn immutable_levels(&self) -> GLuint {unsafe{get_tex_parameter_iv(self, gl::TEXTURE_IMMUTABLE_LEVELS) as GLuint}}
    #[inline] fn base_level(&self) -> GLuint {unsafe{get_tex_parameter_iv(self, gl::TEXTURE_BASE_LEVEL) as GLuint}}
    #[inline] fn max_level(&self) -> GLuint {unsafe{get_tex_parameter_iv(self, gl::TEXTURE_MAX_LEVEL) as GLuint}}

    #[inline] fn set_base_level(&mut self, level: GLuint) {
        if cfg!(debug_assertions) && level > self.max_level() { panic!("Base level higher than current max level"); }
        unsafe { tex_parameter_iv(self, gl::TEXTURE_BASE_LEVEL, &(level as GLint) as *const GLint) }
    }

    #[inline] fn set_max_level(&mut self, level: GLuint) {
        if cfg!(debug_assertions) {
            if level < self.base_level() { panic!("Max level lower than max level"); }
            if self.immutable_storage() && level >= self.immutable_levels() {
                panic!("Base level higher than allocated immutable storage");
            }
        }
        unsafe { tex_parameter_iv(self, gl::TEXTURE_MAX_LEVEL, &(level as GLint) as *const GLint) }
    }

    #[inline]
    unsafe fn alloc_mipmaps(gl:&GL2, levels:GLuint, dim:Self::Dim) -> Self {
        let raw = RawTex::gen(gl);
        if let Ok(gl4) = gl.upgrade().and_then(|gl3| gl3.upgrade()) {
            if_sized!(
                helper()(_gl:&GL4,tex:RawTex,l:GLuint,d:T::Dim) -> T
                    {unsafe{T::image_mipmaps(tex, l, d)}}
                    {unsafe{T::storage_mipmaps(_gl, tex, l, d)}}
                where T:MipmappedTexture
            );
            Self::InternalFormat::helper(&gl4, raw, levels, dim)
        } else {
            Self::image_mipmaps(raw, levels, dim)
        }
    }

    #[inline]
    unsafe fn image_mipmaps(raw:RawTex, levels:GLuint, dim:Self::Dim) -> Self {
        let mut tex = Self::image(raw, dim);
        tex.set_max_level(levels);
        // tex.gen_mipmaps(0..levels);
        tex
    }

    unsafe fn storage_mipmaps(gl:&GL4, raw:RawTex, levels:GLuint, dim:Self::Dim) -> Self;

    fn from_mipmaps<P:PixelData<Self::PixelFormat>+?Sized>(
        gl:&GL2, levels:GLuint, dim:Self::Dim, base:&P, mipmaps: Option<HashMap<GLuint, &P>>
    ) -> Self {
        let raw = RawTex::gen(gl);
        if let Ok(gl4) = gl.upgrade().and_then(|gl3| gl3.upgrade()) {
            if_sized!(
                helper(P:PixelData<T::PixelFormat>+?Sized)(
                    _gl:&GL4, tex:RawTex, l:GLuint, d:T::Dim, b:&P, m: Option<HashMap<GLuint, &P>>
                ) -> T
                    {T::image_from_mipmaps(tex, l, d, b, m)}
                    {T::storage_from_mipmaps(_gl, tex, l, d, b, m)}
                where T:MipmappedTexture
            );
            Self::InternalFormat::helper(&gl4, raw, levels, dim, base, mipmaps)
        } else {
            Self::image_from_mipmaps(raw, levels, dim, base, mipmaps)
        }
    }

    fn image_from_mipmaps<P:PixelData<Self::PixelFormat>+?Sized>(
        raw:RawTex, levels:GLuint, dim:Self::Dim, base:&P, mipmaps: Option<HashMap<GLuint, &P>>
    ) -> Self {
        let mut tex = Self::image_from_pixels(raw, dim, base);
        tex.set_max_level(levels);
        match mipmaps {
            None => tex.gen_mipmaps(0..levels),
            Some(map) => {
                for (level, pixels) in map.into_iter() {
                    tex.level(level).image(pixels);
                }
            }
        }
        tex
    }

    fn storage_from_mipmaps<P>(
        gl:&GL4, raw:RawTex, levels:GLuint, dim:Self::Dim, base:&P, mipmaps: Option<HashMap<GLuint, &P>>
    ) -> Self where P:PixelData<Self::PixelFormat>+?Sized, Self::InternalFormat:SizedInternalFormat {
        let mut tex = unsafe { Self::storage_mipmaps(gl, raw, levels, dim) };
        tex.set_max_level(levels);
        tex.level(0).image(base);
        match mipmaps {
            None => tex.gen_mipmaps(0..levels),
            Some(map) => {
                let mut levels = Vec::with_capacity(map.len());
                for (level, pixels) in map.into_iter() {
                    //load the mipmap image
                    tex.level(level).image(pixels);

                    //insert the level and sort
                    let mut i = levels.len();
                    levels.push(level);
                    while i>0 {
                        if levels[i-1] > levels[i] { levels.swap(i-1, i) }
                        i -= 1;
                    }
                }

                //generate the empty levels
                for i in 0..levels.len()-1 {
                    if levels[i]+1 < levels[i+1] {
                        tex.gen_mipmaps(levels[i] .. levels[i+1]);
                    }
                }
            }
        }
        tex
    }

    fn gen_mipmaps<R:RangeBounds<GLuint>>(&mut self, range: R) {

        let (min, max) = clamp_range(self, &range);
        if max >= min {
            return;
        } else {
            let (prev_base, prev_max) = (self.base_level(), self.max_level());
            self.set_base_level(min);
            self.set_max_level(max);

            unsafe {
                let mut target = <Self as Texture>::TARGET.as_loc();
                let binding = target.bind_unchecked(self.id());
                gl::GenerateMipmap(binding.target_id());
            }

            self.set_base_level(prev_base);
            self.set_max_level(prev_max);
        }

    }

    #[inline] fn level(&mut self, level: GLuint) -> MipmapLevel<Self> {MipmapLevel{tex:self, level:level}}
    #[inline] fn level_range<R:RangeBounds<GLuint>>(&mut self, range: R) -> Box<[MipmapLevel<Self>]> {
        let (min, max) = clamp_range(self, &range);
        unsafe {
            let ptr = self as *mut Self;
            (min..=max).map(|i| (&mut *ptr).level(i)).collect::<Box<[_]>>()
        }
    }

}

pub unsafe trait MultisampledTexture: Texture {

    #[inline] fn samples(&self) -> GLuint { get_level_parameter_iv(self, 0, TexLevelParameteriv::Samples) as GLuint }
    #[inline] fn fixed_sample_locations(&self) -> bool {
        get_level_parameter_iv(self, 0, TexLevelParameteriv::FixedSampleLocations) != 0
    }

    #[inline]
    unsafe fn alloc_multisample(gl:&GL2, samples:GLuint, dim:Self::Dim, fixed_sample_locations: bool) -> Self {
        let raw = RawTex::gen(gl);
        if let Ok(gl4) = gl.upgrade().and_then(|gl3| gl3.upgrade()) {
            if_sized!(
                helper()(_gl:&GL4,tex:RawTex,s:GLuint,d:T::Dim,f:bool) -> T
                    {unsafe{T::image_multisample(tex, s, d, f)}}
                    {unsafe{T::storage_multisample(_gl, tex, s, d, f)}}
                where T:MultisampledTexture
            );
            Self::InternalFormat::helper(&gl4, raw, samples, dim, fixed_sample_locations)
        } else {
            Self::image_multisample(raw, samples, dim, fixed_sample_locations)
        }
    }

    unsafe fn image_multisample(mut raw:RawTex, samples:GLuint, dim:Self::Dim, fixed_sample_locations: bool) -> Self {
        tex_image_multisample::<Self>(&mut raw, dim, samples, fixed_sample_locations);
        Self::from_raw(raw, dim)
    }

    unsafe fn storage_multisample(gl:&GL4, raw:RawTex, samples:GLuint, dim:Self::Dim, fixed_sample_locations: bool) -> Self;

}

pub struct Tex1D<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;1],
    fmt: PhantomData<F>
}

pub struct Tex1DArray<F:InternalFormat> {
    raw: RawTex,
    dim: ([usize;1], usize),
    fmt: PhantomData<F>
}

pub struct Tex2D<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;2],
    fmt: PhantomData<F>
}

pub struct TexCubemap<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;2],
    fmt: PhantomData<F>
}

pub struct TexRectangle<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;2],
    fmt: PhantomData<F>
}

pub struct Tex2DMultisample<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;2],
    fmt: PhantomData<F>
}

pub struct Tex2DArray<F:InternalFormat> {
    raw: RawTex,
    dim: ([usize;2], usize),
    fmt: PhantomData<F>
}

pub struct TexCubemapArray<F:InternalFormat> {
    raw: RawTex,
    dim: (usize, [usize;2]),
    fmt: PhantomData<F>
}

pub struct Tex2DMultisampleArray<F:InternalFormat> {
    raw: RawTex,
    dim: ([usize;2], usize),
    fmt: PhantomData<F>
}

pub struct Tex3D<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;3],
    fmt: PhantomData<F>
}
