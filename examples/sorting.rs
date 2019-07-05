#![recursion_limit="1024"]

extern crate gl_struct;
extern crate glfw;
extern crate gl;
extern crate rand;
extern crate rayon;

#[macro_use]
extern crate macro_program;

use gl_struct::*;
use rayon::slice::ParallelSliceMut;
use rayon::iter::*;
// use gl_struct::glsl_type::glsl::*;

use std::env;

glsl! {$

    pub mod SmallBitonic {

        @Compute

            #version 450

            layout(local_size_x = 8, local_size_y = 1, local_size_z = 1) in;


            layout(std430) buffer Values {
                float values[];
            };


            void main() {
                for(uint order=0; order<4; order++){
                    for(uint suborder=order; ; suborder--){

                        bool flip = suborder==order && order > 0;
                        uint swap_offset = 1 << suborder;
                        uint mask = 0xFFFFFFFF << suborder;
                        uint id = ((gl_GlobalInvocationID.x & mask) << 1) | (gl_GlobalInvocationID.x & ~mask);

                        uint offset;
                        if(!flip){
                            offset = swap_offset;
                        } else {
                            offset = (swap_offset<<1) - ((gl_GlobalInvocationID.x & ~mask)<<1) - 1;
                        }

                        if(values[id] > values[id+offset]){
                            float tmp = values[id];
                            values[id] = values[id+offset];
                            values[id+offset] = tmp;
                        }

                        barrier();
                        if(suborder==0) break;
                    }
                }
            }

    }


    pub mod BitonicSwaps {

        @Compute

            #version 450

            layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

            uniform uint order;
            uniform bool flip = false;

            layout(std430) buffer Values {
                float values[];
            };


            void main() {
                const uint swap_offset = 1 << order;
                const uint mask = 0xFFFFFFFF << order;
                uint id = ((gl_GlobalInvocationID.x & mask) << 1) | (gl_GlobalInvocationID.x & ~mask);

                uint offset;
                if(!flip){
                    offset = swap_offset;
                } else {
                    offset = (swap_offset<<1) - ((gl_GlobalInvocationID.x & ~mask)<<1) - 1;
                }

                if(values[id] > values[id+offset]){
                    float tmp = values[id];
                    values[id] = values[id+offset];
                    values[id+offset] = tmp;
                }

            }

    }


}



glsl! {$
    pub mod XtremeBubbleSort {

        @Compute

            #version 450

            uniform bool offset;

            layout(local_size_x = 1024, local_size_y = 1, local_size_z = 1) in;

            layout(std430) buffer Values {
                float values[];
            };

            void main() {
                uint ptr = gl_GlobalInvocationID.x;
                bool odd = (ptr & 1) == 1;
                for(uint i=0; i<gl_WorkGroupSize.x; i++) {
                    if(!odd) {
                        if(values[ptr] > values[ptr+1]) {
                            float tmp1 = values[ptr];
                            float tmp2 = values[ptr+1];
                            values[ptr] = tmp2;
                            values[ptr+1] = tmp1;
                        }
                    }
                    barrier();
                    if(odd) {
                        if(values[ptr] > values[ptr+1]) {
                            float tmp1 = values[ptr];
                            float tmp2 = values[ptr+1];
                            values[ptr] = tmp2;
                            values[ptr+1] = tmp1;
                        }
                    }
                    barrier();
                }

            }

    }
}

fn main() {

    //get the runtime params
    let args: Vec<String> = env::args().collect();

    let mut gpu = false;
    let mut cpu = false;
    let mut par_cpu = false;
    let mut order = 0;

    //parse the params
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--gpu" { gpu = true; }
        else if args[i] == "--cpu" { cpu = true; }
        else if args[i] == "--par_cpu" { par_cpu = true; }
        else if args[i] == "--order" || args[i] == "--o" {
            i+=1;
            if i < args.len() {
                order = args[i].parse::<u32>().unwrap();
            }
        }
        i+=1;
    }

    //create the list we will sort
    let num = 1 << order;
    println!("Generating {} random numbers", num);
    let rand_start = ::std::time::Instant::now();
    let mut points = Vec::with_capacity(num);
    unsafe {points.set_len(num)};
    points.par_iter_mut().for_each(|f| *f = rand::random::<f32>());
    println!("List generated in {:?}", ::std::time::Instant::now() - rand_start);

    if gpu {
        //set up a glfw window to get a gl context
        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let mut window = glfw.create_window(640, 480, "Sorting!", glfw::WindowMode::Windowed).unwrap().0;
        glfw::Context::make_current(&mut window);
        window.set_key_polling(false);

        //get the context and load the gl methods
        let gl_provider = unsafe {
            GLProvider::load(|s| ::std::mem::transmute(glfw.get_proc_address_raw(s)))
        };
        let context = Context::init(&gl_provider);

        //init the shaders for the gpu sorting
        let mut computer = BitonicSwaps::init(&gl_provider).unwrap();
        let mut small_bitonic = SmallBitonic::init(&gl_provider).unwrap();

        println!("uploading to gpu");
        let upload_start = ::std::time::Instant::now();
        let mut list: Buffer<[f32], _> = Buffer::readonly_from(&gl_provider, points.clone().into_boxed_slice());
        unsafe { gl::Finish(); }

        println!("Data uploaded in {:?}", ::std::time::Instant::now() - upload_start);
        println!("Starting GPU sorting");

        let start = ::std::time::Instant::now();

        fn bitonic_sort<A:ReadAccess>(n: u32, c:&mut BitonicSwaps::Program, c2:&mut SmallBitonic::Program, buf: &mut Buffer<[f32], A>) {

            if n==0 {
                return;
            } else if n==4 {
                c2.compute((buf.len()>>4) as u32, 1, 1, buf);
            } else {
                bitonic_sort(n-1, c, c2, buf);

                *c.order = n-1;
                c.flip.set(true);
                c.compute((buf.len()>>1) as u32, 1, 1, buf);

                c.flip.set(false);
                for m in (0..n-1).rev() {
                    *c.order = m;
                    c.compute((buf.len()>>1) as u32, 1, 1, buf);
                }
            }

        }

        // small_bitonic.compute(1, 1, 1, &mut list);
        // unsafe { gl::Finish(); }
        bitonic_sort(order, &mut computer, &mut small_bitonic, &mut list);
        unsafe { gl::Finish();}


        println!("Finished GPU sorting in {:?}", ::std::time::Instant::now() - start);

        println!("Reading GPU buffer");
        let download_start = ::std::time::Instant::now();
        let res = list.read_into_box();
        println!("Finished Reading in {:?}", ::std::time::Instant::now()-download_start);
        // println!("{:?}", res);

    }

    if cpu {
        println!("Starting CPU sorting");
        let mut start = ::std::time::Instant::now();
        if !par_cpu {
            points.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            let mut new_p = points.clone();
            start = ::std::time::Instant::now();
            new_p.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        }
        println!("Finished CPU sorting in {:?}", ::std::time::Instant::now()-start);
    }

    if par_cpu {
        println!("Starting Parallel CPU sorting");
        let start = ::std::time::Instant::now();
        points.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        println!("Finished Parallel CPU sorting in {:?}", ::std::time::Instant::now()-start);
        // println!("{:?}", points);
    }

}
