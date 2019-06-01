#![recursion_limit="512"]

extern crate gl_struct;
extern crate glfw;
extern crate gl;

#[macro_use]
extern crate macro_program;

use gl_struct::*;

use std::f32::consts::PI;


glsl!{$

    pub mod Shaderinator {

        @Lib
            ///
            ///A struct for making a color gradient
            ///
            public struct Gradient {
                vec4 left;
                vec4 right;
            };

        @Vertex

            #version 450

            // uniform mat4 trans[60];

            extern struct Gradient;

            uniform Gradient colors;

            layout(std140) uniform Matrices {
                mat4 trans;
            };

            attribute vec3 pos;
            out vec4 color;

            void main() {
                gl_Position = trans * vec4(pos, 1);
                color = mix(colors.left, colors.right, (gl_Position.x+1)/2);
            }

        @Fragment

            #version 140

            in vec4 color;

            void main() {
                gl_FragColor = color;
            }

    }

}

use gl_struct::glsl_type;

fn main() {

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let mut window = glfw.create_window(640, 480, "WOOOOOOOOOOOOO", glfw::WindowMode::Windowed).unwrap().0;

    glfw::Context::make_current(&mut window);
    window.set_key_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::Adaptive);

    let gl_provider = unsafe {
        GLProvider::load(|s| ::std::mem::transmute(glfw.get_proc_address_raw(s)))
    };
    let mut context = Context::init(&gl_provider);
    let mut shader = Shaderinator::init(&gl_provider).unwrap();


    let points = [[-0.5f32,-0.5,0.0],[0.0,0.866,0.0],[0.5,-0.5,0.0]];
    // let points = [[-0.5f32,-0.5,0.0],[0.5,-0.5,0.0],[-0.5,0.5,0.0],[-0.5,0.5,0.0],[0.5,-0.5,0.0],[0.5,0.5,0.0]];
    let triangle: Buffer<[[f32;3]],_> = Buffer::immut_from(&gl_provider, Box::new(points));

    let theta = PI/4.0;
    let mat = [[theta.cos(),theta.sin(),0.0,0.0],[-theta.sin(),theta.cos(),0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]];

    let mut theta = 0.0f32;
    let mut mat = [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]];

    let mut trans = Buffer::<glsl_type::mat4, ReadWrite>::new(&gl_provider, mat.into());

    shader.colors.left = [1.0,1.0,0.0,1.0].into();
    shader.colors.right = [0.0,0.0,1.0,1.0].into();

    unsafe {
        gl::Viewport(80,0,480,480);
        gl::Disable(gl::CULL_FACE);
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::BLEND);
    }

    while !window.should_close() {
        let start = ::std::time::Instant::now();

        glfw::Context::swap_buffers(&mut window);
        glfw.poll_events();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(1.0,1.0,1.0,0.0);
        }

        theta = theta + 0.1;
        mat = [[theta.cos(),theta.sin(),0.0,0.0],[-theta.sin(),theta.cos(),0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]];
        trans.update_data(mat.into());

        shader.draw(&mut context, DrawMode::Triangles, 3, &mut trans, Attribute::Array(triangle.as_attrib_array()));

        println!("{:?}", ::std::time::Instant::now()-start);

        // ::std::thread::sleep(::std::time::Duration::from_millis(300));

    }



}
