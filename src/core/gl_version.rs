
use super::*;

use std::mem::MaybeUninit;

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
        let mut dest = MaybeUninit::uninit();
        gl::GetIntegerv(param, dest.get_mut());
        dest.assume_init()
    }
}

#[inline] pub fn supports<Test:GLVersion+?Sized, Version:GLVersion+Sized>(gl: &Test) -> bool {
    let target: Version = unsafe { ::std::mem::zeroed() };
    let version = (target.major_version(), target.minor_version());
    (gl.major_version(), gl.minor_version()) <= version ||
    (gl.get_major_version(), gl.get_minor_version()) <= version
}

fn upgrade_to<Test:GLVersion+?Sized, Version:GLVersion+Sized>(gl: &Test) -> Result<Version,GLError> {
    let target: Version = unsafe { ::std::mem::zeroed() };
    if supports::<Test,Version>(gl){
        Ok(target)
    } else {
        Err(GLError::Version(target.major_version(), target.minor_version()))
    }
}

///
///A trait representing the currently loaded openGL version
///
///The purpose of this is two-fold:
/// * To provide an object that can "own" the functions loaded by [`gl::load_with()`]:<p>
///     <i>As it stands currently, by default, all of the GL calls in [`gl-rs`](crate::gl) will panic unless loaded
///     with a function pointer to the driver functions. However, by having an object be created when
///     those functions are loaded and making it a requirement for instantiating the OpenGL object
///     structs, we can guarrantee that those panics will not occur (and at compile-time with near 0 cost) </i>
/// * To provide an object that can enscapulate openGL versioning: <p>
///     <i>Even with a guarrantee that [`gl::load_with()`] has been called, there is still no guarrantee
///     that the driver implements or the hardware supports any given version of OpenGL (or that the
///     functions were even loaded properly!). Thus, by having an object be created on version checking
///     and requiring it for creating objects using that version, we can guarrantee that the GL version
///     is available (also at compile time at near 0 cost)</i>
///
/// ## Usage
///
///Every openGL object in this crate will require a reference to some object that implements this
///trait. In order to obtain this object, two things are required:
/// * Loading the function pointers to obtain a [`GL10`] object:<p>
///   This can be done by passing a fuction loader from any compatible context-creation library
///   to [`GL10::load()`], as per [`gl::load_with()`] from [`gl-rs`](crate::gl). However, since this is
///   fundamentally unsafe, it is highly recommended that such aforementioned crates implemement a
///   safe function to do this automatically and return the `GL10` object for the end user to use
/// * "Upgrading" that `GL10` object to the appropriate version:<p>
///   This is done with the corresponding `try_as_*` functions in this trait and any necessary error
///   handling in the case of the version not being supported.
///
pub unsafe trait GLVersion {

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

///Signifies that a given [GLVersion] object is a superset of another
pub unsafe trait Supports<V:GLVersion>: GLVersion{}
unsafe impl<G:GLVersion> Supports<G> for G {}

///Signifies that a given [GLVersion] object supports all versions before [2.1](GL21)
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

unsafe impl<V> GL2 for V where V:
    Supports<GL10> + Supports<GL11> + Supports<GL12> + Supports<GL13> + Supports<GL14> +
    Supports<GL15> + Supports<GL20> {}

///Signifies that a given [GLVersion] object supports all versions before [3.1](GL31)
pub unsafe trait GL3: GL2 + Supports<GL21> + Supports<GL30> {
    #[inline(always)] fn as_gl20(&self) -> GL20 {GL20 {_private:()}}
    #[inline(always)] fn as_gl21(&self) -> GL21 {GL21 {_private:()}}
}

unsafe impl<V> GL3 for V where V: GL2 + Supports<GL21> + Supports<GL30> {}

///Signifies that a given [GLVersion] object supports all versions before [4.1](GL41)
pub unsafe trait GL4: GL3 + Supports<GL31> + Supports<GL32> + Supports<GL33> + Supports<GL40> {
    #[inline(always)] fn as_gl30(&self) -> GL30 {GL30 {_private:()}}
    #[inline(always)] fn as_gl31(&self) -> GL31 {GL31 {_private:()}}
    #[inline(always)] fn as_gl32(&self) -> GL32 {GL32 {_private:()}}
    #[inline(always)] fn as_gl33(&self) -> GL33 {GL33 {_private:()}}
}

unsafe impl<V> GL4 for V where V: GL3 + Supports<GL31> + Supports<GL32> + Supports<GL33> + Supports<GL40> {}

macro_rules! version_struct {
    ({$($prev:ident)*} $gl:ident $maj:tt $min:tt $str:expr, $($rest:tt)*) => {

        #[doc = "A [GLVersion] object for OpenGL version "]
        #[doc = $str]
        #[derive(Clone, PartialEq, Eq, Hash, Debug)] pub struct $gl { _private: () }
        unsafe impl GLVersion for $gl {
            #[inline(always)] fn major_version(&self) -> GLuint {$maj}
            #[inline(always)] fn minor_version(&self) -> GLuint {$min}
        }

        $(unsafe impl Supports<$prev> for $gl {})*
        version_struct!({$($prev)* $gl} $($rest)*);
    };

    ({$($prev:ident)*} ) => {}
}

version_struct!{ {}
    GL10 1 0 "1.0", GL11 1 1 "1.1", GL12 1 2 "1.2", GL13 1 3 "1.3", GL14 1 4 "1.4", GL15 1 5 "1.5",
    GL20 2 0 "2.0", GL21 2 1 "2.1",
    GL30 3 0 "3.0", GL31 3 1 "3.1", GL32 3 2 "3.2", GL33 3 3 "3.3",
    GL40 4 0 "4.0", GL41 4 1 "4.1", GL42 4 2 "4.2", GL43 4 3 "4.3", GL44 4 4 "4.4", GL45 4 5 "4.5", GL46 4 6 "4.6",
}

//TODO: add actual checking of if functions are loaded

impl GL10 {

    pub unsafe fn assume_loaded() -> GL10 { GL10 {_private:()}}

}
