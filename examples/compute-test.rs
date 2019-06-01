

#![recursion_limit="512"]

extern crate gl_struct;
extern crate glfw;
extern crate gl;
extern crate rand;

#[macro_use]
extern crate macro_program;

use gl_struct::*;
use gl_struct::glsl_type::*;

// use std::f32::consts::PI;

glsl!{$

    pub mod ParticleShader {
        @Vertex
            #version 140
            attribute vec4 pos;
            out float distance;
            void main() {
                distance = length(pos.xy);
                gl_Position = pos;
            }

        @Fragment
            #version 140
            in float distance;
            void main() {
                gl_FragColor = vec4(1, 1-distance, 0, 1);
            }

    }

    pub mod ParticleUpdator {

        @Compute

            #version 440

            layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

            layout(std430) buffer Positions {
                vec4 pos[];
            };



            void main() {
                const float dt = 0.001;
                // const float dt = 0.00025;
                const mat2 rot = mat2(vec2(0,1), vec2(-1,0));
                vec2 yn = pos[gl_GlobalInvocationID.x].xy;

                vec2 k1 = dt * normalize(rot * (yn));
                vec2 k2 = dt * normalize(rot * (yn + k1/2.0));
                vec2 k3 = dt * normalize(rot * (yn + k2/2.0));
                vec2 k4 = dt * normalize(rot * (yn + k3));

                // pos[gl_GlobalInvocationID.x].xy += k1;
                // pos[gl_GlobalInvocationID.x].xy += k2;
                pos[gl_GlobalInvocationID.x].xy += (k1 + 2*k2 + 2*k3 + k4) / 6;
            }

    }

}


fn main() {

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let mut window = glfw.create_window(640*2, 480*2, "WOOOOOOOOOOOOO", glfw::WindowMode::Windowed).unwrap().0;

    glfw::Context::make_current(&mut window);
    window.set_key_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::Adaptive);

    let gl_provider = unsafe {
        GLProvider::load(|s| ::std::mem::transmute(glfw.get_proc_address_raw(s)))
    };
    let mut context = Context::init(&gl_provider);
    let shader = ParticleShader::init(&gl_provider).unwrap();
    let computer = ParticleUpdator::init(&gl_provider).unwrap();

    let num = 2000000;
    let mut points = Vec::with_capacity(num);
    for _i in 0..num {
        points.push([rand::random::<f32>() * 2.0 - 1.0, rand::random::<f32>() * 2.0 - 1.0, 0.0, 1.0].into());
    }
    let mut particles: Buffer<[vec4], _> = Buffer::immut_from(&gl_provider, points.into_boxed_slice());

    unsafe {
        gl::Viewport(80*2,0,480*2,480*2);
        gl::Disable(gl::CULL_FACE);
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::BLEND);
    }

    while !window.should_close() {
        // let start = ::std::time::Instant::now();

        glfw::Context::swap_buffers(&mut window);
        glfw.poll_events();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(1.0,1.0,1.0,0.0);
        }

        let p = particles.len();

        computer.compute(p as u32, 1, 1, &mut particles);
        shader.draw(&mut context, DrawMode::Points, p, Attribute::Array(particles.as_attrib_array()));

        // ::std::thread::sleep(::std::time::Duration::from_millis(300));

        // println!("{:?}", ::std::time::Instant::now() - start);

    }



}
