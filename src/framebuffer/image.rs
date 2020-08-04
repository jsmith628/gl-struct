use super::*;

pub struct FramebufferImage<'a, F> {
    raw: RawFramebufferImage,
    data: PhantomData<(Option<TexImageMut<'a,F,!>>, Option<&'a Renderbuffer<F>>)>
}

pub(super) enum RawFramebufferImage {
    Renderbuffer{ id: GLuint },
    TextureLevel{ id: GLuint, level: GLuint },
    TextureLayer{ id: GLuint, level: GLuint, layer: GLuint }
}

pub unsafe trait RenderableImage<'a> {
    type GLSL;
    fn raw_image(self) -> FramebufferImage<'a, Self::GLSL>;
}

pub unsafe trait FragData: InternalFormat {
    type GLSL: GLSLType;
}

unsafe impl<'a, F> RenderableImage<'a> for FramebufferImage<'a,F> {
    type GLSL = F;
    fn raw_image(self) -> FramebufferImage<'a, F> { self }
}

unsafe impl<'a, F:FragData> RenderableImage<'a> for &'a Renderbuffer<F> {
    type GLSL = F::GLSL;
}
unsafe impl<'a, F:FragData, MS:RenderbufferMSFormat> RenderableImage<'a> for &'a Renderbuffer<F,MS> {
    default type GLSL = Multisampled<MS, F::GLSL>;
    fn raw_image(self) -> FramebufferImage<'a, Self::GLSL> {
        FramebufferImage {
            raw: RawFramebufferImage::Renderbuffer { id: self.id() },
            data: PhantomData
        }
    }
}

unsafe impl<'a, F:FragData, T:OwnedTarget<F>> RenderableImage<'a> for TexImageMut<'a,F,T> {

    default type GLSL = F::GLSL;
    default fn raw_image(self) -> FramebufferImage<'a, Self::GLSL> {
        FramebufferImage {
            raw: RawFramebufferImage::TextureLevel { id: self.id(), level: self.level() },
            data: PhantomData
        }
    }

}

unsafe impl<'a, F:FragData, T:LayeredTarget<F>+crate::texture::Layered> RenderableImage<'a> for TexImageMut<'a,F,T> {
    default type GLSL = Layered<T, F::GLSL>;
}

unsafe impl<'a, F:FragData, MS: MultisampleFormat> RenderableImage<'a> for TexImageMut<'a,F,TEXTURE_2D_MULTISAMPLE<MS>>
where TEXTURE_2D_MULTISAMPLE<MS>: OwnedTarget<F>
{
    type GLSL = Multisampled<MS, F::GLSL>;
}

unsafe impl<'a, F:FragData> RenderableImage<'a> for TexImageMut<'a,F,TEXTURE_CUBE_MAP>
where TEXTURE_CUBE_MAP: OwnedTarget<F>
{
    type GLSL = F::GLSL;
    fn raw_image(self) -> FramebufferImage<'a, Self::GLSL> {
        FramebufferImage {
            raw: RawFramebufferImage::TextureLayer {
                id: self.id(), level: self.level(), layer: self.face().into()
            },
            data: PhantomData
        }
    }
}
