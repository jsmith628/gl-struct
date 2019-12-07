use super::*;
use object::sampler::*;

impl<F:InternalFormat, T:SampledTarget<F>> Texture<F,T> {

    //TODO: exclude rectangle textures

    pub fn wrap_s(&mut self, wrapping: Wrapping) {
        unsafe { self.parameter_iv(gl::TEXTURE_WRAP_S, &(GLenum::from(wrapping) as GLint)) }
    }

    pub fn wrap_t(&mut self, wrapping: Wrapping) {
        if T::Dim::dim()>=2 {
            unsafe { self.parameter_iv(gl::TEXTURE_WRAP_T, &(GLenum::from(wrapping) as GLint)) }
        }
    }

    pub fn wrap_r(&mut self, wrapping: Wrapping) {
        if T::Dim::dim()>=3 {
            unsafe { self.parameter_iv(gl::TEXTURE_WRAP_R, &(GLenum::from(wrapping) as GLint)) }
        }
    }

    pub fn mag_filter(&mut self, filter: MagFilter) {
        unsafe { self.parameter_iv(gl::TEXTURE_MAG_FILTER, &(GLenum::from(filter) as GLint)) }
    }

    pub fn min_filter_non_mipmap(&mut self, filter: MagFilter) {
        unsafe { self.parameter_iv(gl::TEXTURE_MIN_FILTER, &(GLenum::from(filter) as GLint)) }
    }

    pub fn min_filter(&mut self, filter: MinFilter) where T: MipmappedTarget<F> {
        unsafe { self.parameter_iv(gl::TEXTURE_MIN_FILTER, &(GLenum::from(filter) as GLint)) }
    }

    #[allow(unused_variables)]
    pub fn min_lod(&mut self, gl:&GL12, value:GLfloat) {
        unsafe { self.parameter_fv(gl::TEXTURE_MIN_LOD, &value) }
    }

    #[allow(unused_variables)]
    pub fn max_lod(&mut self, gl:&GL12, value:GLfloat) {
        unsafe { self.parameter_fv(gl::TEXTURE_MAX_LOD, &value) }
    }

    pub fn border_color(&mut self, color: [GLfloat; 4]) where F: FloatFormat {
        unsafe { self.parameter_fv(gl::TEXTURE_BORDER_COLOR, &color[0]) }
    }

    pub fn border_color_normalized(&mut self, color: [GLint; 4]) where F: FloatFormat {
        unsafe { self.parameter_iv(gl::TEXTURE_BORDER_COLOR, &color[0]) }
    }

    pub fn border_color_int(&mut self, color: [GLint; 4]) where F: IntFormat {
        unsafe { self.parameter_i_iv(gl::TEXTURE_BORDER_COLOR, &color[0]) }
    }

    pub fn border_color_uint(&mut self, color: [GLuint; 4]) where F: UIntFormat {
        unsafe { self.parameter_i_uiv(gl::TEXTURE_BORDER_COLOR, &color[0]) }
    }

    pub fn border_color_stencil(&mut self, value: GLuint) where F: StencilFormat {
        let color = [value, 0, 0, 0];
        unsafe { self.parameter_i_uiv(gl::TEXTURE_BORDER_COLOR, &color[0]) }
    }

    pub fn border_color_depth(&mut self, value: GLfloat) where F: DepthFormat {
        let color = [value, 0.0, 0.0, 0.0];
        unsafe { self.parameter_fv(gl::TEXTURE_BORDER_COLOR, &color[0]) }
    }

    pub fn border_color_depth_normalized(&mut self, value: GLint) where F: DepthFormat {
        let color = [value, 0, 0, 0];
        unsafe { self.parameter_iv(gl::TEXTURE_BORDER_COLOR, &color[0]) }
    }

    pub fn compare_mode(&mut self, mode: CompareMode) where F: DepthFormat {
        unsafe { self.parameter_iv(gl::TEXTURE_COMPARE_MODE, &(GLenum::from(mode) as GLint)) }
    }

    pub fn compare_func(&mut self, func: CompareFunc) where F: DepthFormat {
        unsafe { self.parameter_iv(gl::TEXTURE_COMPARE_FUNC, &(GLenum::from(func) as GLint)) }
    }

    pub fn get_wrap_s(&self) -> Wrapping {
        unsafe { (self.get_parameter_i(gl::TEXTURE_WRAP_S) as GLenum).try_into().unwrap() }
    }

    pub fn get_wrap_t(&self) -> Wrapping {
        if T::Dim::dim()>=2 {
            unsafe { (self.get_parameter_i(gl::TEXTURE_WRAP_T) as GLenum).try_into().unwrap() }
        } else {
            Default::default()
        }
    }

    pub fn get_wrap_r(&self) -> Wrapping {
        if T::Dim::dim()>=3 {
            unsafe { (self.get_parameter_i(gl::TEXTURE_WRAP_R) as GLenum).try_into().unwrap() }
        } else if T::glenum()==gl::TEXTURE_RECTANGLE {
            Wrapping::ClampToEdge(unsafe {assume_supported()})
        } else {
            Default::default()
        }
    }

    pub fn get_mag_filter(&self) -> MagFilter {
        unsafe { (self.get_parameter_i(gl::TEXTURE_MAG_FILTER) as GLenum).try_into().unwrap() }
    }

    pub fn get_min_filter(&self) -> MinFilter {
        unsafe { (self.get_parameter_i(gl::TEXTURE_MIN_FILTER) as GLenum).try_into().unwrap() }
    }

    #[allow(unused_variables)]
    pub fn get_min_lod(&self, gl:&GL12) -> GLfloat {
        unsafe { self.get_parameter_f(gl::TEXTURE_MIN_LOD) }
    }

    #[allow(unused_variables)]
    pub fn get_max_lod(&self, gl:&GL12) -> GLfloat {
        unsafe { self.get_parameter_f(gl::TEXTURE_MAX_LOD) }
    }

    pub fn get_border_color(&self) -> [GLfloat;4] where F: FloatFormat {
        let mut param = MaybeUninit::<[_;4]>::uninit();
        unsafe {
            self.get_parameter_fv(gl::TEXTURE_BORDER_COLOR, param.as_mut_ptr() as *mut _);
            param.assume_init()
        }
    }

    pub fn get_border_color_normalized(&self) -> [GLint;4] where F: FloatFormat {
        let mut param = MaybeUninit::<[_;4]>::uninit();
        unsafe {
            self.get_parameter_iv(gl::TEXTURE_BORDER_COLOR, param.as_mut_ptr() as *mut _);
            param.assume_init()
        }
    }

    pub fn get_border_color_int(&self) -> [GLint;4] where F: IntFormat {
        let mut param = MaybeUninit::<[_;4]>::uninit();
        unsafe {
            self.get_parameter_i_iv(gl::TEXTURE_BORDER_COLOR, param.as_mut_ptr() as *mut _);
            param.assume_init()
        }
    }

    pub fn get_border_color_stencil(&self) -> GLuint where F: UIntFormat {
        let mut param = MaybeUninit::<[_;4]>::uninit();
        unsafe {
            self.get_parameter_i_uiv(gl::TEXTURE_BORDER_COLOR, param.as_mut_ptr() as *mut _);
            param.assume_init()[0]
        }
    }

    pub fn get_border_color_depth(&self) -> GLfloat where F: UIntFormat {
        let mut param = MaybeUninit::<[_;4]>::uninit();
        unsafe {
            self.get_parameter_fv(gl::TEXTURE_BORDER_COLOR, param.as_mut_ptr() as *mut _);
            param.assume_init()[0]
        }
    }

    pub fn get_border_color_depth_normalized(&self) -> GLint where F: UIntFormat {
        let mut param = MaybeUninit::<[_;4]>::uninit();
        unsafe {
            self.get_parameter_iv(gl::TEXTURE_BORDER_COLOR, param.as_mut_ptr() as *mut _);
            param.assume_init()[0]
        }
    }


    pub fn get_compare_mode(&self) -> CompareMode where F: DepthFormat {
        unsafe { (self.get_parameter_i(gl::TEXTURE_COMPARE_MODE) as GLenum).try_into().unwrap() }
    }

    pub fn get_compare_func(&self) -> CompareMode where F: DepthFormat {
        unsafe { (self.get_parameter_i(gl::TEXTURE_COMPARE_MODE) as GLenum).try_into().unwrap() }
    }

}
