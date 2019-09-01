use super::*;

pub struct TexRectangle<F:InternalFormat> {
    raw: RawTex<TEXTURE_RECTANGLE>,
    dim: [usize;2],
    fmt: PhantomData<F>
}

pub struct Image2DRect<F:InternalFormat> {
    tex: *mut TexRectangle<F>
}

unsafe impl<F:InternalFormat> Texture for TexRectangle<F> {
    type InternalFormat = F;
    type Target = TEXTURE_RECTANGLE;

    //the types that are effectively aliases
    type ClientFormat = F::ClientFormat;
    type Dim = <TEXTURE_RECTANGLE as TextureTarget>::Dim;
    type GL = <TEXTURE_RECTANGLE as TextureTarget>::GL;

    #[inline] fn dim(&self) -> Self::Dim {self.dim}
    #[inline] fn raw(&self) -> &RawTex<Self::Target> {&self.raw}

    #[inline] unsafe fn from_raw(raw:RawTex<Self::Target>, dim:Self::Dim) -> Self {
        TexRectangle { raw: raw, dim: dim, fmt: PhantomData }
    }
}

unsafe impl<F:InternalFormat> PixelTransfer for TexRectangle<F> {
    type BaseImage = Image2DRect<F>;
    #[inline] fn base_image(&mut self) -> Self::BaseImage {Image2DRect {tex: self}}
}

impl<F:InternalFormat> Image2DRect<F> {
    unsafe fn as_mipmap_level(&self) -> MipmapLevel<TexRectangle<F>> {MipmapLevel { tex:self.tex, level:0 }}
}

unsafe impl<F:InternalFormat> Image for Image2DRect<F> {
    type InternalFormat = F;
    type ClientFormat = F::ClientFormat;
    type Dim = <TEXTURE_RECTANGLE as TextureTarget>::Dim;
    type Target = TEXTURE_RECTANGLE;

    #[inline] fn level(&self) -> GLuint {0}
    #[inline] fn dim(&self) -> Self::Dim { unsafe {&*self.tex}.dim()}
    #[inline] fn raw(&self) -> &RawTex<TEXTURE_RECTANGLE> { unsafe { (&*self.tex).raw() } }

    #[inline]
    fn image<P:PixelData<Self::ClientFormat>+?Sized>(&mut self, data:&P) {
        unsafe { self.as_mipmap_level().image(data) }
    }

    #[inline]
    fn sub_image<P:PixelData<Self::ClientFormat>+?Sized>(&mut self, offset:Self::Dim, dim:Self::Dim, data:&P) {
        unsafe { self.as_mipmap_level().sub_image(offset,dim,data) }
    }

    #[inline]
    fn clear_image<P:PixelType<Self::ClientFormat>>(&mut self, data:P) {
        unsafe { self.as_mipmap_level().clear_image(data) }
    }

    #[inline]
    fn clear_sub_image<P:PixelType<Self::ClientFormat>>(&mut self, offset:Self::Dim, dim:Self::Dim, data:P) {
        unsafe { self.as_mipmap_level().clear_sub_image(offset, dim, data) }
    }

    fn get_image<P:PixelDataMut<Self::ClientFormat>+?Sized>(&self, data: &mut P) {
        unsafe { self.as_mipmap_level().get_image(data) }
    }

}
