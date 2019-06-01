#![recursion_limit="512"]
#![feature(trivial_bounds)]

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

    use self::ParticleUpdator::Particle;

    pub mod ParticleShader {
        @Vertex
            #version 140
            uniform float expansion;
            in vec2 pos;
            out float distance;
            void main() {
                distance = length(pos) / expansion;
                gl_Position = vec4(pos, 0, expansion);
            }

        @Fragment
            #version 140
            in float distance;
            void main() {
                gl_FragColor = vec4(1-distance/10, 0, 1, 1);
            }

    }

    pub mod ParticleUpdator {

        @Compute

            #version 440

            layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

            uniform float factor;
            uniform float dt;

            public struct Particle {
                vec2 pos;
                vec2 vel;
            };

            layout(std430) buffer Particles {
                Particle p1[];
            };

            layout(std430) buffer NewParticles {
                Particle p2[];
            };

            void main() {
                const float g = 0.1;

                vec2 p = p1[gl_GlobalInvocationID.x].pos;
                vec2 v = p1[gl_GlobalInvocationID.x].vel;

                // vec2 corner = vec2(-1,-1);
                // p2[gl_GlobalInvocationID.x].pos = mod((v * dt + p)-corner, 2)+corner;


                p2[gl_GlobalInvocationID.x].pos = (v * dt + p* factor*exp(dt));

                for(int i=0; i<gl_NumWorkGroups.x; i++) {
                    if(i!=gl_GlobalInvocationID.x) {
                        // vec2 d = p - p1[i].pos;
                        // float r2 = dot(d, d);
                        // if(r2 > 0.0000025) {
                        //     v -= g * dt * (1 / (r2)) * inversesqrt(r2) * d ;
                        // }

                        for(int x=-4; x<=4; x+=2){
                            for(int y=-4; y<=4; y+=2){
                                vec2 d = (p+vec2(x,y)) - p1[i].pos;
                                float r2 = dot(d, d);
                                if(r2 > 0.0000025) {
                                    v -= g * dt * (1 / (r2)) * inversesqrt(r2) * d ;
                                }
                            }
                        }
                    }
                }

                p2[gl_GlobalInvocationID.x].vel = v;

            }

    }

}


fn main() {

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let mut window = glfw.create_window(640*2, 480*2, "WOOOOOOOOOOOOO", glfw::WindowMode::Windowed).unwrap().0;

    glfw::Context::make_current(&mut window);
    window.set_key_polling(true);
    glfw.set_swap_interval(glfw::SwapInterval::None);

    let gl_provider = unsafe {
        GLProvider::load(|s| ::std::mem::transmute(glfw.get_proc_address_raw(s)))
    };
    let mut context = Context::init(&gl_provider);
    let mut shader = ParticleShader::init(&gl_provider).unwrap();
    let mut computer = ParticleUpdator::init(&gl_provider).unwrap();

    let num = 3000;
    let speed = 1.0;
    let mut init = Vec::with_capacity(num);
    for _i in 0..num {
        init.push(Particle{
            pos: [rand::random::<f32>() * 2.0 - 1.0, rand::random::<f32>() * 2.0 - 1.0].into(),
            vel: [(rand::random::<f32>() - 0.5)*speed, (rand::random::<f32>() - 0.5)*speed].into()
        });
    }
    let (mut buf1, mut buf2) =
        (Buffer::immut_from(&gl_provider, init.clone().into_boxed_slice()),
         Buffer::immut_from(&gl_provider, init.into_boxed_slice()));

    unsafe {
        gl::Viewport(80*2,0,480*2,480*2);
        gl::Disable(gl::CULL_FACE);
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::BLEND);
        gl::PointSize(2.0);
    }

    let mut flip = false;
    *shader.expansion = 1.0;
    *computer.factor = 1.000;
    *computer.dt = 0.0001;

    while !window.should_close() {
        // let start = ::std::time::Instant::now();

        glfw::Context::swap_buffers(&mut window);
        glfw.poll_events();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(1.0,1.0,1.0,0.0);
        }

        let p1 = &mut buf1;
        let p2 = &mut buf2;

        if flip {
            computer.compute(num as u32, 1, 1, p1, p2);
            shader.draw(&mut context, DrawMode::Points, num, Particle::get_attributes(p1).0);
        } else {
            computer.compute(num as u32, 1, 1, p2, p1);
            shader.draw(&mut context, DrawMode::Points, num, Particle::get_attributes(p2).0);
        }

        flip = !flip;
        *shader.expansion *= *computer.factor * (*computer.dt).exp();
        // shader.expansion = computer.factor;

        // ::std::thread::sleep(::std::time::Duration::from_millis(300));

        // println!("{:?}", ::std::time::Instant::now() - start);

    }



}
