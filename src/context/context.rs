use super::*;

///
///A struct for keeping track of global GL state while
///enforcing rust-like borrow rules on things like gl settings
///and bind points
///
pub struct GLContext<V:GLVersion> {
    pub version: V,
    // pub state: GLState<V>,
    _private: ()
}

impl GLContext<GL10> {
    pub unsafe fn load_with<F:Fn(&str) -> *const GLvoid>(api_addr: F) -> Self {
        gl::load_with(api_addr);
        GLContext {version: GL10::assume_loaded(), _private: ()}
    }
}

impl<V:GLVersion> GLContext<V> {
    pub fn upgrade_to<V2:GLVersion>(self) -> Result<GLContext<V2>, (Self, GLError)> {
        let v2 = unsafe { ::std::mem::zeroed::<V2>() };
        if supports::<V,V2>(&self.version) {
            return Ok(GLContext {version: v2, _private: ()} );
        } else {
            return Err((self, GLError::Version(v2.major_version(), v2.minor_version())));
        }
    }
}

impl<V:GLVersion> !Send for GLContext<V> {}
impl<V:GLVersion> !Sync for GLContext<V> {}

pub trait ContextProvider {
    type ContextError: Debug;
    fn make_current(self) -> Result<GLContext<GL10>,Self::ContextError>;
}

#[cfg(feature = "glfw_context")]
mod glfw_impl {

    use super::*;

    impl<'a> ContextProvider for &'a mut glfw::Window {
        type ContextError = !;
        fn make_current(self) -> Result<GLContext<GL10>,!> {
            unsafe {
                glfw::Context::make_current(self);
                Some(GLContext::load_with(|name| self.get_proc_address(name)))
            }
        }
    }

}

#[cfg(feature = "glutin_context")]
mod glutin_impl {

    use super::*;

    impl<T:glutin::ContextCurrentState> ContextProvider for glutin::Context<T> {
        type ContextError = (Self, glutin::ContextError);
        fn make_current(self) -> Result<GLContext<GL10>,Self::ContextError> {
            unsafe {
                glutin::Context::<T>::make_current(self).map(
                    |context| GLContext::load_with(|name| context.get_proc_address(name) as *const GLvoid)
                );
            }
        }
    }

}
