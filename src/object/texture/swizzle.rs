use super::*;

glenum! {
    pub enum SwizzleParameter {
        [Red RED],
        [Green GREEN],
        [Blue BLUE],
        [Alpha ALPHA],
        [Zero ZERO],
        [One ONE]
    }
}

#[allow(unused_variables)]
impl<F:ColorFormat, T:SampledTarget<F>> Texture<F,T> {

    pub fn swizzle_r(&mut self, gl:&GL33, swizzle: SwizzleParameter) {
        unsafe { self.parameter_iv(gl::TEXTURE_SWIZZLE_R, &(swizzle as GLint)) }
    }

    pub fn swizzle_g(&mut self, gl:&GL33, swizzle: SwizzleParameter) {
        unsafe { self.parameter_iv(gl::TEXTURE_SWIZZLE_G, &(swizzle as GLint)) }
    }

    pub fn swizzle_b(&mut self, gl:&GL33, swizzle: SwizzleParameter) {
        unsafe { self.parameter_iv(gl::TEXTURE_SWIZZLE_B, &(swizzle as GLint)) }
    }

    pub fn swizzle_a(&mut self, gl:&GL33, swizzle: SwizzleParameter) {
        unsafe { self.parameter_iv(gl::TEXTURE_SWIZZLE_A, &(swizzle as GLint)) }
    }

    pub fn swizzle_rgba(&mut self, gl:&GL33, swizzle: [SwizzleParameter; 4]) {
        unsafe {
            let array = [swizzle[0] as GLint, swizzle[1] as GLint, swizzle[2] as GLint, swizzle[3] as GLint];
            self.parameter_iv(gl::TEXTURE_SWIZZLE_RGBA, &array[0])
        }
    }

    pub fn get_swizzle_r(&self, gl:&GL33) -> SwizzleParameter {
        unsafe { (self.get_parameter_i(gl::TEXTURE_SWIZZLE_R) as GLenum).try_into().unwrap() }
    }

    pub fn get_swizzle_g(&self, gl:&GL33) -> SwizzleParameter {
        unsafe { (self.get_parameter_i(gl::TEXTURE_SWIZZLE_G) as GLenum).try_into().unwrap() }
    }

    pub fn get_swizzle_b(&self, gl:&GL33) -> SwizzleParameter {
        unsafe { (self.get_parameter_i(gl::TEXTURE_SWIZZLE_B) as GLenum).try_into().unwrap() }
    }

    pub fn get_swizzle_a(&self, gl:&GL33) -> SwizzleParameter {
        unsafe { (self.get_parameter_i(gl::TEXTURE_SWIZZLE_A) as GLenum).try_into().unwrap() }
    }

    pub fn get_swizzle_rgba(&self, gl:&GL33) -> [SwizzleParameter; 4] {
        unsafe {
            let mut param = MaybeUninit::<[GLint;4]>::uninit();
            self.get_parameter_iv(gl::TEXTURE_SWIZZLE_RGBA, param.as_mut_ptr() as *mut _);

            let rgba: [GLint; 4] = param.assume_init();
            [
                (rgba[0] as GLenum).try_into().unwrap(),
                (rgba[1] as GLenum).try_into().unwrap(),
                (rgba[2] as GLenum).try_into().unwrap(),
                (rgba[3] as GLenum).try_into().unwrap(),
            ]
        }
    }

}
