use super::*;
use crate::texture::TextureUnitID;

gl_resource!{
    pub struct Sampler {
        gl = GL3,
        target = TextureUnitID,
        gen = GenSamplers,
        bind = BindSampler,
        is = IsSampler,
        delete = DeleteSamplers
    }
}
