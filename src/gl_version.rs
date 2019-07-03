
use super::*;

// macro_rules! check_loaded {
//     ($gl_fun0:ident, $($gl_fun:ident),+; $expr:expr) => {
//         check_loaded!($gl_fun0; check_loaded!($($gl_fun),+; $expr)).map_or_else(|e| Err(e), |ok| ok)
//     };
//
//     ($gl_fun:ident; $expr:expr) => {
//         if $crate::gl::$gl_fun::is_loaded() {
//             Ok($expr)
//         } else {
//             Err($crate::GLError::FunctionNotLoaded(concat!("gl", stringify!($gl_fun))))
//         }
//     }
// }

fn get_integerv(param: GLenum) -> GLint {
    unsafe {
        let mut dest = ::std::mem::uninitialized();
        gl::GetIntegerv(param, &mut dest);
        dest
    }
}

#[inline(always)] fn upgrade_to<Test:GL+?Sized, Version:GL+Sized>(gl: &Test) -> Result<Version,GLError> {
    let target: Version = unsafe { ::std::mem::zeroed() };
    let version = (target.major_version(), target.minor_version());
    if
        (gl.major_version(), gl.minor_version()) <= version ||
        (gl.get_major_version(), gl.get_minor_version()) <= version
    {
        Ok(target)
    } else {
        Err(GLError::Version(version.0, version.1))
    }
}

pub unsafe trait GL {

    fn major_version(&self) -> GLuint;
    fn minor_version(&self) -> GLuint;

    #[inline] fn get_major_version(&self) -> GLuint { get_integerv(gl::MAJOR_VERSION) as GLuint }
    #[inline] fn get_minor_version(&self) -> GLuint { get_integerv(gl::MINOR_VERSION) as GLuint }

    #[inline(always)] fn as_gl10(&self) -> GL10 {GL10 {_private:()}}

    #[inline(always)] fn try_as_gl11(&self) -> Result<GL11,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl12(&self) -> Result<GL12,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl13(&self) -> Result<GL13,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl14(&self) -> Result<GL14,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl15(&self) -> Result<GL15,GLError> {upgrade_to(self)}

    #[inline(always)] fn try_as_gl20(&self) -> Result<GL20,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl21(&self) -> Result<GL21,GLError> {upgrade_to(self)}

    #[inline(always)] fn try_as_gl30(&self) -> Result<GL30,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl31(&self) -> Result<GL31,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl32(&self) -> Result<GL32,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl33(&self) -> Result<GL33,GLError> {upgrade_to(self)}

    #[inline(always)] fn try_as_gl40(&self) -> Result<GL40,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl41(&self) -> Result<GL41,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl42(&self) -> Result<GL42,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl43(&self) -> Result<GL43,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl44(&self) -> Result<GL44,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl45(&self) -> Result<GL45,GLError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl46(&self) -> Result<GL46,GLError> {upgrade_to(self)}

}

pub unsafe trait Supports<V:GL>: GL {}
unsafe impl<G:GL> Supports<G> for G {}

pub unsafe trait GL2:
    Supports<GL10> + Supports<GL11> + Supports<GL12> + Supports<GL13> + Supports<GL14> +
    Supports<GL15> + Supports<GL20>
{
    #[inline(always)] fn as_gl11(&self) -> GL11 {GL11 {_private:()}}
    #[inline(always)] fn as_gl12(&self) -> GL12 {GL12 {_private:()}}
    #[inline(always)] fn as_gl13(&self) -> GL13 {GL13 {_private:()}}
    #[inline(always)] fn as_gl14(&self) -> GL14 {GL14 {_private:()}}
    #[inline(always)] fn as_gl15(&self) -> GL15 {GL15 {_private:()}}
}

pub unsafe trait GL3: GL2 + Supports<GL21> + Supports<GL30> {
    #[inline(always)] fn as_gl20(&self) -> GL20 {GL20 {_private:()}}
    #[inline(always)] fn as_gl21(&self) -> GL21 {GL21 {_private:()}}
}

pub unsafe trait GL4: GL3 + Supports<GL31> + Supports<GL32> + Supports<GL33> + Supports<GL40> {
    #[inline(always)] fn as_gl30(&self) -> GL30 {GL30 {_private:()}}
    #[inline(always)] fn as_gl31(&self) -> GL31 {GL31 {_private:()}}
    #[inline(always)] fn as_gl32(&self) -> GL32 {GL32 {_private:()}}
    #[inline(always)] fn as_gl33(&self) -> GL33 {GL33 {_private:()}}
}

macro_rules! version_struct {
    (@major $gl:ident 1) => {};
    (@major $gl:ident 2) => { unsafe impl GL2 for $gl {} version_struct!(@major $gl 1); };
    (@major $gl:ident 3) => { unsafe impl GL3 for $gl {} version_struct!(@major $gl 2); };
    (@major $gl:ident 4) => { unsafe impl GL4 for $gl {} version_struct!(@major $gl 3); };

    ({$($prev:ident)*} $gl:ident $maj:tt $min:tt , $($rest:tt)*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct $gl { _private: () }
        unsafe impl GL for $gl {
            #[inline(always)] fn major_version(&self) -> GLuint {$maj}
            #[inline(always)] fn minor_version(&self) -> GLuint {$min}
        }

        $(unsafe impl Supports<$prev> for $gl {})*

        version_struct!(@major $gl $maj);

        version_struct!({$($prev)* $gl} $($rest)*);
    };

    ({$($prev:ident)*} ) => {}
}

version_struct!{ {}
    GL10 1 0, GL11 1 1, GL12 1 2, GL13 1 3, GL14 1 4, GL15 1 5,
    GL20 2 0, GL21 2 1,
    GL30 3 0, GL31 3 1, GL32 3 2, GL33 3 3,
    GL40 4 0, GL41 4 1, GL42 4 2, GL43 4 3, GL44 4 4, GL45 4 5, GL46 4 6,
}

//TODO: add actual checking of if functions are loaded

impl GL10 {

    pub fn get_current() -> Result<Self, ()> {
        //if glFinish isn't loaded, we can pretty safely assume nothing has
        if gl::Finish::is_loaded() {
            Ok(GL10{ _private: () })
        } else {
            Err(())
        }
    }

    pub unsafe fn load<F: FnMut(&'static str) -> *const GLvoid>(proc_addr: F) -> GL10 {
        gl::load_with(proc_addr);
        GL10{ _private: () }
    }

}
