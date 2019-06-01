
#![recursion_limit="1024"]

#[macro_use]
extern crate gl_struct;
extern crate glfw;
extern crate rand;

use gl_struct::*;
use gl_struct::glsl_type::*;

glsl! {$

    pub mod QRDecomp {

        @Compute

            #version 460
            #define I (mat4(vec4(1,0,0,0),vec4(0,1,0,0),vec4(0,0,1,0),vec4(0,0,0,1)))
            #define EPSILON 0.0001

            layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

            layout(std140) buffer mat_input { readonly restrict mat4 mats[]; };
            layout(std140) buffer rot_out { writeonly restrict mat4 rot[]; };
            layout(std140) buffer upper_triangular_out { writeonly restrict mat4 ut[]; };
            layout(std140) buffer mat_output { writeonly restrict mat4 result[]; };

            mat4 hausdorf(vec4 v) {
                float l2 = dot(v,v);
                return l2==0 ? I : I - (2/l2)*outerProduct(v,v);
            }

            void qr(in mat4 A, out mat4 Q, out mat4 R) {
                Q = I;  R = A;

                for(uint i=0; i<4; i++) {
                    vec4 a = R[i];
                    for(uint j=0; j<i; j++) { a[j] = 0.0; }

                    float alpha = length(a) * sign(a[i]);
                    vec4 ei = vec4(0.0,0.0,0.0,0.0);
                    ei[i] = alpha;

                    mat4 Qi = hausdorf(a - ei);
                    Q = Q * transpose(Qi);
                    R = Qi * R;
                }
            }

            dmat4 hausdorf_64(dvec4 v) {
                double l2 = dot(v,v);
                return l2==0 ? I : I - (2/l2)*outerProduct(v,v);
            }

            void qr_64(in dmat4 A, out dmat4 q, out dmat4 r) {
                dmat4 Q = I;
                dmat4 R = A;

                for(uint i=0; i<4; i++) {
                    dvec4 a = R[i];
                    for(uint j=0; j<i; j++) { a[j] = 0.0; }

                    double alpha = length(a) * sign(a[i]);
                    dvec4 ei = dvec4(0.0,0.0,0.0,0.0);
                    ei[i] = alpha;

                    dmat4 Qi = hausdorf_64(a - ei);
                    Q = Q * transpose(Qi);
                    R = Qi * R;
                }

                q = Q;
                r = R;

            }

            mat4 babylonian_sqrt(mat4 X) {
                mat4 Xi = I;

                const uint N = 20;
                for(uint i=0; i<N; i++) Xi = 0.5*(Xi + X*inverse(Xi));

                return Xi;
            }

            dmat4 babylonian_sqrt_64(dmat4 X) {
                dmat4 Xi = I;

                const uint N = 100;
                for(uint i=0; i<N; i++) Xi = 0.5*(Xi + X*inverse(Xi));

                return Xi;
            }

            dmat4 denman_beavers_64(dmat4 X) {
                dmat4 Yi = X;
                dmat4 Zi = I;

                const uint N = 5;
                for(uint i=0; i<N; i++){
                    dmat4 Y = 0.5*(Yi + inverse(Zi));
                    dmat4 Z = 0.5*(Zi + inverse(Yi));
                    Yi = Y;
                    Zi = Z;
                }

                return Yi;
            }


            void main() {
                uint id = gl_GlobalInvocationID.x;

                // mat4 q;
                // mat4 r;
                // qr(mats[id], q, r);
                //
                // rot[id] = q;
                // ut[id] = r;
                // result[id] = q*r;

                mat4 u = mat4(denman_beavers_64(mats[id]));
                ut[id] = u;
                result[id] = u*u;

                // dmat4 q;
                // dmat4 r;
                // qr_64(mats[id], q, r);
                //
                // rot[id] = mat4(q);
                // ut[id] = mat4(r);
                // result[id] = mat4(q*r);
            }


    }


}


fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let mut window = glfw.create_window(640, 480, "QR-Decomposition", glfw::WindowMode::Windowed).unwrap().0;
    glfw::Context::make_current(&mut window);

    let gl_provider = unsafe {
        GLProvider::load(|s| ::std::mem::transmute(glfw.get_proc_address_raw(s)))
    };

    let decomposer = QRDecomp::init(&gl_provider).unwrap();

    fn rf() -> f32 {rand::random::<f32>()}

    let count = 10000;
    let mats = (0..count).map(
        |_| {
            let rand = [[rf(),rf(),rf(),rf()],[rf(),rf(),rf(),rf()],[rf(),rf(),rf(),rf()],[rf(),rf(),rf(),rf()]];
            //now make a positive semidefinite matrix
            let mut m = [[0.0;4];4];
            for i in 0..4 {
                for j in 0..4 {
                    m[j][i] = 0.0;
                    for k in 0..4 {
                        m[j][i] += rand[i][k]*rand[j][k];
                    }
                }
            }

            m.into()
        }
    ).collect::<Vec<mat4>>();

    let mut mat_buf: Buffer<[mat4], _> = Buffer::readonly_from(&gl_provider, mats.into_boxed_slice());
    let mut q_buf = mat_buf.clone();
    let mut r_buf = mat_buf.clone();
    let mut res_buf = mat_buf.clone();

    let start = ::std::time::Instant::now();
    decomposer.compute(mat_buf.len() as u32, 1, 1, &mut mat_buf, &mut q_buf, &mut r_buf, &mut res_buf);
    let (b1, b2, b3, b4) = (mat_buf.into_box(), q_buf.into_box(), r_buf.into_box(), res_buf.into_box());

    println!("{:?}", ::std::time::Instant::now() - start);

    for i in 0..count {
        // println!("{:?}", b1[i].value[0]);
        // println!("{:?}", b1[i].value[1]);
        // println!("{:?}", b1[i].value[2]);
        // println!("{:?}", b1[i].value[3]);
        println!("{} {:?} == {:?}", b1[i].value == b4[i].value, b1[i].value, b4[i].value);
        // println!("{} {:?} {:?} == {:?}", b1[i].value == b4[i].value, b3[i].value, b1[i].value, b4[i].value);
        println!();
    }

}
