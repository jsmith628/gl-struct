use super::*;


///
///A struct for keeping track of global GL state while
///enforcing rust-like borrow rules on things like gl settings
///and bind points
///
pub struct GLContext {

}

impl GLContext {
    pub unsafe fn load_with<F:Fn(&str) -> *const GLvoid>(api_addr: F) -> (GL10,Self) {
        gl::load_with(api_addr);
        (GL10::assume_loaded(), GLContext {})
    }
}

impl !Send for GLContext {}
impl !Sync for GLContext {}

pub trait ContextProvider {
    type ContextError: Debug;
    fn make_current(self) -> Result<(GL10,GLContext),Self::ContextError>;
}

#[cfg(feature = "glfw_context")]
mod glfw_impl {

    use super::*;

    impl<'a> ContextProvider for &'a mut glfw::Window {
        type ContextError = !;
        fn make_current(self) -> Result<(GL10,GLContext),!> {
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
        fn make_current(self) -> Result<(GL10,GLContext),Self::ContextError> {
            unsafe {
                glutin::Context::<T>::make_current(self).map(
                    |context| GLContext::load_with(|name| context.get_proc_address(name) as *const GLvoid)
                );
            }
        }
    }

}
