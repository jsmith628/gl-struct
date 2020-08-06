
use super::*;

use std::mem::*;
use std::str::*;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::collections::HashSet;

unsafe fn get_integerv(name: GLenum) -> GLint {
    let mut dest = MaybeUninit::uninit();
    gl::GetIntegerv(name, dest.get_mut());
    dest.assume_init()
}

unsafe fn get_string(name: GLenum) -> &'static CStr {
    CStr::from_ptr(gl::GetString(name) as *const c_char)
}

unsafe fn get_string_i(name: GLenum, index: GLuint) -> &'static CStr {
    CStr::from_ptr(gl::GetStringi(name, index) as *const c_char)
}

fn get_major_version() -> GLuint {
    if gl::GetIntegerv::is_loaded() {
        unsafe { get_integerv(gl::MAJOR_VERSION) as GLuint }
    } else {
        0
    }
}

fn get_minor_version() -> GLuint {
    if gl::GetIntegerv::is_loaded() {
        unsafe { get_integerv(gl::MINOR_VERSION) as GLuint }
    } else {
        0
    }
}

fn get_version() -> (GLuint, GLuint) {
    (get_major_version(), get_minor_version())
}

enum ExtensionsIter {
    String(SplitWhitespace<'static>),
    Stringi(usize, usize),
}

impl Iterator for ExtensionsIter {
    type Item = &'static str;

    fn next(&mut self) -> Option<&'static str> {
        match self {
            Self::String(iter) => iter.next(),
            Self::Stringi(count, index) => {
                if index < count {
                    *index += 1;
                    unsafe {
                        Some(get_string_i(gl::EXTENSIONS, (*index-1) as GLuint).to_str().unwrap())
                    }
                } else {
                    None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::String(iter) => iter.size_hint(),
            Self::Stringi(count, index) => (count - index, Some(count - index))
        }
    }

}


fn get_extensions() -> ExtensionsIter {
    unsafe {
        if gl::GetStringi::is_loaded() {
            //for GL30 onwards, we want to use gl::GetStringi and loop through that way
            ExtensionsIter::Stringi(get_integerv(gl::NUM_EXTENSIONS).max(0) as usize, 0)
        } else if gl::GetString::is_loaded() {
            //else, we use glGetString to get a space-separated list of extensions
            ExtensionsIter::String(get_string(gl::EXTENSIONS).to_str().unwrap().split_whitespace())
        } else {
            //else, the GL isn't loaded, so we just return nothing
            ExtensionsIter::Stringi(0, 0)
        }
    }
}


#[inline]
pub unsafe fn assume_supported<GL:GLVersion>() -> GL {
    MaybeUninit::zeroed().assume_init()
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum GLVersionError {
    Version(GLuint, GLuint),
    Extension(&'static str)
}

impl ::std::fmt::Display for GLVersionError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Self::Version(maj, min) => write!(
                f,"The current OpenGL context does not support GL version {}.{}", maj, min
            ),
            Self::Extension(ex) => write!(
                f,"The current OpenGL context does not support the extension {}", ex
            ),
        }
    }
}



pub fn supported<GL:GLVersion>() -> Result<GL,GLVersionError> {

    //since the version methods take &self, we need to construct an instance
    let target: GL = unsafe { assume_supported() };

    let req_version = (target.req_major_version(), target.req_minor_version());
    let mut req_extensions = target.req_extensions();

    //if the required version is satisfied
    if get_version() >= req_version {

        //make sure that every extra required extension is supported
        if req_extensions.len() == 0 { return Ok(target); }
        for e in get_extensions() {
            req_extensions.remove(e);
            if req_extensions.len() == 0 { return Ok(target); }
        }

        Err(GLVersionError::Extension(req_extensions.into_iter().next().unwrap()))

    } else {
        Err(GLVersionError::Version(req_version.0, req_version.1))
    }

}

#[inline]
pub fn supports<Test:GLVersion+?Sized, Version:GLVersion+Sized>(
    #[allow(unused_variables)] gl: &Test
) -> bool {

    //use specialization and a helper trait to determine if Test supports Version
    trait Helper<GL> { fn _supports() -> bool; }
    impl<T:?Sized,U> Helper<U> for T { default fn _supports() -> bool {false} }
    impl<T:Supports<U>+?Sized,U:GLVersion> Helper<U> for T {
        fn _supports() -> bool {true}
    }

    <Test as Helper<Version>>::_supports()

}

pub fn upgrade_to<Test:GLVersion+?Sized, Version:GLVersion+Sized>(gl: &Test) -> Result<Version,GLVersionError> {
    if supports::<Test,Version>(gl) {
        Ok(unsafe { assume_supported() } )
    } else {
        supported::<Version>()
    }
}

pub unsafe trait GLVersion {

    fn req_major_version(&self) -> GLuint;
    fn req_minor_version(&self) -> GLuint;

    fn req_extensions(&self) -> HashSet<&'static str>;

    #[inline(always)] fn as_gl10(&self) -> GL10 {GL10 {_private:PhantomData}}

    #[inline(always)] fn try_as_gl11(&self) -> Result<GL11,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl12(&self) -> Result<GL12,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl13(&self) -> Result<GL13,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl14(&self) -> Result<GL14,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl15(&self) -> Result<GL15,GLVersionError> {upgrade_to(self)}

    #[inline(always)] fn try_as_gl20(&self) -> Result<GL20,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl21(&self) -> Result<GL21,GLVersionError> {upgrade_to(self)}

    #[inline(always)] fn try_as_gl30(&self) -> Result<GL30,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl31(&self) -> Result<GL31,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl32(&self) -> Result<GL32,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl33(&self) -> Result<GL33,GLVersionError> {upgrade_to(self)}

    #[inline(always)] fn try_as_gl40(&self) -> Result<GL40,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl41(&self) -> Result<GL41,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl42(&self) -> Result<GL42,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl43(&self) -> Result<GL43,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl44(&self) -> Result<GL44,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl45(&self) -> Result<GL45,GLVersionError> {upgrade_to(self)}
    #[inline(always)] fn try_as_gl46(&self) -> Result<GL46,GLVersionError> {upgrade_to(self)}

}

///Signifies that a given [GLVersion] object is a superset of another
#[marker] pub unsafe trait Supports<V:GLVersion>: GLVersion {}
unsafe impl<G:GLVersion> Supports<G> for G {}

///Signifies that a given [GLVersion] object supports all versions before [2.1](GL21)
pub unsafe trait GL2:
    Supports<GL10> + Supports<GL11> + Supports<GL12> + Supports<GL13> + Supports<GL14> +
    Supports<GL15> + Supports<GL20>
{
    #[inline(always)] fn as_gl11(&self) -> GL11 {GL11 {_private:PhantomData}}
    #[inline(always)] fn as_gl12(&self) -> GL12 {GL12 {_private:PhantomData}}
    #[inline(always)] fn as_gl13(&self) -> GL13 {GL13 {_private:PhantomData}}
    #[inline(always)] fn as_gl14(&self) -> GL14 {GL14 {_private:PhantomData}}
    #[inline(always)] fn as_gl15(&self) -> GL15 {GL15 {_private:PhantomData}}
}

unsafe impl<V> GL2 for V where V:
    Supports<GL10> + Supports<GL11> + Supports<GL12> + Supports<GL13> + Supports<GL14> +
    Supports<GL15> + Supports<GL20> {}

///Signifies that a given [GLVersion] object supports all versions before [3.1](GL31)
pub unsafe trait GL3: GL2 + Supports<GL21> + Supports<GL30> {
    #[inline(always)] fn as_gl20(&self) -> GL20 {GL20 {_private:PhantomData}}
    #[inline(always)] fn as_gl21(&self) -> GL21 {GL21 {_private:PhantomData}}
}

unsafe impl<V> GL3 for V where V: GL2 + Supports<GL21> + Supports<GL30> {}

///Signifies that a given [GLVersion] object supports all versions before [4.1](GL41)
pub unsafe trait GL4: GL3 + Supports<GL31> + Supports<GL32> + Supports<GL33> + Supports<GL40> {
    #[inline(always)] fn as_gl30(&self) -> GL30 {GL30 {_private:PhantomData}}
    #[inline(always)] fn as_gl31(&self) -> GL31 {GL31 {_private:PhantomData}}
    #[inline(always)] fn as_gl32(&self) -> GL32 {GL32 {_private:PhantomData}}
    #[inline(always)] fn as_gl33(&self) -> GL33 {GL33 {_private:PhantomData}}
}

unsafe impl<V> GL4 for V where V: GL3 + Supports<GL31> + Supports<GL32> + Supports<GL33> + Supports<GL40> {}

macro_rules! version_struct {
    ({$($prev:ident)*} $gl:ident $maj:tt $min:tt $str:expr, $($rest:tt)*) => {

        #[doc = "A [GLVersion] object for OpenGL version "]
        #[doc = $str]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct $gl { _private: ::std::marker::PhantomData<*const ()> }

        unsafe impl GLVersion for $gl {
            fn req_major_version(&self) -> GLuint {$maj}
            fn req_minor_version(&self) -> GLuint {$min}
            fn req_extensions(&self) -> HashSet<&'static str> { HashSet::new() }
        }

        $(unsafe impl<G:GLVersion> Supports<G> for $gl where $prev:Supports<G> {})*
        version_struct!({$gl} $($rest)*);
    };

    ({$($prev:ident)*} ) => {}
}

unsafe impl GLVersion for ! {
    fn req_major_version(&self) -> GLuint {!0}
    fn req_minor_version(&self) -> GLuint {!0}
    fn req_extensions(&self) -> HashSet<&'static str> { HashSet::new() }
}

version_struct!{ {}
    GL10 1 0 "1.0", GL11 1 1 "1.1", GL12 1 2 "1.2", GL13 1 3 "1.3", GL14 1 4 "1.4", GL15 1 5 "1.5",
    GL20 2 0 "2.0", GL21 2 1 "2.1",
    GL30 3 0 "3.0", GL31 3 1 "3.1", GL32 3 2 "3.2", GL33 3 3 "3.3",
    GL40 4 0 "4.0", GL41 4 1 "4.1", GL42 4 2 "4.2", GL43 4 3 "4.3", GL44 4 4 "4.4", GL45 4 5 "4.5", GL46 4 6 "4.6",
}

//TODO: add actual checking of if functions are loaded

impl GL10 {

    pub unsafe fn assume_loaded() -> GL10 { GL10 {_private:PhantomData}}

}
