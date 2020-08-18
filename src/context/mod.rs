
use super::*;
use std::marker::PhantomData;

use crate::version::*;
pub use self::state::*;

mod state;

///
///A struct for keeping track of global GL state while
///enforcing rust-like borrow rules on things like gl settings
///and bind points
///
pub struct GLContext<V:GLVersion> {
    pub version: V,
    // pub state: GLState<V>,
    _private: ::std::marker::PhantomData<*const ()>
}

impl GLContext<GL10> {
    pub unsafe fn load_with<F:FnMut(&str) -> *const GLvoid>(api_addr: F) -> Self {
        gl::load_with(api_addr);
        GLContext {version: GL10::assume_loaded(), _private: PhantomData}
    }
}

impl<V:GLVersion> GLContext<V> {
    pub fn upgrade_to<V2:Supports<V>>(self) -> Result<GLContext<V2>, (Self, GLError)> {
        let v2 = unsafe { ::std::mem::zeroed::<V2>() };
        if supports::<V,V2>(&self.version) {
            Ok(GLContext {version: v2, _private: PhantomData} )
        } else {
            Err((self, GLError::Version(v2.req_major_version(), v2.req_minor_version())))
        }
    }
}

pub trait ContextProvider {
    type ContextError: Debug;
    fn make_current(self) -> Result<GLContext<GL10>,Self::ContextError>;
}

#[cfg(feature = "glfw-context")]
mod glfw_impl {

    use super::*;

    impl<'a> ContextProvider for &'a mut glfw::Window {
        type ContextError = !;
        fn make_current(self) -> Result<GLContext<GL10>,!> {
            unsafe {
                glfw::Context::make_current(self);
                Ok(GLContext::load_with(|name| self.get_proc_address(name)))
            }
        }
    }

}

#[cfg(feature = "glutin-context")]
mod glutin_impl {

    use super::*;

    impl<T:glutin::ContextCurrentState> ContextProvider for glutin::Context<T> {
        type ContextError = (Self, glutin::ContextError);
        fn make_current(self) -> Result<GLContext<GL10>,Self::ContextError> {
            unsafe {
                glutin::Context::<T>::make_current(self).map(
                    |context| GLContext::load_with(|name| context.get_proc_address(name) as *const GLvoid)
                )
            }
        }
    }

}
