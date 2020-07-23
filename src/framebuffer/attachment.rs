use super::*;

pub struct FramebufferAttachment<'a,'b,F:'b> {
    fb: GLuint,
    attachment: GLenum,
    reference: PhantomData<&'a Framebuffer<'b,DEPTH_STENCIL,(F,)>>
}

pub struct FramebufferAttachmentMut<'a,'b,F:'b> {
    fb: GLuint,
    attachment: GLenum,
    reference: PhantomData<&'a mut Framebuffer<'b,DEPTH_STENCIL,(F,)>>
}
