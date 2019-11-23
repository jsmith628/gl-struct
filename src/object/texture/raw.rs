use super::*;

macro_rules! target {
    ($([$name:ident $display:expr]; $GL:ty; $dim:ty),*) => {
        $(
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
            pub struct $name;

            display_from_debug!($name);
            impl std::fmt::Debug for $name {
                #[inline] fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, $display)
                }
            }

            impl From<$name> for GLenum { #[inline(always)] fn from(_:$name) -> GLenum {gl::$name} }
            impl TryFrom<GLenum> for $name {
                type Error = GLError;
                #[inline(always)] fn try_from(val:GLenum) -> Result<Self,GLError> {
                    if val == $name.into() { Ok($name) } else {Err(GLError::InvalidEnum(val,"".to_string()))}
                }
            }
            impl GLEnum for $name {}

            unsafe impl TextureTarget for $name { type GL = $GL; type Dim = $dim; }
            impl Target<RawTex<Self>> for $name {
                #[inline] fn target_id(self) -> GLenum { self.into() }
                #[inline] unsafe fn bind(self, tex:&RawTex<Self>) { gl::BindTexture(self.into(), tex.id()); }
                #[inline] unsafe fn unbind(self) { gl::BindTexture(self.into(), 0); }
            }
        )*
    }
}

pub unsafe trait TextureTarget: GLEnum + Default + Target<RawTex<Self>> {
    type GL: GLVersion;
    type Dim: TexDim;

    #[inline] fn glenum() -> GLenum {Self::default().into()}
    #[inline] unsafe fn binding_location() -> BindingLocation<RawTex<Self>,Self> {Self::default().as_loc()}

    #[inline]
    fn multisampled() -> bool {
        match Self::glenum() {
            gl::TEXTURE_2D_MULTISAMPLE | gl::TEXTURE_2D_MULTISAMPLE_ARRAY => true,
            _ => false
        }
    }

    #[inline]
    fn mipmapped() -> bool {
        match Self::glenum() {
            gl::TEXTURE_2D_MULTISAMPLE | gl::TEXTURE_2D_MULTISAMPLE_ARRAY => false,
            gl::TEXTURE_RECTANGLE | gl::TEXTURE_BUFFER => false,
            _ => true
        }
    }

    #[inline]
    fn cube_mapped() -> bool {
        match Self::glenum() {
            gl::TEXTURE_CUBE_MAP | gl::TEXTURE_CUBE_MAP_ARRAY => true,
            _ => false
        }
    }

}

target! {
    [TEXTURE_1D "Texture 1D"]; GL10; [usize;1],
    [TEXTURE_2D "Texture 2D"]; GL10; [usize;2],
    [TEXTURE_3D "Texture 3D"]; GL11; [usize;3],
    [TEXTURE_1D_ARRAY "Texture 1D Array"]; GL30; (<TEXTURE_1D as TextureTarget>::Dim, usize),
    [TEXTURE_2D_ARRAY "Texture 2D Array"]; GL30; (<TEXTURE_2D as TextureTarget>::Dim, usize),
    [TEXTURE_RECTANGLE "Texture Rectangle"]; GL31; <TEXTURE_2D as TextureTarget>::Dim,
    [TEXTURE_BUFFER "Texture Buffer"]; GL31; usize,
    [TEXTURE_CUBE_MAP "Texture Cube Map"]; GL13; <TEXTURE_2D as TextureTarget>::Dim,
    [TEXTURE_CUBE_MAP_ARRAY "Texture Cube Map Array"]; GL40; <TEXTURE_2D_ARRAY as TextureTarget>::Dim,
    [TEXTURE_2D_MULTISAMPLE "Texture 2D Multisample"]; GL32; <TEXTURE_2D as TextureTarget>::Dim,
    [TEXTURE_2D_MULTISAMPLE_ARRAY "Texture 2D Multisample Array"]; GL32; <TEXTURE_2D_ARRAY as TextureTarget>::Dim
}

pub struct RawTex<T: TextureTarget>(GLuint, PhantomData<T>);

pub type RawTex1D                 = RawTex<TEXTURE_1D>;
pub type RawTex2D                 = RawTex<TEXTURE_2D>;
pub type RawTex3D                 = RawTex<TEXTURE_3D>;
pub type RawTex1DArray            = RawTex<TEXTURE_1D_ARRAY>;
pub type RawTex2DArray            = RawTex<TEXTURE_2D_ARRAY>;
pub type RawTexRectangle          = RawTex<TEXTURE_RECTANGLE>;
pub type RawTexBuffer             = RawTex<TEXTURE_BUFFER>;
pub type RawTexCubeMap            = RawTex<TEXTURE_CUBE_MAP>;
pub type RawTexCubeMapArray       = RawTex<TEXTURE_CUBE_MAP_ARRAY>;
pub type RawTex2DMultisample      = RawTex<TEXTURE_2D_MULTISAMPLE>;
pub type RawTex2DMultisampleArray = RawTex<TEXTURE_2D_MULTISAMPLE_ARRAY>;

unsafe impl<T: TextureTarget> Object for RawTex<T> {
    type GL = T::GL;
    type Raw = GLuint;

    #[inline(always)] fn into_raw(self) -> GLuint {self.0}

    #[inline(always)] unsafe fn from_raw(id:GLuint) -> Option<Self> {
        if Self::is(id) { Some(RawTex(id, PhantomData)) } else { None }
    }

    fn is(id: GLuint) -> bool {
        unsafe {
            //check if it is even a texture
            if gl::IsTexture(id)!=0 {
                //now, check if it is of the particular type
                //TODO: implement
                true
            } else {
                false
            }
        }
    }

    #[inline(always)] fn delete(self) { drop(self) }

    #[inline] fn label(&mut self, label: &str) -> Result<(),GLError> { object::object_label(self, label) }
    #[inline] fn get_label(&self) -> Option<String> { object::get_object_label(self) }


}


unsafe impl<T: TextureTarget> Resource for RawTex<T> {
    type BindingTarget = T;

    const IDENTIFIER: ResourceIdentifier = ResourceIdentifier::Texture;

    #[inline(always)] fn id(&self) -> GLuint {self.0}

    #[inline] fn gen(_gl: &<Self as Object>::GL) -> Self {
        let mut raw = Self(0, PhantomData);
        unsafe { gl::GenTextures(1, &mut raw.0 as *mut GLuint); }
        raw
    }

    #[inline] fn gen_resources(_gl: &<Self as Object>::GL, count: GLuint) -> Box<[Self]> {
        let mut raw = Vec::with_capacity(count as usize);
        unsafe {
            if count > 0 {
                raw.set_len(count as usize);
                gl::GenTextures(count as GLsizei, &mut raw[0] as *mut GLuint);
            }
            ::std::mem::transmute(raw.into_boxed_slice())
        }
    }

    #[inline]
    fn delete_resources(resources: Box<[Self]>) {
        unsafe {
            //the transmutation makes sure that we don't double-free
            let ids = ::std::mem::transmute::<Box<[Self]>, Box<[gl::types::GLuint]>>(resources);
            if ids.len()>0 {
                gl::DeleteTextures(ids.len() as gl::types::GLsizei, &ids[0] as *const gl::types::GLuint);
            }
        }
    }
}

impl<T: TextureTarget> Drop for RawTex<T> {
    #[inline] fn drop(&mut self) { unsafe { gl::DeleteTextures(1, self.0 as *mut GLuint) } }
}

impl<T: TextureTarget> !Send for RawTex<T> {}
impl<T: TextureTarget> !Sync for RawTex<T> {}
