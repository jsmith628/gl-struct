use super::*;

pub type UninitTex<T> = Texture<!,T>;

impl<T:TextureType> UninitTex<T> {
    pub fn gen(#[allow(unused_variables)] gl: &T::GL) -> GLuint {
        unsafe {
            let mut tex = MaybeUninit::uninit();
            gl::GenTextures(1, tex.as_mut_ptr());
            tex.assume_init()
        }
    }

    pub fn gen_textures(#[allow(unused_variables)] gl: &T::GL, n: GLuint) -> Box<[GLuint]> {
        if n==0 { return Box::new([]); }
        unsafe {
            let mut tex = Box::new_uninit_slice(n as usize);
            gl::GenTextures(tex.len().try_into().unwrap(), MaybeUninit::slice_as_mut_ptr(&mut *tex));
            tex.assume_init()
        }
    }

    pub fn create(#[allow(unused_variables)] gl: &T::GL) -> Self {
        let mut tex:MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe {
            if gl::CreateTextures::is_loaded() {
                gl::CreateTextures(T::glenum(), 1, tex.as_mut_ptr() as *mut GLuint);
            } else {
                gl::GenTextures(1, tex.as_mut_ptr() as *mut GLuint);
                gl::BindTexture(T::glenum(), tex.assume_init_mut().id());
                gl::BindTexture(T::glenum(), 0);
            }
            tex.assume_init()
        }
    }

    pub fn create_textures(#[allow(unused_variables)] gl: &T::GL, n: GLuint) -> Box<[Self]> {
        if n==0 { return Box::new([]); }
        let mut tex:Box<[MaybeUninit<Self>]> = Box::new_uninit_slice(n as usize);
        unsafe {
            if gl::CreateTextures::is_loaded() {
                gl::CreateTextures(T::glenum(), tex.len().try_into().unwrap(), tex[0].as_mut_ptr() as *mut GLuint);
            } else {
                gl::GenTextures(tex.len().try_into().unwrap(), tex[0].as_mut_ptr() as *mut GLuint);
                for t in tex.iter_mut() { gl::BindTexture(T::glenum(), t.assume_init_mut().id()) }
                gl::BindTexture(T::glenum(), 0);
            }
            tex.assume_init()
        }
    }

    pub fn image<F:InternalFormat,GL,I:ImageSrc>(self, gl: &GL, img: I) -> Texture<F,T> where
        T: PixelTransferTarget<F> + BaseImage,
        I::Pixels: UncompressedPixelData<F::PixelLayout>,
        GL: Supports<F::GL> + Supports<I::GL> + Supports<<I::Pixels as UncompressedPixelData<F::PixelLayout>>::GL>
    {
        let mut tex = Texture { id:self.id(), phantom:PhantomData };
        unsafe { tex.base_image_mut().image_init(gl, img.image().unlock(gl)); }
        forget(self);
        tex
    }

    pub fn compressed_image<F:SpecificCompressed,GL,I:ImageSrc>(self, gl:&GL, img:I) -> Texture<F,T> where
        T: CompressedTransferTarget<F> + BaseImage,
        I::Pixels: CompressedPixelData<Format=F>,
        GL: Supports<F::GL> + Supports<I::GL>
    {
        let mut tex = Texture { id:self.id(), phantom:PhantomData };
        unsafe { tex.base_image_mut().compressed_image_init(img.image().unlock(gl)); }
        forget(self);
        tex
    }

}
