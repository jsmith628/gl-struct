use super::*;

pub unsafe fn tex_storage<T:Texture>(
    _gl:&GL43, mut tex: RawTex<T::Target>, levels: GLuint, dim: T::Dim, sampling: Option<(GLuint,bool)>
) -> T where T::InternalFormat: SizedInternalFormat {
    let mut target = T::Target::binding_location();
    let binding = target.bind(&mut tex);
    let fmt = T::InternalFormat::glenum();
    let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);
    let coords = T::Dim::dim();

    match sampling {
        Some((samples, fixed)) => match coords {
            2 => gl::TexStorage2DMultisample(binding.target_id(), samples as GLsizei, fmt, w, h, fixed as GLboolean),
            3 => gl::TexStorage3DMultisample(binding.target_id(), samples as GLsizei, fmt, w, h, d, fixed as GLboolean),
            _ => panic!("{}D Multisample textures not currently supported", coords)
        },
        None => match coords {
            1 => gl::TexStorage1D(binding.target_id(), levels as GLsizei, fmt, w),
            2 => gl::TexStorage2D(binding.target_id(), levels as GLsizei, fmt, w, h),
            3 => gl::TexStorage3D(binding.target_id(), levels as GLsizei, fmt, w, h, d),
            _ => panic!("{}D Textures not currently supported", coords)
        }
    }
    drop(binding);

    T::from_raw(tex, dim)
}

pub unsafe fn tex_image_data<T:Texture, P:PixelData<T::ClientFormat>+?Sized>(
    tex: GLuint, level: GLuint, dim: T::Dim, data:&P
) {
    let (fmt,ty) = data.format_type().format_type();
    let mut pixel_buf = BufferTarget::PixelUnpackBuffer.as_loc();
    let (_buf, ptr) = data.pixels(&mut pixel_buf);
    tex_image::<T>(tex, level, dim, fmt.into(), ty.into(), ptr);
    drop(_buf)
}

#[inline]
pub unsafe fn tex_image_null<T:Texture>(tex: GLuint, level: GLuint, dim: T::Dim) {
    tex_image::<T>(tex, level, dim, 0, 0, ::std::ptr::null());
}

pub unsafe fn tex_image<T:Texture>(
    tex: GLuint, level: GLuint, dim: T::Dim, fmt:GLenum, ty:GLenum, data:*const GLvoid
) {
    //bind the texture
    let mut target = T::Target::binding_location();
    let binding = target.bind_unchecked(tex);

    //convert and rename params
    let int_fmt = T::InternalFormat::glenum() as GLint;
    let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);
    let coords = T::Dim::dim();

    //now, select the right function based on the dimensionality of the texture
    match coords {
        1 => gl::TexImage1D(binding.target_id(), level as GLint, int_fmt, w, 0, fmt, ty, data),
        2 => gl::TexImage2D(binding.target_id(), level as GLint, int_fmt, w, h, 0, fmt, ty, data),
        3 => gl::TexImage3D(binding.target_id(), level as GLint, int_fmt, w, h, d, 0, fmt, ty, data),
        _ => panic!("{}D Textures not currently supported", coords)
    }
}

pub unsafe fn tex_image_multisample<T:Texture>(
    tex: &mut RawTex<T::Target>, dim: T::Dim, samples: GLuint, fixed: bool
) {
    let mut target = T::Target::binding_location();
    let binding = target.bind(tex);
    let int_fmt = T::InternalFormat::glenum();
    let (w,h,d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);
    let coords = T::Dim::dim();

    match coords {
        2 => gl::TexImage2DMultisample(binding.target_id(), samples as GLsizei, int_fmt, w, h, fixed as GLboolean),
        3 => gl::TexImage3DMultisample(binding.target_id(), samples as GLsizei, int_fmt, w, h, d, fixed as GLboolean),
        _ => panic!("{}D Multisample Textures not currently supported", coords)
    }
}

pub unsafe fn tex_parameter_iv<T:Texture>(tex:&mut T, pname:GLenum, params: *const GLint) {
    if gl::TextureParameteriv::is_loaded() {
        gl::TextureParameteriv(tex.raw().id(), pname, params);
    } else {
        let mut target = T::Target::binding_location();
        let binding = target.bind(tex.raw());
        gl::TexParameteriv(binding.target_id(), pname, params);
    }
}

pub unsafe fn get_tex_parameter_iv<T:Texture>(tex:&T, pname:GLenum) -> GLint {
    let mut param = ::std::mem::uninitialized();
    if gl::GetTextureParameteriv::is_loaded() {
        gl::GetTextureParameteriv(tex.raw().id(), pname, &mut param as *mut GLint);
    } else {
        let mut target = T::Target::binding_location();
        let binding = target.bind(tex.raw());
        gl::GetTexParameteriv(binding.target_id(), pname, &mut param as *mut GLint);
    }
    param
}

#[inline]
pub unsafe fn get_swizzle_param<T:Texture>(tex:&T, pname:GLenum) -> TextureSwizzle {
    (get_tex_parameter_iv(tex, pname) as GLenum).try_into().unwrap()
}

#[inline]
pub unsafe fn swizzle_param<T:Texture>(tex:&mut T, pname:GLenum, param:TextureSwizzle) {
    tex_parameter_iv(tex, pname, &mut (param as GLint) as *mut GLint)
}

macro_rules! if_sized {
    ($name:ident($($gen:tt)*)($($param:ident: $ty:ty),*) -> $ret:ty {$($c1:tt)*} {$($c2:tt)*} where $($rest:tt)* ) => {
        trait Helper<T:Texture>: InternalFormat {
            fn $name<$($gen)*>($($param: $ty),*) -> $ret where $($rest)*;
        }

        impl<F,T> Helper<T> for F
        where F:InternalFormat, T:Texture<InternalFormat=F,ClientFormat=F::ClientFormat>
        {
            #[inline] default fn $name<$($gen)*>($($param: $ty),*) -> $ret where $($rest)* {$($c1)*}
        }

        impl<F,T> Helper<T> for F
        where F:SizedInternalFormat, T:Texture<InternalFormat=F,ClientFormat=F::ClientFormat>
        {
            #[inline] fn $name<$($gen)*>($($param: $ty),*) -> $ret where $($rest)* {$($c2)*}
        }
    }
}
