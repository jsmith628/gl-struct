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

    pub fn image<F,P>(self, dim: T::Dim, data: &P) -> Texture<F,T> where
        F:InternalFormat,
        P:PixelData<F::ClientFormat>,
        T:ImageTarget<F>
    {
        unsafe {

            size_check::<_,F,_>(dim, data);
            apply_unpacking_settings(data);

            let mut target = BufferTarget::PixelUnpackBuffer.as_loc();
            let pixels = data.pixels();
            let (binding, ptr, client_format) = match pixels {
                Pixels::Slice(f, slice) => (None, slice as *const [P::Pixel] as *const GLvoid, f),
                Pixels::Buffer(f, ref slice) => (Some(target.bind(slice)), slice.offset() as *const GLvoid, f),
            };

            let (format, ty) = client_format.format_type().into();
            let (internal, format, ty) = (F::glenum() as GLint, format.into(), ty.into());
            let (w, h, d) = (dim.width() as GLsizei, dim.height() as GLsizei, dim.depth() as GLsizei);

            T::bind_loc::<!>().map_bind(&self,
                |b| match T::Dim::dim() {
                    1 => gl::TexImage1D(b.target_id(), 0, internal, w, 0, format, ty, ptr),
                    2 => gl::TexImage2D(b.target_id(), 0, internal, w, h, 0, format, ty, ptr),
                    3 => gl::TexImage3D(b.target_id(), 0, internal, w, h, d, 0, format, ty, ptr),
                    n => panic!("{}D Textures not supported", n)
                }
            );

            drop(binding);

            let id = self.id();
            forget(self);

            Texture { id:id, phantom:PhantomData }

        }
    }

}
