
use super::*;
use std::str::*;
use std::slice::*;
use std::sync::*;
use std::ptr::*;

glenum! {

    pub enum DebugSource {
        [API DEBUG_SOURCE_API "API"],
        [ShaderCompiler DEBUG_SOURCE_SHADER_COMPILER "Shader Compiler"],
        [WindowSystem DEBUG_SOURCE_WINDOW_SYSTEM "Window System"],
        [ThirdParty DEBUG_SOURCE_THIRD_PARTY "Third Party"],
        [Application DEBUG_SOURCE_APPLICATION "Application"],
        [Other DEBUG_SOURCE_OTHER "Other"]
    }

    pub enum DebugType {
        [Error DEBUG_TYPE_ERROR "Error"],
        [DeprecatedBehavior DEBUG_TYPE_DEPRECATED_BEHAVIOR "Deprecated Behavior"],
        [UndefinedBehavior DEBUG_TYPE_UNDEFINED_BEHAVIOR "Undefined Behavior"],
        [Performance DEBUG_TYPE_PERFORMANCE "Performance"],
        [Portability DEBUG_TYPE_PORTABILITY "Portability"],
        [Marker DEBUG_TYPE_MARKER "Marker"],
        [PushGroup DEBUG_TYPE_PUSH_GROUP "Push Group"],
        [PopGroup DEBUG_TYPE_POP_GROUP "Pop Group"],
        [Other DEBUG_TYPE_OTHER "Other"]
    }

    pub enum DebugSeverity {
        [High DEBUG_SEVERITY_HIGH "High"],
        [Medium DEBUG_SEVERITY_MEDIUM "Medium"],
        [Low DEBUG_SEVERITY_LOW "Low"],
        [Notification DEBUG_SEVERITY_NOTIFICATION "Notification"]
    }
}

pub(super) enum DebugCallback {
    Null,
    Asynchronous(Box<Arc<Mutex<dyn FnMut(DebugMessage) + Send + 'static>>>),
    Synchronous(Box<Box<dyn FnMut(DebugMessage) + 'static>>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DebugMessage {
    pub source: DebugSource,
    pub ty: DebugType,
    pub id: GLuint,
    pub severity: DebugSeverity,
    pub message: String
}

pub struct DebugMessageLog<'a>(&'a mut GLState<GL43>);

impl<'a> Iterator for DebugMessageLog<'a> {
    type Item = DebugMessage;
    fn next(&mut self) -> Option<DebugMessage> { self.0.get_debug_message() }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.get_debug_logged_messages() as usize, Some(self.0.get_max_debug_logged_messages() as usize))
    }
}

extern "system" fn debug_callback_sync(
    source: GLenum, ty: GLenum, id: GLuint, severity: GLenum,
    length: GLsizei, message: *const GLchar,
    user_param: *mut GLvoid
) {

    unsafe {
        let message_str = from_utf8(from_raw_parts(message as *const u8, length as usize));
        if let Ok(msg) = message_str {
            let callback = user_param as *mut Box<dyn FnMut(DebugMessage) + 'static>;

            (&mut **callback)(
                DebugMessage {
                    source: source.try_into().unwrap(),
                    ty: ty.try_into().unwrap(),
                    id: id,
                    severity: severity.try_into().unwrap(),
                    message: msg.to_string()
                }
            )
        }
    }

}

extern "system" fn debug_callback_async(
    source: GLenum, ty: GLenum, id: GLuint, severity: GLenum,
    length: GLsizei, message: *const GLchar,
    user_param: *mut GLvoid
) {
    unsafe {
        let message_str = from_utf8(from_raw_parts(message as *const u8, length as usize));
        if let Ok(msg) = message_str {
            let callback = user_param as *mut Arc<Mutex<dyn FnMut(DebugMessage) + Send + 'static>>;
            let arc = (&*callback).clone();
            let lock = arc.lock();

            if let Ok(mut c) = lock {
                (&mut *c)(
                    DebugMessage {
                        source: source.try_into().unwrap(),
                        ty: ty.try_into().unwrap(),
                        id: id,
                        severity: severity.try_into().unwrap(),
                        message: msg.to_string()
                    }
                )
            }
        }

    }

}

impl<V:Supports<GL43>+Supports<GL20>> GLState<V> {

    pub fn is_debug_output_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::DEBUG_OUTPUT)!= 0} }
    pub fn enable_debug_output(&mut self) { unsafe {gl::Enable(gl::DEBUG_OUTPUT)} }
    pub fn disable_debug_output(&mut self) { unsafe {gl::Disable(gl::DEBUG_OUTPUT)} }

    pub fn is_debug_output_synchronous_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::DEBUG_OUTPUT_SYNCHRONOUS)!= 0} }
    pub unsafe fn enable_debug_output_synchronous(&mut self) { gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS) }
    pub unsafe fn disable_debug_output_synchronous(&mut self) { gl::Disable(gl::DEBUG_OUTPUT_SYNCHRONOUS) }

    pub fn get_debug_group_stack_depth(&self) -> GLuint { unsafe { self.get_unsigned_integer(gl::DEBUG_GROUP_STACK_DEPTH) } }
    pub fn get_max_debug_group_stack_depth(&self) -> GLuint {
        unsafe { self.get_unsigned_integer(gl::MAX_DEBUG_GROUP_STACK_DEPTH) }
    }

    pub fn get_debug_logged_messages(&self) -> GLuint {unsafe{self.get_unsigned_integer(gl::DEBUG_LOGGED_MESSAGES)}}
    pub fn get_max_debug_logged_messages(&self) -> GLuint {unsafe{self.get_unsigned_integer(gl::MAX_DEBUG_LOGGED_MESSAGES)}}

    pub fn get_max_debug_message_length(&self) -> GLuint {unsafe{self.get_unsigned_integer(gl::MAX_DEBUG_MESSAGE_LENGTH)}}
    pub fn get_next_debug_logged_message_length(&self) -> GLuint {
        unsafe{self.get_unsigned_integer(gl::DEBUG_NEXT_LOGGED_MESSAGE_LENGTH)}
    }



    pub fn debug_message_control(
        &mut self,
        source: Option<DebugSource>, ty: Option<DebugType>, severity: Option<DebugSeverity>,
        ids: Option<&[GLuint]>,
        enabled: bool
    ) {
        unsafe {
            let ids: &[GLuint] = ids.map_or(&[], |i| i);
            gl::DebugMessageControl(
                source.map_or(gl::DONT_CARE, |s| s as GLenum),
                ty.map_or(gl::DONT_CARE, |s| s as GLenum),
                severity.map_or(gl::DONT_CARE, |s| s as GLenum),
                ids.len() as GLsizei, ids.as_ptr(),
                enabled as GLboolean
            );
        }
    }

    pub fn debug_message_callback_null(&mut self) {
        unsafe {
            gl::DebugMessageCallback(::std::mem::transmute(null::<GLvoid>()), null());
            self.debug_callback = DebugCallback::Null;
        }
    }

    pub fn debug_message_callback<F:FnMut(DebugMessage)+Send+'static>(&mut self, callback:F) {
        unsafe {
            let mut boxed: Box<Arc<Mutex<dyn FnMut(DebugMessage)+Send+'static>>> =
                Box::new(Arc::new(Mutex::new(callback)));
            gl::DebugMessageCallback(
                Some(debug_callback_async),
                &mut *boxed as *mut Arc<Mutex<dyn FnMut(DebugMessage)+Send+'static>> as *mut GLvoid
            );
            self.debug_callback = DebugCallback::Asynchronous(boxed);
            self.disable_debug_output_synchronous();
        }
    }

    pub fn debug_message_callback_synchronous<F:FnMut(DebugMessage)+'static>(&mut self, callback:F) {
        unsafe {
            let mut boxed: Box<Box<dyn FnMut(DebugMessage)+'static>> = Box::new(Box::new(callback));
            self.enable_debug_output_synchronous();
            gl::DebugMessageCallback(
                Some(debug_callback_sync),
                &mut *boxed as *mut Box<dyn FnMut(DebugMessage)+'static> as *mut GLvoid
            );
            self.debug_callback = DebugCallback::Synchronous(boxed);
        }
    }

    pub fn push_debug_group(&mut self, source: DebugSource, id: GLuint, message: &str) {
        unsafe {
            gl::PushDebugGroup(source as GLenum, id, message.len() as GLsizei, message.as_ptr() as *const GLchar);
        }
    }

    pub fn pop_debug_group(&mut self) { unsafe {gl::PopDebugGroup()} }

    pub fn debug_message_insert(&mut self, message: DebugMessage) {
        unsafe {
            gl::DebugMessageInsert(
                message.source as GLenum,
                message.ty as GLenum,
                message.id,
                message.severity as GLenum,
                message.message.len() as GLsizei,
                message.message.as_ptr() as *const GLchar
            );
        }
    }

    pub fn get_debug_message_log(&mut self) -> DebugMessageLog {
        unsafe { DebugMessageLog(transmute(self)) }
    }

    pub fn get_debug_message(&mut self) -> Option<DebugMessage> {
        let mut source = MaybeUninit::uninit();
        let mut ty = MaybeUninit::uninit();
        let mut id = MaybeUninit::uninit();
        let mut severity = MaybeUninit::uninit();
        let mut length = MaybeUninit::uninit();
        let mut message = Vec::<u8>::with_capacity(self.get_next_debug_logged_message_length() as usize);

        unsafe {

            let count = gl::GetDebugMessageLog(
                1, message.capacity() as GLsizei,
                source.as_mut_ptr(), ty.as_mut_ptr(), id.as_mut_ptr(), severity.as_mut_ptr(),
                length.as_mut_ptr(), message.as_mut_ptr() as *mut GLchar
            );

            if count>0 {
                message.set_len(length.assume_init() as usize);
                Some(
                    DebugMessage{
                        source: source.assume_init().try_into().unwrap(),
                        ty: ty.assume_init().try_into().unwrap(),
                        id: id.assume_init().try_into().unwrap(),
                        severity: severity.assume_init().try_into().unwrap(),
                        message: String::from_utf8(message).unwrap(),
                    }
                )
            } else {
                None
            }

        }

    }


}
