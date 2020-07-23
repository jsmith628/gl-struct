use super::*;

pub struct Bitplane<F> {
    target: GLenum,
    id: GLuint,
    level: GLuint,
    layer: GLuint,
    data: PhantomData<*mut F>
}

impl<F> Bitplane<F> {

    pub fn target(&self) -> GLenum { self.target }
    pub fn id(&self) -> GLuint { self.id }
    pub fn level(&self) -> GLuint { self.level }
    pub fn layer(&self) -> GLuint { self.layer }

}
