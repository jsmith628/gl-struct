use super::*;
use crate::gl;
use crate::Target;

use image_format::*;
use buffer_new::{UninitBuf, BufferTarget};

use std::convert::TryInto;
use std::marker::PhantomData;
use std::ops::{Bound,RangeBounds};
use std::collections::HashMap;
use std::mem::size_of;

pub use self::pixel_format::*;
pub use self::pixel_data::*;

mod pixel_format;
mod pixel_data;

glenum! {
    pub enum TextureTarget {
        [Texture1D TEXTURE_1D "Texture 1D"],
        [Texture2D TEXTURE_2D "Texture 2D"],
        [Texture3D TEXTURE_3D "Texture 3D"],
        [Texture1DArray TEXTURE_1D_ARRAY "Texture 1D Array"],
        [Texture2DArray TEXTURE_2D_ARRAY "Texture 2D Array"],
        [TextureRectangle TEXTURE_RECTANGLE "Texture Rectangle"],
        [TextureBuffer TEXTURE_BUFFER "Texture Buffer"],
        [TextureCubeMap TEXTURE_CUBE_MAP "Texture Cube Map"],
        [TextureCubeMapArray TEXTURE_CUBE_MAP_ARRAY "Texture Cube Map Array"],
        [Texture2DMultisample TEXTURE_2D_MULTISAMPLE "Texture 2D Multisample"],
        [Texture1DMultisampleArray TEXTURE_2D_MULTISAMPLE_ARRAY "Texture 2D Multisample Array"]
    }

    pub enum TextureSwizzle {
        [Red RED "Red Component"],
        [Green GREEN "Green Component"],
        [Blue BLUE "Blue Component"],
        [Alpha ALPHA "Alpha Component"],
        [Zero ZERO "Zero"],
        [One ONE "One"]
    }

    pub enum TexLevelParameteriv {
        [Width TEXTURE_WIDTH "Width"],
        [Height TEXTURE_HEIGHT "Height"],
        [Depth TEXTURE_DEPTH "Depth"],
        [Samples TEXTURE_SAMPLES "Samples"],
        [FixedSampleLocations TEXTURE_FIXED_SAMPLE_LOCATIONS "Fixed Sample Locations"],

        [InternalFormat TEXTURE_INTERNAL_FORMAT "Width"],

        [RedType TEXTURE_RED_TYPE "Red Type"],
        [GreenType TEXTURE_GREEN_TYPE "Green Type"],
        [BlueType TEXTURE_BLUE_TYPE "Blue Type"],
        [AlphaType TEXTURE_ALPHA_TYPE "Alpha Type"],
        [DepthType TEXTURE_DEPTH_TYPE "Depth Type"],

        [RedSize TEXTURE_RED_SIZE "Red Size"],
        [GreenSize TEXTURE_GREEN_SIZE "Green Size"],
        [BlueSize TEXTURE_BLUE_SIZE "Blue Size"],
        [AlphaSize TEXTURE_ALPHA_SIZE "Alpha Size"],
        [DepthSize TEXTURE_DEPTH_SIZE "Depth Size"],
        [StencilSize TEXTURE_STENCIL_SIZE "Stencil Size"],
        [SharedSize TEXTURE_SHARED_SIZE "Shared Size"],

        [Compressed TEXTURE_COMPRESSED "Compressed"],
        [CompressedImageSize TEXTURE_COMPRESSED_IMAGE_SIZE "Compressed Image Size"],

        [BufferDataStoreBinding TEXTURE_BUFFER_DATA_STORE_BINDING "Buffer Data Store Binding"],
        [BufferOffset TEXTURE_BUFFER_OFFSET "Buffer Offset"],
        [BufferSize TEXTURE_BUFFER_SIZE "Buffer Size"]
    }

}

impl TextureTarget {
    #[inline]
    fn multisample(self) -> bool {
        match self {
            Self::Texture2DMultisample | Self::Texture1DMultisampleArray => true,
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

pub unsafe trait TexDim:Copy {
    fn dim() -> usize;
    fn minimized(&self, level: GLuint) -> Self;

    #[inline] fn pixels(&self) -> usize {self.width() * self.height() * self.depth()}
    #[inline] fn max_levels(&self) -> GLuint {
        (0 as GLuint).leading_zeros() - (self.width().max(self.height().max(self.depth()))).leading_zeros()
    }

    #[inline] fn width(&self) -> usize {1}
    #[inline] fn height(&self) -> usize {1}
    #[inline] fn depth(&self) -> usize {1}

}

unsafe impl TexDim for [usize;1] {
    #[inline] fn dim() -> usize {1}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn minimized(&self, level: GLuint) -> Self { [(self[0] >> level).max(1)] }
}

unsafe impl TexDim for [usize;2] {
    #[inline] fn dim() -> usize {2}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn height(&self) -> usize {self[1]}
    #[inline] fn minimized(&self, level: GLuint) -> Self {
        [(self[0] >> level).max(1), (self[1] >> level).max(1)]
    }
}

unsafe impl TexDim for [usize;3] {
    #[inline] fn dim() -> usize {3}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn height(&self) -> usize {self[1]}
    #[inline] fn depth(&self) -> usize {self[2]}
    #[inline] fn minimized(&self, level: GLuint) -> Self {
        [(self[0] >> level).max(1), (self[1] >> level).max(1), (self[2] >> level).max(1)]
    }
}

unsafe impl TexDim for ([usize;1], usize) {
    #[inline] fn dim() -> usize {2}
    #[inline] fn minimized(&self, level: GLuint) -> Self {(self.0.minimized(level), self.1)}
    #[inline] fn max_levels(&self) -> GLuint {self.0.max_levels()}

    #[inline] fn width(&self) -> usize {self.0[0]}
    #[inline] fn height(&self) -> usize {self.1}
}

unsafe impl TexDim for ([usize;2], usize) {
    #[inline] fn dim() -> usize {3}
    #[inline] fn minimized(&self, level: GLuint) -> Self {(self.0.minimized(level), self.1)}
    #[inline] fn max_levels(&self) -> GLuint {self.0.max_levels()}

    #[inline] fn width(&self) -> usize {self.0[0]}
    #[inline] fn height(&self) -> usize {self.0[1]}
    #[inline] fn depth(&self) -> usize {self.1}
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

unsafe fn tex_image<T:Texture>(
    tex: GLuint, level: GLuint, dim: T::Dim, format_type: T::PixelFormat, data:*const GLvoid
) {
    let mut target = T::TARGET.as_loc();
    let binding = target.bind_raw(tex).unwrap();
    let int_fmt = T::InternalFormat::glenum() as GLint;
    let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);
    let (fmt, ty) = format_type.format_type();

    match T::Dim::dim() {
        1 => gl::TexImage1D(binding.target_id(), level as GLint, int_fmt, w, 0, fmt.into(), ty.into(), data),
        2 => gl::TexImage2D(binding.target_id(), level as GLint, int_fmt, w, h, 0, fmt.into(), ty.into(), data),
        3 => gl::TexImage3D(binding.target_id(), level as GLint, int_fmt, w, h, d, 0, fmt.into(), ty.into(), data),
        _ => panic!("{}D Textures not currently supported", T::Dim::dim())
    }
}

unsafe fn tex_image_multisample<T:Texture>(tex: &mut RawTex, dim: T::Dim, samples: GLsizei, fixed: bool) {
    let mut target = T::TARGET.as_loc();
    let binding = target.bind(tex);
    let int_fmt = T::InternalFormat::glenum();
    let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);

    match T::Dim::dim() {
        2 => gl::TexImage2DMultisample(binding.target_id(), samples, int_fmt, w, h, fixed as GLboolean),
        3 => gl::TexImage3DMultisample(binding.target_id(), samples, int_fmt, w, h, d, fixed as GLboolean),
        _ => panic!("{}D Multisample Textures not currently supported", T::Dim::dim())
    }
}

pub unsafe trait Texture: Sized {
    type InternalFormat: InternalFormat<FormatType=Self::PixelFormat>;
    type PixelFormat: PixelFormatType;
    type Dim: TexDim;

    const TARGET: TextureTarget;

    fn id(&self) -> GLuint;
    fn dim(&self) -> Self::Dim;
    fn immutable_storage(&self) -> bool;

    unsafe fn alloc(gl:&GL2, dim:Self::Dim) -> Self;
    unsafe fn image(raw:RawTex, dim:Self::Dim) -> Self;
    unsafe fn storage(gl:&GL4, raw:RawTex, dim:Self::Dim) -> Self where Self::InternalFormat:SizedInternalFormat;

}

pub unsafe trait PixelTransfer: Texture {

    type BaseImage: Image<
        Dim = <Self as Texture>::Dim,
        PixelFormat = <Self as Texture>::PixelFormat,
        InternalFormat = <Self as Texture>::InternalFormat,
    >;

    fn from_pixels<P:PixelData<Self::PixelFormat>+?Sized>(gl:&GL2, dim:Self::Dim, data:&P) -> Self;
    fn image_from_pixels<P:PixelData<Self::PixelFormat>+?Sized>(raw:RawTex, dim:Self::Dim, data:&P) -> Self;
    fn storage_from_pixels<P>(gl:&GL4, raw:RawTex, dim:Self::Dim, data:&P) -> Self
        where P:PixelData<Self::PixelFormat>+?Sized, Self::InternalFormat:SizedInternalFormat;

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

    fn allocated_levels(&self) -> GLuint;
    fn base_level(&self) -> GLuint;
    fn max_level(&self) -> GLuint;

    fn set_base_level(&mut self, level: GLuint);
    fn set_max_level(&mut self, level: GLuint);

    unsafe fn alloc_mipmaps(gl:&GL2, levels:GLuint, dim:Self::Dim) -> Self;
    unsafe fn image_mipmaps(gl:&GL2, levels:GLuint, dim:Self::Dim) -> Self;
    unsafe fn storage_mipmaps(gl:&GL2, levels:GLuint, dim:Self::Dim) -> Self;

    fn from_mipmaps<P:PixelData<Self::PixelFormat>>(
        gl:&GL2, levels:GLuint, dim:Self::Dim, base:P, mipmaps: Option<HashMap<GLuint, &P>>
    ) -> Self;

    fn image_from_mipmaps<P:PixelData<Self::PixelFormat>>(
        raw:RawTex, levels:GLuint, dim:Self::Dim, base:P, mipmaps: Option<HashMap<GLuint, &P>>
    ) -> Self;

    fn storage_from_mipmaps<P>(
        gl:&GL4, raw:RawTex, levels:GLuint, dim:Self::Dim, base:P, mipmaps: Option<HashMap<GLuint, &P>>
    ) -> Self where P:PixelData<Self::PixelFormat>, Self::InternalFormat:SizedInternalFormat;

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
                let binding = target.bind_raw(self.id()).unwrap();
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

    fn samples(&self) -> GLuint;
    fn fixed_sample_locations(&self) -> bool;

    unsafe fn alloc_multisample(gl:&GL2, samples:GLuint, dim:Self::Dim, fixed_sample_locations: bool) -> Self;
    unsafe fn image_multisample(raw:RawTex, samples:GLuint, dim:Self::Dim, fixed_sample_locations: bool) -> Self;
    unsafe fn storage_multisample(gl:&GL4, raw:RawTex, samples:GLuint, dim:Self::Dim, fixed_sample_locations: bool) -> Self;

}

pub unsafe trait Image: Sized {
    type InternalFormat: InternalFormat<FormatType=Self::PixelFormat>;
    type PixelFormat: PixelFormatType;
    type Dim: TexDim;

    const TARGET: TextureTarget;

    fn id(&self) -> GLuint;
    fn level(&self) -> GLuint;
    fn dim(&self) -> Self::Dim;

    fn image<P:PixelData<Self::PixelFormat>+?Sized>(&mut self, data:&P);
    fn sub_image<P:PixelData<Self::PixelFormat>+?Sized>(&mut self, offset:Self::Dim, dim:Self::Dim, data:&P);

    fn clear_image<P:PixelType<Self::PixelFormat>>(&mut self, data:P);
    fn clear_sub_image<P:PixelType<Self::PixelFormat>>(&mut self, offset:Self::Dim, dim:Self::Dim, data:P);

    fn get_image<P:PixelDataMut<Self::PixelFormat>+?Sized>(&self, data: &mut P);

    fn into_box<P:PixelType<Self::PixelFormat>>(&self) -> Box<[P]> {
        let size = size_of::<P>()*self.dim().pixels();
        let mut dest = Vec::with_capacity(size);
        unsafe { dest.set_len(size) };
        self.get_image(dest.as_mut_slice());
        dest.into_boxed_slice()
    }

}

pub struct MipmapLevel<'a, T:Texture> {
    tex: &'a mut T,
    level: GLuint
}

impl<'a, T:Texture> MipmapLevel<'a,T> {
    fn get_parameter_iv(&self, pname: TexLevelParameteriv) -> GLint {
        unsafe {
            let mut params = ::std::mem::uninitialized::<GLint>();
            let mut target = T::TARGET.as_loc();
            let binding = target.bind_raw(self.tex.id()).unwrap();
            gl::GetTexLevelParameteriv(
                binding.target_id(), self.level as GLint, pname as GLenum, &mut params as *mut GLint
            );
            params
        }

    }
}

impl<'a, T:Texture+PixelTransfer> MipmapLevel<'a,T> {
    #[inline]
    unsafe fn prepare_transfer<'b, P:PixelData<T::PixelFormat>+?Sized>(
        &self, data:&'b P, loc: &'b mut BindingLocation<UninitBuf>
    ) -> Option<Binding<'b, UninitBuf>> {
        let dim = self.dim();
        if data.count() != dim.pixels() {
            panic!("Invalid pixel count");
        } else {
            match loc.target() {
                BufferTarget::PixelUnpackBuffer => apply_unpacking_settings(data),
                BufferTarget::PixelPackBuffer => apply_packing_settings(data),
                _ => panic!("Invalid target for pixel buffer transfer")
            }
            data.bind_pixel_buffer(loc)
        }
    }
}

unsafe impl<'a, T:Texture+PixelTransfer> Image for MipmapLevel<'a, T> {
    type InternalFormat = T::InternalFormat;
    type PixelFormat = T::PixelFormat;
    type Dim = T::Dim;

    const TARGET: TextureTarget = T::TARGET;

    #[inline] fn id(&self) -> GLuint {self.tex.id()}
    #[inline] fn level(&self) -> GLuint {self.level}
    #[inline] fn dim(&self) -> Self::Dim { self.tex.dim().minimized(self.level)}

    fn image<P:PixelData<Self::PixelFormat>+?Sized>(&mut self, data:&P) {
        unsafe {
            let mut pixel_buf = BufferTarget::PixelUnpackBuffer.as_loc();
            let _buf = self.prepare_transfer(data, &mut pixel_buf);
            tex_image::<T>(self.id(),self.level(),self.dim(),data.format_type(),data.pixels());
        }
    }

    fn sub_image<P:PixelData<Self::PixelFormat>+?Sized>(&mut self, offset:Self::Dim, dim:Self::Dim, data:&P) {
        unsafe {
            let mut pixel_buf = BufferTarget::PixelUnpackBuffer.as_loc();
            let _buf = self.prepare_transfer(data, &mut pixel_buf);

            let mut target = T::TARGET.as_loc();
            let binding = target.bind_raw(self.id()).unwrap();
            let level = self.level() as GLint;

            //TODO add error checking for dimensions
            let (x,y,z) = (offset.width() as GLsizei, offset.height() as GLsizei, offset.depth() as GLsizei);
            let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);
            let (fmt, ty) = data.format_type().format_type();

            match T::Dim::dim() {
                1 => gl::TexSubImage1D(binding.target_id(), level, x, w, fmt.into(), ty.into(), data.pixels()),
                2 => gl::TexSubImage2D(binding.target_id(), level, x,y, w,h, fmt.into(), ty.into(), data.pixels()),
                3 => gl::TexSubImage3D(binding.target_id(), level, x,y,z, w,h,d, fmt.into(), ty.into(), data.pixels()),
                _ => panic!("{}D Textures not currently supported", T::Dim::dim())
            }
        }
    }

    fn clear_image<P:PixelType<Self::PixelFormat>>(&mut self, data:P) {
        //TODO: provide some sort of fallback for if glClearTexImage isn't available
        unsafe {
            let (fmt, ty) = P::format_type().format_type();
            gl::ClearTexImage(self.id(), self.level() as GLint, fmt.into(), ty.into(), &data as *const P as *const GLvoid);
        }
    }
    fn clear_sub_image<P:PixelType<Self::PixelFormat>>(&mut self, offset:Self::Dim, dim:Self::Dim, data:P) {
        //TODO: provide some sort of fallback for if glClearTexImage isn't available
        unsafe {
            let (fmt, ty) = P::format_type().format_type();
            let (x,y,z) = (offset.width() as GLsizei, offset.height() as GLsizei, offset.depth() as GLsizei);
            let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);
            gl::ClearTexSubImage(self.id(), self.level() as GLint, x,y,z, w,h,d, fmt.into(), ty.into(), &data as *const P as *const GLvoid);
        }
    }

    fn get_image<P:PixelDataMut<Self::PixelFormat>+?Sized>(&self, data: &mut P) {
        unsafe {
            let (fmt, ty) = data.format_type().format_type();

            if gl::GetTextureImage::is_loaded() {
                gl::GetTextureImage(self.id(), self.level() as GLint, fmt.into(), ty.into(), data.size() as GLsizei, data.pixels_mut());
            } else {
                let mut target = T::TARGET.as_loc();
                let binding = target.bind_raw(self.id()).unwrap();
                gl::GetTexImage(binding.target_id(), self.level() as GLint, fmt.into(), ty.into(), data.pixels_mut());
            }

        }
    }

}


pub struct Tex1D<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;1],
    fmt: PhantomData<F>
}

pub struct Tex1DArray<F:InternalFormat> {
    raw: RawTex,
    dim: (usize, [usize;1]),
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
    dim: (usize, [usize;2]),
    fmt: PhantomData<F>
}

pub struct TexCubemapArray<F:InternalFormat> {
    raw: RawTex,
    dim: (usize, [usize;2]),
    fmt: PhantomData<F>
}

pub struct Tex2DMultisampleArray<F:InternalFormat> {
    raw: RawTex,
    dim: (usize, [usize;2]),
    fmt: PhantomData<F>
}

pub struct Tex3D<F:InternalFormat> {
    raw: RawTex,
    dim: [usize;3],
    fmt: PhantomData<F>
}
