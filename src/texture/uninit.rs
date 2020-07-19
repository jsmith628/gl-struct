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
            gl::GenTextures(tex.len().try_into().unwrap(), MaybeUninit::first_ptr_mut(&mut *tex));
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
                gl::BindTexture(T::glenum(), tex.get_mut().id());
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
                for t in tex.iter_mut() { gl::BindTexture(T::glenum(), t.get_mut().id()) }
                gl::BindTexture(T::glenum(), 0);
            }
            tex.assume_init()
        }
    }

    #[allow(unused_variables)]
    pub fn image<F,I>(self, gl: &F::GL, data: I) -> Texture<F,T> where
        F: InternalFormat,
        I: TexImageSrc<F>,
        T: PixelTransferTarget<F> + BaseImage
    {
        let mut tex = Texture { id:self.id(), phantom:PhantomData };
        unsafe { tex.base_image_mut().image_unchecked(data); }
        forget(self);
        tex
    }

    #[allow(unused_variables)]
    pub fn compressed_image<F,I>(self, gl: &F::GL, data: I) -> Texture<F,T> where
        F: SpecificCompressed,
        I: CompressedImageSrc<Format=F>,
        T: CompressedTransferTarget<F> + BaseImage
    {
        let mut tex = Texture { id:self.id(), phantom:PhantomData };
        unsafe { tex.base_image_mut().compressed_image_unchecked(data); }
        forget(self);
        tex
    }

}
