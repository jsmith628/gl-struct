use super::*;
use std::mem::*;
use std::ptr::*;
use std::time::*;
use std::convert::TryInto;

glenum! {
    pub enum SyncStatus {
        [AlreadySignaled ALREADY_SIGNALED "Already Signaled"],
        [TimeoutExpired TIMEOUT_EXPIRED "Timeout Expired"],
        [ConditionSatisfied CONDITION_SATISFIED "Condition Satisfied"],
        [WaitFailed WAIT_FAILED "Wait Failed"]
    }
}


pub struct Sync(GLsync);

impl Sync {

    pub fn fence(_gl: &GL32) -> Self {
        unsafe { Sync(gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0)) }
    }

    pub fn client_wait(self, flush: bool, timeout: Option<Duration>) -> SyncStatus {
        unsafe {
            let status = gl::ClientWaitSync(
                self.0,
                if flush {gl::SYNC_FLUSH_COMMANDS_BIT} else {0},
                timeout.map_or(0, |d| d.as_nanos() as GLuint64)
            );
            forget(self);
            status.try_into().unwrap()
        }
    }

    pub fn wait(self) { unsafe { gl::WaitSync(self.0, 0, gl::TIMEOUT_IGNORED); forget(self); } }
}

impl Drop for Sync {
    fn drop(&mut self) { unsafe { gl::DeleteSync(self.0) } }
}

unsafe impl Object for Sync {
    type GL = GL32;
    type Raw = GLsync;

    fn into_raw(self) -> GLsync {
        let sync = self.0;
        forget(self);
        sync
    }

    fn is(raw: GLsync) -> bool { unsafe {gl::IsSync(raw) != 0} }

    unsafe fn from_raw(raw: GLsync) -> Option<Self> {
        if Self::is(raw) { Some(Self(raw)) } else { None }
    }

    fn delete(self) { unsafe { gl::DeleteSync(self.0); forget(self); } }

    fn label(&mut self, label: &str) -> Result<(), GLError> {
        unsafe {
            if gl::ObjectPtrLabel::is_loaded() {
                let mut max_length = MaybeUninit::uninit();
                gl::GetIntegerv(gl::MAX_LABEL_LENGTH, max_length.as_mut_ptr());
                if max_length.assume_init() >= label.len() as GLint {
                    gl::ObjectPtrLabel(
                        self.0 as *mut GLvoid, label.len() as GLsizei, label.as_ptr() as *const GLchar
                    );
                    Ok(())
                } else {
                    Err(GLError::InvalidValue("object label longer than maximum allowed length".to_string()))
                }
            } else {
                Err(GLError::FunctionNotLoaded("ObjectPtrLabel"))
            }
        }
    }

    fn get_label(&self) -> Option<String> {
        unsafe {
            if gl::GetObjectPtrLabel::is_loaded() {
                //get the length of the label
                let mut length = MaybeUninit::uninit();
                gl::GetObjectPtrLabel(self.0 as *mut GLvoid, 0, length.as_mut_ptr(), null_mut());

                let length = length.assume_init();
                if length==0 { //if there is no label
                    None
                } else {
                    //allocate the space for the label
                    let mut bytes = Vec::with_capacity(length as usize);
                    bytes.set_len(length as usize);

                    //copy the label
                    gl::GetObjectPtrLabel(
                        self.0 as *mut GLvoid, length as GLsizei, null_mut(), bytes.as_mut_ptr() as *mut GLchar
                    );

                    //since label() loads from a &str, we can assume the returned bytes are valid utf8
                    Some(String::from_utf8_unchecked(bytes))
                }

            } else {
                None
            }
        }
    }

}
