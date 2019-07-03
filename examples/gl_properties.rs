
extern crate gl_struct;
extern crate glfw;

use gl_struct::*;


fn main() {

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let mut window = glfw.create_window(640, 480, "WOOOOOOOOOOOOO", glfw::WindowMode::Windowed).unwrap().0;

    glfw::Context::make_current(&mut window);
    window.set_key_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::Adaptive);

    unsafe {
        let gl_provider = GL10::load(|s| ::std::mem::transmute(glfw.get_proc_address_raw(s)));

        let mut dest = 0;

        gl::GetIntegerv(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS, &mut dest);
        println!("texture units = {}", dest);

        gl::GetIntegerv(gl::MAX_IMAGE_UNITS, &mut dest);
        println!("image units = {}", dest);

        gl::GetIntegerv(gl::MAX_UNIFORM_BUFFER_BINDINGS, &mut dest);
        println!("uniform buffer bindings = {}", dest);

        gl::GetIntegerv(gl::MAX_SHADER_STORAGE_BUFFER_BINDINGS, &mut dest);
        println!("shader storage buffer bindings = {}", dest);

        gl::GetIntegerv(gl::MAX_ATOMIC_COUNTER_BUFFER_BINDINGS, &mut dest);
        println!("atomic buffer bindings = {}", dest);
    }

}
