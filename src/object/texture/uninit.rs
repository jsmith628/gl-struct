use super::*;

pub type UninitTex<T> = Texture<!,T>;

impl<T:TextureType> UninitTex<T> {
    pub fn gen(#[allow(unused_variables)] gl: &T::GL) -> Self {
        let mut tex = MaybeUninit::uninit();
        unsafe {
            gl::GenTextures(1, tex.as_mut_ptr() as *mut GLuint);
            tex.assume_init()
        }
    }

    pub fn gen_textures(#[allow(unused_variables)] gl: &T::GL, count: GLuint) -> Box<[Self]> {
        if count==0 { return Box::new([]); }
        let mut tex:Box<[MaybeUninit<Self>]> = Box::new_uninit_slice(count as usize);
        unsafe {
            gl::GenTextures(tex.len() as GLsizei, tex[0].as_mut_ptr() as *mut GLuint);
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

    pub fn create_textures(#[allow(unused_variables)] gl: &T::GL, count: GLuint) -> Box<[Self]> {
        if count==0 { return Box::new([]); }
        let mut tex:Box<[MaybeUninit<Self>]> = Box::new_uninit_slice(count as usize);
        unsafe {
            if gl::CreateTextures::is_loaded() {
                gl::CreateTextures(T::glenum(), tex.len() as GLsizei, tex[0].as_mut_ptr() as *mut GLuint);
            } else {
                gl::GenTextures(tex.len() as GLsizei, tex[0].as_mut_ptr() as *mut GLuint);
                for t in tex.iter_mut() { gl::BindTexture(T::glenum(), t.get_mut().id()) }
                gl::BindTexture(T::glenum(), 0);
            }
            tex.assume_init()
        }
    }



}
