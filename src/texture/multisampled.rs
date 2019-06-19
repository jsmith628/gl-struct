use super::*;

pub unsafe trait MultisampledTexture: Texture {

    #[inline] fn samples(&self) -> GLuint { get_level_parameter_iv(self, 0, TexLevelParameteriv::Samples) as GLuint }
    #[inline] fn fixed_sample_locations(&self) -> bool {
        get_level_parameter_iv(self, 0, TexLevelParameteriv::FixedSampleLocations) != 0
    }

    #[inline]
    unsafe fn alloc_multisample(
        gl:&Self::GL, samples:GLuint, dim:Self::Dim, fixed_sample_locations: bool
    ) -> Self {
        let raw = RawTex::gen(gl);
        if let Ok(gl4) = gl.try_as_gl4() {
            if_sized!(
                helper()(_gl:&GL4,tex:RawTex<T::Target>,s:GLuint,d:T::Dim,f:bool) -> T
                    {unsafe{T::image_multisample(tex, s, d, f)}}
                    {unsafe{T::storage_multisample(_gl, tex, s, d, f)}}
                where T:MultisampledTexture
            );
            Self::InternalFormat::helper(&gl4, raw, samples, dim, fixed_sample_locations)
        } else {
            Self::image_multisample(raw, samples, dim, fixed_sample_locations)
        }
    }

    #[inline]
    unsafe fn image_multisample(mut raw:RawTex<Self::Target>, samples:GLuint, dim:Self::Dim, fixed_sample_locations: bool) -> Self {
        tex_image_multisample::<Self>(&mut raw, dim, samples, fixed_sample_locations);
        Self::from_raw(raw, dim)
    }

    #[inline]
    unsafe fn storage_multisample(
        gl:&GL4, raw:RawTex<Self::Target>, samples:GLuint, dim:Self::Dim, fixed_sample_locations: bool
    ) -> Self
    where <Self as Texture>::InternalFormat: SizedInternalFormat
    {
        tex_storage::<Self>(gl, raw, 1, dim, Some((samples, fixed_sample_locations)))
    }

}
