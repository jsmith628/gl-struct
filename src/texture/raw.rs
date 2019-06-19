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
            unsafe impl Target for $name {
                type Resource = RawTex<Self>;
                #[inline] unsafe fn bind(self, id: GLuint) { gl::BindTexture(self.into(), id); }
            }
        )*
    }
}

pub unsafe trait TextureTarget: GLEnum + Default + Target<Resource=RawTex<Self>> {
    type GL: GLProvider;
    type Dim: TexDim;

    #[inline] fn glenum() -> GLenum {Self::default().into()}
    #[inline] unsafe fn binding_location() -> BindingLocation<RawTex<Self>> {Self::default().as_loc()}

    #[inline]
    fn multisample() -> bool {
        match Self::glenum() {
            gl::TEXTURE_2D_MULTISAMPLE | gl::TEXTURE_2D_MULTISAMPLE_ARRAY => true,
            _ => false
        }
    }

}

target! {
    [TEXTURE_1D "Texture 1D"]; GL1; [usize;1],
    [TEXTURE_2D "Texture 2D"]; GL1; [usize;2],
    [TEXTURE_3D "Texture 3D"]; GL1; [usize;3],
    [TEXTURE_1D_ARRAY "Texture 1D Array"]; GL3; (<TEXTURE_1D as TextureTarget>::Dim, usize),
    [TEXTURE_2D_ARRAY "Texture 2D Array"]; GL3; (<TEXTURE_2D as TextureTarget>::Dim, usize),
    [TEXTURE_RECTANGLE "Texture Rectangle"]; GL3; <TEXTURE_2D as TextureTarget>::Dim,
    [TEXTURE_BUFFER "Texture Buffer"]; GL3; usize,
    [TEXTURE_CUBE_MAP "Texture Cube Map"]; GL1; <TEXTURE_2D as TextureTarget>::Dim,
    [TEXTURE_CUBE_MAP_ARRAY "Texture Cube Map Array"]; GL4; <TEXTURE_2D_ARRAY as TextureTarget>::Dim,
    [TEXTURE_2D_MULTISAMPLE "Texture 2D Multisample"]; GL3; <TEXTURE_2D as TextureTarget>::Dim,
    [TEXTURE_2D_MULTISAMPLE_ARRAY "Texture 2D Multisample Array"]; GL3; <TEXTURE_2D_ARRAY as TextureTarget>::Dim
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

unsafe impl<T: TextureTarget> Resource for RawTex<T> {
    type GL = T::GL;
    type BindingTarget = T;

    #[inline(always)] fn id(&self) -> GLuint {self.0}
    #[inline(always)] fn into_raw(self) -> GLuint {self.0}

    #[inline] fn gen(_gl: &Self::GL) -> Self {
        let mut raw = Self(0, PhantomData);
        unsafe { gl::GenTextures(1, &mut raw.0 as *mut GLuint); }
        raw
    }

    #[inline] fn gen_resources(_gl: &Self::GL, count: GLuint) -> Box<[Self]> {
        let mut raw = Vec::with_capacity(count as usize);
        unsafe {
            if count > 0 {
                raw.set_len(count as usize);
                gl::GenTextures(count as GLsizei, &mut raw[0] as *mut GLuint);
            }
            ::std::mem::transmute(raw.into_boxed_slice())
        }
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

    #[inline(always)] unsafe fn from_raw(id:GLuint) -> Option<Self> {
        if Self::is(id) { Some(RawTex(id, PhantomData)) } else { None }
    }

    #[inline(always)] fn delete(self) { drop(self) }

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

pub unsafe trait TexDim: Sized + Copy + Eq + Hash + Debug {
    fn dim() -> usize;
    fn minimized(&self, level: GLuint) -> Self;

    #[inline] fn pixels(&self) -> usize {self.width() * self.height() * self.depth()}
    #[inline] fn max_levels(&self) -> GLuint {
        (0 as GLuint).leading_zeros() - (self.width().max(self.height().max(self.depth()))).leading_zeros()
    }

    #[inline] fn width(&self) -> usize {1}
    #[inline] fn height(&self) -> usize {1}
    #[inline] fn depth(&self) -> usize {1}

}

unsafe impl TexDim for usize {
    #[inline] fn dim() -> usize {1}
    #[inline] fn width(&self) -> usize {*self}
    #[inline] fn minimized(&self, level: GLuint) -> Self { (self >> level).max(1) }
}

unsafe impl TexDim for [usize;1] {
    #[inline] fn dim() -> usize {1}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn minimized(&self, level: GLuint) -> Self { [self[0].minimized(level)] }
}

unsafe impl TexDim for [usize;2] {
    #[inline] fn dim() -> usize {2}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn height(&self) -> usize {self[1]}
    #[inline] fn minimized(&self, level: GLuint) -> Self {
        [self[0].minimized(level), self[1].minimized(level)]
    }
}

unsafe impl TexDim for [usize;3] {
    #[inline] fn dim() -> usize {3}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn height(&self) -> usize {self[1]}
    #[inline] fn depth(&self) -> usize {self[2]}
    #[inline] fn minimized(&self, level: GLuint) -> Self {
        [self[0].minimized(level), self[1].minimized(level), self[2].minimized(level)]
    }
}

unsafe impl TexDim for ([usize;1], usize) {
    #[inline] fn dim() -> usize {2}
    #[inline] fn minimized(&self, level: GLuint) -> Self {(self.0.minimized(level), self.1)}
    #[inline] fn max_levels(&self) -> GLuint {self.0.max_levels()}

    #[inline] fn width(&self) -> usize {self.0[0]}
    #[inline] fn height(&self) -> usize {self.1}
}

unsafe impl TexDim for ([usize;2], usize) {
    #[inline] fn dim() -> usize {3}
    #[inline] fn minimized(&self, level: GLuint) -> Self {(self.0.minimized(level), self.1)}
    #[inline] fn max_levels(&self) -> GLuint {self.0.max_levels()}

    #[inline] fn width(&self) -> usize {self.0[0]}
    #[inline] fn height(&self) -> usize {self.0[1]}
    #[inline] fn depth(&self) -> usize {self.1}
}
