#![feature(new_uninit)]

extern crate gl;
extern crate gl_struct;
extern crate glfw;
#[macro_use] extern crate glsl_to_spirv_macros;
#[macro_use] extern crate glsl_to_spirv_macros_impl;

use gl_struct::*;
use std::ptr::*;
use std::mem::*;

fn main() {

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let mut window = glfw.create_window(640, 480, "WOOOOOOOOOOOOO", glfw::WindowMode::Windowed).unwrap().0;

    glfw::Context::make_current(&mut window);
    window.set_key_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::Adaptive);

    let gl_provider = unsafe {
        GLProvider::load(|s| ::std::mem::transmute(glfw.get_proc_address_raw(s)))
    };
    let _context = Context::init(&gl_provider);

    unsafe {

        let src = r"
            #version 460
            #line 45

            uniform layout(binding=1) sampler3D a, b;


            void main() {

            }

        ";

        let print_log = |p:u32, get_iv: unsafe fn(u32,u32,*mut i32), get_log: unsafe fn(u32,i32,*mut i32,*mut i8)| {
            let mut len = MaybeUninit::uninit();
            get_iv(p, gl::INFO_LOG_LENGTH, len.as_mut_ptr());
            let len = len.assume_init();

            println!("{}", len);

            let mut log = Box::<[u8]>::new_uninit_slice(len as usize);
            if len!=0 { get_log(p, len, null_mut(), log[0].as_mut_ptr() as *mut i8); }
            let log = String::from_raw_parts(Box::into_raw(log) as *mut _, len as usize, len as usize);

            println!("{}", log);
        };

        // let src = src.split('\n').map(|s| s.to_string() + "\n").collect::<Vec<_>>();
        let src = vec![src];

        let ptrs = src.iter().map(|s| s.as_ptr() as *const i8).collect::<Vec<_>>();
        let lens = src.iter().map(|s| s.len() as i32).collect::<Vec<_>>();

        println!{"{:?}", src};

        let shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(shader, src.len() as i32, &ptrs[0], &lens[0]);
        gl::CompileShader(shader);
        print_log(shader, gl::GetShaderiv, gl::GetShaderInfoLog);

        let program = gl::CreateProgram();
        gl::AttachShader(program, shader);
        gl::LinkProgram(program);
        print_log(program, gl::GetProgramiv, gl::GetProgramInfoLog);

        gl::ValidateProgram(program);
        print_log(program, gl::GetProgramiv, gl::GetProgramInfoLog);



    }
}
