#![recursion_limit="2048"]
#![feature(trivial_bounds)]

extern crate gl_struct;
extern crate glutin;
extern crate gl;

use gl_struct::*;
use gl_struct::glsl_type::*;

use std::io::Write;
use std::f32::consts::PI;

const WIDTH:u32 = 1280;
const HEIGHT:u32 = 720;
const RECH: u32 = 720;
const RECW: u32 =  RECH*WIDTH/HEIGHT;

const CENTER: [f64;2] = [-0.0753889159636163, 0.6238713078071512];
// const ZOOM: f64 = 1.0;
const ZOOM: f64 = 0.5;
// const ZOOM: f64 = 0.0000000000000009807156282925116;
const ZOOM_RATE: f64 = 0.99;
const ZOOM_SCROLL: f64 = 0.9;

const RECORD: bool = true;
const DO_ZOOM: bool = true;

glsl!{$

    pub mod Mandelbrot {

        @Rust

            pub const MANTISSA_BYTES: usize = 4;
            pub const BITS: GLuint = GLuint::BITS;

        @Vertex

            #version 450

            uniform mat4 trans;

            in vec2 pos;
            in vec2 uv;

            out vec2 c;

            void main() {
                c = uv;
                gl_Position = trans * vec4(pos,0.0,1.0);
            }

        @Fragment

            #version 410
            #define MANTISSA_BYTES 4
            #define BITS 32
            #define PRECISION 3
            #define MAX_ITER 1000
            // #pragma optionNV unroll all

            //we have to not have sign first to avoid a seg fault :/
            public struct HighPFloat {
                int exp;
                uint mantissa[4];
                bool sign;
            };

            HighPFloat shr(HighPFloat x, uint b, uint carry) {

                HighPFloat dest;
                dest.sign = x.sign;
                dest.exp = x.exp;

                uint q = b / 32;
                uint r = b % 32;

                for(uint i=0; i<MANTISSA_BYTES; i++) {
                    dest.mantissa[i] =
                        i < MANTISSA_BYTES-q ? x.mantissa[i+q] :
                        i == MANTISSA_BYTES-q ? carry : 0;
                }
                carry = q>0 ? 0 : carry;

                uint low_mask = (~0) >> (BITS-r);
                for(int i=MANTISSA_BYTES-1; i>=0; i--) {
                    uint low_bits = dest.mantissa[i] & low_mask;
                    dest.mantissa[i] >>= r;
                    dest.mantissa[i] |= (carry << (BITS-r));
                    carry = low_bits;
                }
                return dest;
            }

            HighPFloat shr(HighPFloat x, uint b) {
                return shr(x,b,0);
            }

            HighPFloat shl(HighPFloat x, uint b) {
                HighPFloat dest;
                dest.sign = x.sign;
                dest.exp = x.exp;

                uint q = b / 32;
                uint r = b % 32;
                for(uint i=MANTISSA_BYTES-1; i>=0; i--) {
                    dest.mantissa[i] = i>=q ? x.mantissa[i-q] : 0;
                }

                uint carry = 0;
                uint high_mask = (~0) << (BITS-r);
                for(int i=0; i<MANTISSA_BYTES; i++) {
                    uint high_bits = dest.mantissa[i] & high_mask;
                    dest.mantissa[i] <<= r;
                    dest.mantissa[i] |= carry >> (BITS-r);
                    carry = high_bits;
                }
                return dest;
            }

            HighPFloat normalize_exp(HighPFloat x) {
                for(int b=0; b<MANTISSA_BYTES; b++){
                    for(int i=0; i<BITS; i++) {
                        uint msb = x.mantissa[MANTISSA_BYTES-1-b];
                        if((msb & (1<<(BITS-1-i))) != 0){
                            if(i==0 && b==0) return x;

                            x = shl(x, BITS*b + i);
                            x.exp -= b*BITS + i;
                            return x;
                        }
                    }
                }
                return x;
            }

            HighPFloat add(HighPFloat a, HighPFloat b) {

                if(b.exp>a.exp){
                    a = shr(a, b.exp - a.exp);
                    a.exp = b.exp;
                }
                if(a.exp>b.exp){
                    b = shr(b, a.exp - b.exp);
                    b.exp = a.exp;
                }

                HighPFloat dest;
                dest.sign = a.sign;
                dest.exp = a.exp;

                //add digit-wise
                bool sub = a.sign ^^ b.sign;

                if(sub) {
                    uint borrow = 0;
                    for(int i=0; i<MANTISSA_BYTES; i++) {

                        uint d1 = a.mantissa[i];
                        uint d2 = b.mantissa[i];

                        uint c1=0, c2=0;
                        dest.mantissa[i] = usubBorrow(usubBorrow(d1, d2, c1), borrow, c2);
                        borrow = c1|c2;
                    }

                    if(borrow != 0) {
                        //if we have a carry when subtracting the mantissas, flip the sign
                        //and find the twos complement of the result
                        dest.sign = !a.sign;
                        uint carry = 1;
                        for(int i=0; i<MANTISSA_BYTES; i++) {
                            dest.mantissa[i] = uaddCarry(~dest.mantissa[i], carry, carry);
                        }
                    }

                } else {
                    uint carry = 0;
                    for(int i=0; i<MANTISSA_BYTES; i++) {

                        uint d1 = a.mantissa[i];
                        uint d2 = b.mantissa[i];

                        uint c1=0, c2=0;
                        dest.mantissa[i] = uaddCarry(uaddCarry(d1, d2, c1), carry, c2);
                        carry = c1+c2;
                    }

                    if(carry == 1) {
                        dest = shr(dest, 1, carry);
                        dest.exp++;
                    } else if(carry>1) {
                        dest = shr(dest, 2, carry);
                        dest.exp++;
                    }
                }

                return normalize_exp(dest);
                // return dest;

            }

            HighPFloat neg(HighPFloat x) {
                x.sign = !x.sign;
                return x;
            }

            HighPFloat sub(HighPFloat a, HighPFloat b) {
                return add(a, neg(b));
            }

            HighPFloat mul(HighPFloat a, HighPFloat b) {

                uint dest[MANTISSA_BYTES*2];
                for(int i=0; i<MANTISSA_BYTES*2; i++) dest[i] = 0;

                for(int i=0; i<MANTISSA_BYTES; i++) {
                    uint d1 = a.mantissa[i];
                    for(int j=0; j<MANTISSA_BYTES; j++) {
                        uint d2 = b.mantissa[j];

                        //multiply the digits
                        uint msb, lsb;
                        umulExtended(d1, d2, msb, lsb);

                        //add the result to the dest
                        int start = i+j;
                        uint carry = 0;
                        uint c1=0,c2=0;

                        dest[start] = uaddCarry(lsb, dest[start], carry);
                        dest[start+1] = uaddCarry(uaddCarry(msb, dest[start+1], c1), carry, c2);

                        carry = c1+c2;
                        for(int k=start+2; k<MANTISSA_BYTES*2; k++) {
                            if(carry==0) break;
                            dest[k] = uaddCarry(dest[k], carry, carry);
                        }

                    }
                }

                // for(int i=0; i<MANTISSA_BYTES; i++) dest[i] = a.mantissa[i];

                HighPFloat prod;
                prod.exp = a.exp+b.exp;
                prod.sign = a.sign ^^ b.sign;

                //find the most significant byte
                int end = MANTISSA_BYTES*2 - 1;
                while(end>MANTISSA_BYTES && dest[end]==0) end--;
                int start = end-MANTISSA_BYTES;

                //copy in everything before the most significant byte
                for(int i=0; i<MANTISSA_BYTES; i++) {
                    prod.mantissa[i] = dest[start+i];
                }


                //find the most significant bit
                int msb = findMSB(dest[end]) + 1;

                //adjust the exponent according to where the msb is
                prod.exp += (end-MANTISSA_BYTES*2)*32 + msb;

                if(msb>0){
                    //bitshift right filling in the extra bits
                    prod = shr(prod, uint(msb), dest[end]);
                } else {
                    prod = normalize_exp(prod);
                }

                return prod;
            }

            HighPFloat to_highp(float x) {

                uint bits = floatBitsToUint(x);

                HighPFloat dest;
                dest.sign = (bits & (1<<(BITS-1))) != 0;
                dest.exp = int((bits >> 23) & 0xFF) - 127;

                for(int i=0; i<MANTISSA_BYTES; i++) dest.mantissa[i] = 0;

                uint m = (bits & 0x7FFFFF) << (BITS-24);
                if(dest.exp<=-127) {
                    if(m==0) {
                        dest.exp = 0;
                    } else {
                        dest.exp += 1;
                        dest.mantissa[MANTISSA_BYTES-1] = m << 1;
                        dest = normalize_exp(dest);
                    }
                } else {
                    dest.exp += 1;
                    dest.mantissa[MANTISSA_BYTES-1] = m | (1<<31);
                }

                return dest;

            }

            float to_float(HighPFloat x) {

                x = normalize_exp(x);

                uint mantissa = x.mantissa[MANTISSA_BYTES-1] >> (BITS-24);
                if(mantissa==0) {
                    mantissa = 0;
                    x.exp = -127;
                } else if(x.exp<=-126) {
                    mantissa >>= 1 + (-126 - x.exp);
                    x.exp = -127;
                } else {
                    x.exp -= 1;
                    mantissa &= 0x7FFFFF;
                }

                uint exp = (uint(x.exp+127) & 0xFF) << 23;
                uint sign = x.sign ? (1<<31) : 0;

                return uintBitsToFloat( sign | exp | mantissa);
            }

            vec3 hsv2rgb(vec3 c) {
                vec4 k = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
                vec3 p = abs(fract(c.xxx + k.xyz) * 6.0 - k.www);
                return c.z * mix(k.xxx, clamp(p - k.xxx, 0.0, 1.0), c.y);
            }


            uniform dvec2 center;
            uniform double scale;

            //NOTE:: if we use the first field but not all of the rest, we get a segfault... oops?
            uniform HighPFloat scale_hp;

            in vec2 c;

            void main() {

                int iter;
                if(PRECISION==1) {
                    vec2 c0 = (c* float(scale) - vec2(center));
                    vec2 z = c0;
                    for(iter=1; iter<=MAX_ITER; iter++) {
                        if(dot(z,z) >= 4) break;
                        z = vec2(z.s*z.s - z.t*z.t, 2.0*z.s*z.t) + c0;
                    }
                } else if(PRECISION==2) {
                    dvec2 c0 = (dvec2(c)*scale - center);
                    dvec2 z = c0;
                    for(iter=1; iter<=MAX_ITER; iter++) {
                        if(dot(z,z) >= 4) break;
                        z = dvec2(z.s*z.s - z.t*z.t, 2.0*z.s*z.t) + c0;
                    }
                } else {

                    HighPFloat zero = to_highp(0.0);
                    HighPFloat two = to_highp(2.0);
                    HighPFloat four = to_highp(4.0);

                    HighPFloat c1 = sub(mul(to_highp(c.x), scale_hp), to_highp(float(center.x)));
                    HighPFloat c2 = sub(mul(to_highp(c.y), scale_hp), to_highp(float(center.y)));

                    HighPFloat x2 = zero;
                    HighPFloat y2 = zero;
                    HighPFloat w = zero;

                    for(iter=0; iter<=MAX_ITER; iter++) {

                        HighPFloat modulus = add(x2, y2);
                        HighPFloat cmp = sub(four, modulus);
                        if(cmp.sign) break;

                        HighPFloat x = add(sub(x2, y2), c1);
                        HighPFloat y = add(sub(w, modulus), c2);

                        x2 = mul(x, x);
                        y2 = mul(y, y);

                        w = add(x, y);
                        w = mul(w, w);

                    }

                    // HighPFloat two = to_highp(2.0);
                    // HighPFloat four = to_highp(4.0);
                    // HighPFloat c1 = sub(mul(to_highp(c.x), scale_hp), to_highp(float(center.x)));
                    // HighPFloat c2 = sub(mul(to_highp(c.y), scale_hp), to_highp(float(center.y)));
                    // HighPFloat a=c1;
                    // HighPFloat b=c2;
                    //
                    // for(iter=1; iter<=MAX_ITER; iter++) {
                    //
                    //     HighPFloat modulus = add(mul(a,a), mul(b,b));
                    //     HighPFloat cmp = sub(four, modulus);
                    //
                    //     if(cmp.sign) {
                    //         break;
                    //     } else {
                    //
                    //         HighPFloat z1 = sub(mul(a,a), mul(b,b));
                    //         HighPFloat z2 = mul(two, mul(a,b));
                    //
                    //         a = add(z1, c1);
                    //         b = add(z2, c2);
                    //
                    //     }
                    //
                    // }

                }

                if(iter>MAX_ITER) {
                    gl_FragColor = vec4(0,0,0,1);
                } else {

                    // float d = float(length(z)) * log(float(length(z))) / float(2*length(dz));
                    // float hue = 0.001/sqrt(d);
                    // gl_FragColor = vec4(hsv2rgb(vec3(hue,0.5,1-d)), 1.0);

                    float x = float(iter)*0.01 - 1;
                    float light = 0.8 / (exp(-x) + 1);
                    float hue = sqrt(iter)/50.0;
                    gl_FragColor = vec4(hsv2rgb(vec3(hue+0.6,0.5,light)), 1.0);


                    // gl_FragColor = vec4(1);

                    // float x = float(iter)*0.1 - 50;
                    // float t = 1 / (exp(-x) + 1);
                    // gl_FragColor = vec4(t,t,t, 1.0);

                    // gl_FragColor = vec4(1-d,1-d,1-d,1);

                    // float t = float(iter)/float(MAX_ITER);
                    // gl_FragColor = vec4(hsv2rgb(vec3(t+0.5,0.5,t)), 1.0);
                }


            }

    }

}

use std::ops::{Mul, MulAssign};

impl HighPFloat {

    fn shr(self, b:u32, carry:u32) -> Self {
        let mut dest = self;
        let (q, r) = ((b/BITS) as usize, b%BITS);
        for i in 0..MANTISSA_BYTES {
            let start = MANTISSA_BYTES-q;
            dest.mantissa[i] = if i>start { 0 } else if i==start { carry } else {
                self.mantissa[i+q]
            };
        }
        let mut carry = if q>0 { 0 } else { carry };

        let b = r;
        let mask = (!0u32).checked_shr(BITS-b).unwrap_or(0);
        for digit in dest.mantissa.iter_mut().rev() {
            let low_bits = *digit & mask;
            *digit >>= b;
            *digit |= carry.checked_shl(BITS-b).unwrap_or(0);
            carry = low_bits;
        }

        dest

    }

    fn shl(self, b:u32, carry:u32) -> Self {
        let mut dest = self;
        let (q, r) = ((b/BITS) as usize, b%BITS);
        for i in (0..MANTISSA_BYTES).rev() {
            dest.mantissa[i] = if i<q { 0 } else { self.mantissa[i-q] }
        }

        let b = r;
        let mask = (!0u32).checked_shl(BITS-b).unwrap_or(0);
        let mut carry = carry;
        for digit in &mut dest.mantissa {
            let high_bits = *digit & mask;
            *digit <<= b;
            *digit |= carry.checked_shr(BITS-b).unwrap_or(0);
            carry = high_bits;
        }

        dest

    }

    fn normalize_exp(self) -> Self {
        let mut dest = self;
        for byte in (0..MANTISSA_BYTES) {
            let shift = dest.mantissa[MANTISSA_BYTES-1-byte].leading_zeros();
            if shift != BITS {
                let shift = byte as u32 * BITS + shift;
                dest = dest.shl(shift, 0);
                dest.exp -= shift as i32;
                return dest;
            }
        }
        dest
    }

    fn from_f32(x:f32) -> Self {
        let bits = x.to_bits();

        let sign = (bits & (1<<(BITS-1))) != 0;
        let mut exp = ((bits >> 23) & 0xFF) as i32 - 127;
        let mut mantissa = [0; MANTISSA_BYTES];

        let m = (bits & 0x7FFFFF) << (BITS-24);
        if(exp<=-127) {
            if(m==0) {
                exp = 0;
            } else {
                exp += 1;
                mantissa[MANTISSA_BYTES-1] = m << 1;
            }
        } else {
            exp += 1;
            mantissa[MANTISSA_BYTES-1] = m | (1<<31);
        }

        HighPFloat { sign:sign.into(), exp, mantissa}.normalize_exp()
    }

    fn from_f64(x:f64) -> Self {

        //constants
        const BITS: u64 = 64;
        const SIGN_MASK: u64 = 1<<(BITS-1);
        const EXP_BITS: u64 = 11;
        const EXP_MASK: u64 = ((!0) << (BITS-EXP_BITS)) >> 1;
        const EXP_BIAS: i64 = (1 << (EXP_BITS-1)) as i64 - 1;
        const SIG_BITS: u64 = 52;
        const SIG_MASK: u64 = (!0) >> (BITS-SIG_BITS);

        //reinterpret as bits
        let bits = x.to_bits();

        //extract each component
        let sign = (bits & SIGN_MASK) != 0;
        let mut exp = ((bits & EXP_MASK) >> SIG_BITS) as i64 - EXP_BIAS;
        let mut m = (bits & SIG_MASK) << (BITS-SIG_BITS);

        //adjust for subnormal numbers
        if(exp <= -(EXP_BIAS as i64)) {
            if(m==0) {
                //if x is 0, make the exp 0
                exp = 0;
            } else {
                exp += 1;
            }
        } else {
            exp += 1;
            m >>= 1;
            m |= SIGN_MASK;
        }

        let mut mantissa = [0; MANTISSA_BYTES];
        mantissa[MANTISSA_BYTES-1] = (m>>32) as u32;
        mantissa[MANTISSA_BYTES-2] = m as u32;

        HighPFloat { sign:sign.into(), exp: exp as i32, mantissa}.normalize_exp()
    }

    fn to_f32(self) -> f32 {
        let mut x = self.normalize_exp();

        let mut mantissa = x.mantissa[MANTISSA_BYTES-1] >> (BITS-24);
        if(mantissa==0) {
            mantissa = 0;
            x.exp = -127;
        } else if(x.exp<=-126) {
            let shift = 1 + (-126 - x.exp);
            if shift < BITS as i32 { mantissa >>= shift; } else { mantissa = 0; }
            x.exp = -127;
        } else {
            x.exp -= 1;
            mantissa &= 0x7FFFFF;
        }

        let exp = ((x.exp+127) as u32 & 0xFF) << 23;
        let sign = if x.sign.into() { 1<<(BITS-1) } else { 0 };

        return f32::from_bits(sign | exp | mantissa);
    }

}

// impl Add for HighPFloat {
//     type Output = Self;
//     fn add(self, rhs:Self) -> Self {
//         let (mut a, mut b) = (self, rhs);
//
//         //shift the decimals into the same location
//         if a.exp > b.exp {
//             b = shr(b, a.exp-b.exp, 0);
//             b.exp = a.exp;
//         } else if b.exp > a.exp {
//             b = shr(b, b.exp-a.exp, 0);
//             a.exp = b.exp;
//         }
//
//         //dest should have the same exp and sign as a, for now
//         let dest = a;
//
//         let sub = a.sign ^ b.sign;
//         let mut carry = 0;
//         for i in 0..MANTISSA_BYTES {
//
//             let (d1, d2) = (a.mantissa[i] as u64, b.mantissa[j] as u64);
//
//             if sub {
//
//             } else {
//                 let sum = d1 + d2;
//                 let c1 = (sum >> BITS);
//                 let sum = sum + carry;
//                 let c2 = (sum >> BITS);
//                 dest[i] = sum as u32;
//                 carry = c1 + c2;
//             }
//
//         }
//
//     }
// }

use crate::Mandelbrot::{HighPFloat, BITS, MANTISSA_BYTES};

impl MulAssign for HighPFloat {
    fn mul_assign(&mut self, rhs: Self) { *self = *self * rhs; }
}

impl Mul for HighPFloat {

    type Output = Self;
    fn mul(self, rhs:Self) -> Self {

        let (a, b) = (self, rhs);
        let mut dest = [0u32; MANTISSA_BYTES*2];

        for (i, d1) in a.mantissa.iter().copied().enumerate() {
            for (j, d2) in b.mantissa.iter().copied().enumerate() {

                //multiply the digits
                let prod = d1 as u64 * d2 as u64;
                let msb = prod >> BITS;
                let lsb = prod & (!0 >> BITS);

                //add the result to the dest
                let start = i+j;

                //add the product of the digits to the accumulator
                let sum1 = dest[start] as u64 + lsb;
                let sum2 = dest[start+1] as u64 + msb + (sum1>>BITS);

                //store the result
                dest[start] = sum1 as u32;
                dest[start+1] = sum2 as u32;

                //ripple any carry
                let mut carry = sum2 >> BITS;
                for k in (start+2)..(MANTISSA_BYTES*2) {
                    if carry==0 { break; }
                    let sum = dest[k] as u64 + carry;
                    dest[k] = sum as u32;
                    carry = sum >> BITS;
                }

            }
        }

        let mut prod = HighPFloat::default();
        prod.exp = a.exp+b.exp;
        prod.sign = a.sign ^ b.sign;

        //find the most significant byte
        let mut end = MANTISSA_BYTES*2 - 1;
        while end>MANTISSA_BYTES && dest[end]==0 { end-=1; }
        let start = (end-MANTISSA_BYTES) as usize;
        let end = end as usize;

        //copy in everything before the most significant byte
        for i in 0..MANTISSA_BYTES {
            prod.mantissa[i] = dest[start+i];
        }

        //find the most significant bit
        let msb = BITS - dest[end].leading_zeros();

        //adjust the exponent according to where the msb is
        prod.exp += (end as i32 - MANTISSA_BYTES as i32 * 2) * BITS as i32 + msb as i32;

        if(msb>0){
            //bitshift right filling in the extra bits
            prod.shr(msb, dest[end])
        } else {
            prod.normalize_exp()
        }

    }

}

use gl_struct::glsl_type;


use glutin::*;
use glutin::event::*;
use glutin::event_loop::*;
use glutin::window::*;
use glutin::dpi::*;

use std::sync::*;
use std::mem::MaybeUninit;

fn main() {

    let events = EventLoop::new();

    let wb = WindowBuilder::new()
        .with_title("Mandelbrot")
        .with_inner_size(glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));
    let window = ContextBuilder::new()
        .with_gl(GlRequest::Latest)
        .with_gl_profile(GlProfile::Compatibility)
        .build_windowed(wb, &events)
        .unwrap();

    let running = Arc::new(Mutex::new(true));
    let cursor_delta = Arc::new(Mutex::new((0.0,0.0)));
    let scroll_delta = Arc::new(Mutex::new(0.0));

    std::thread::Builder::new()
    .name("render_loop".to_string())
    .stack_size(8*1024*1024)
    .spawn({

        let running = running.clone();
        let cursor_delta = cursor_delta.clone();
        let scroll_delta = scroll_delta.clone();

        move || {

            let window = unsafe { window.make_current().unwrap() };

            let gl_provider = unsafe {
                GLProvider::load(|s| ::std::mem::transmute(window.get_proc_address(s)))
            };
            let mut context = gl_struct::Context::init(&gl_provider);
            let mut shader = Mandelbrot::init(&gl_provider).unwrap();

            let y0 = 2.0;
            let x0 = y0 * WIDTH as f32 / HEIGHT as f32;

            let [p1, p2, p3, p4] = [[1.0,1.0], [-1.0,1.0], [-1.0,-1.0], [1.0,-1.0]];
            // let [t1, t2, t3, t4] = [[1.0,1.0], [-1.0,1.0], [-1.0,-1.0], [1.0,-1.0]];
            let [t1, t2, t3, t4] = [[x0,y0], [-x0,y0], [-x0,-y0], [x0,-y0]];

            let points = [
                (p1,t1), (p2,t2), (p3,t3),
                (p1,t1), (p3,t3), (p4,t4)
            ];
            let quad: Buffer<[_],_> = Buffer::immut_from(&gl_provider, Box::new(points));


            let mut mat: mat4 = [[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[0.0,0.0,0.0,1.0]].into();
            let mut scale = ZOOM;

            *shader.trans = mat;
            *shader.center = CENTER.into();
            *shader.scale = scale;
            *shader.scale_hp = HighPFloat::from_f64(ZOOM);

            unsafe {
                gl::Disable(gl::CULL_FACE);
                gl::Disable(gl::DEPTH_TEST);
                gl::Disable(gl::BLEND);
            }

            let mut pixels = Box::new([0u8; 3 * RECW as usize * RECH as usize]);

            let [color,depth] = unsafe {
                let mut rb = MaybeUninit::<[GLuint;2]>::uninit();
                gl::GenRenderbuffers(2, &mut rb.assume_init_mut()[0] as *mut GLuint);
                rb.assume_init()
            };

            let fb = unsafe {
                let mut fb = MaybeUninit::uninit();
                gl::GenFramebuffers(1, fb.assume_init_mut());
                fb.assume_init()
            };

            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, fb);

                gl::BindRenderbuffer(gl::RENDERBUFFER, color);
                gl::RenderbufferStorage(gl::RENDERBUFFER, gl::RGBA8, RECW as _, RECW as _);
                gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, color);

                gl::BindRenderbuffer(gl::RENDERBUFFER, depth);
                gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, RECW as _, RECW as _);
                gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, depth);

                gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

                let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
                if status != gl::FRAMEBUFFER_COMPLETE {
                    panic!("framebuffer is not complete :(");
                }


                const BUFFERS: [GLenum; 1] = [gl::COLOR_ATTACHMENT0];

            }


            // let (mut mx, mut my) = window.get_cursor_pos();

            let mut stdout = ::std::io::stdout();

            while *running.lock().unwrap() {


                let start = ::std::time::Instant::now();

                window.swap_buffers().unwrap();

                unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                    gl::DrawBuffer(gl::BACK);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    gl::ClearColor(1.0,1.0,1.0,0.0);
                    gl::Viewport(0, 0, WIDTH as i32, HEIGHT as i32);
                }

                //
                //Handle movement events
                //

                let (w,h) = (WIDTH as f64, HEIGHT as f64);
                let s = w.min(h);

                //Scroll wheel scaling
                let dscroll = &mut *scroll_delta.lock().unwrap();

                scale *= ZOOM_SCROLL.powf(- *dscroll as f64);

                if *dscroll != 0.0 {
                    println!("{}", scale);
                }

                *dscroll = 0.0;


                //Mouse translation

                let (ref mut dmx, ref mut dmy) = &mut *cursor_delta.lock().unwrap();

                let cs = 4.0 * scale;
                let (dx, dy) = (cs * *dmx / s, -cs * *dmy / s);

                shader.center[0] += dx;
                shader.center[1] += dy;
                if *dmx != 0.0 || *dmy!=0.0 {
                    println!("({} {})", shader.center[0], shader.center[1]);
                }

                *dmx = 0.0;
                *dmy = 0.0;

                *shader.scale = scale;

                // mat = [
                //     [scale,0.0,0.0,0.0],
                //     [0.0,scale,0.0,0.0],
                //     [0.0,0.0,scale,0.0],
                //     [0.0,0.31999*scale,0.0,1.0]
                // ].into();

                #[allow(dead_code)]
                if DO_ZOOM {
                    scale *= ZOOM_RATE;
                    *shader.scale_hp *= HighPFloat::from_f64(ZOOM_RATE);
                } else {
                    let scale_f32 = scale as f32;
                    *shader.scale_hp = HighPFloat::from_f64(scale);
                    println!("{} {} {:?}", scale, shader.scale_hp.to_f32(), *shader.scale_hp)
                }

                // println!("{:?} {}", *shader.scale_hp, shader.scale_hp.to_f32());

                let (points, uv) = quad.as_attributes();

                #[allow(dead_code)]
                if !RECORD {
                    //draw on the screen
                    shader.draw(&mut context, DrawMode::Triangles, 6, points, uv);
                } else if RECORD {
                    unsafe {

                        //Draw to the recording framebuffer
                        #[allow(unused_variables)]
                        const BUFFERS: [GLenum; 1] = [gl::COLOR_ATTACHMENT0];
                        const CLEAR_COLOR: [GLfloat; 4] = [1.0,0.0,1.0,0.0];
                        gl::BindFramebuffer(gl::FRAMEBUFFER, fb);
                        gl::DrawBuffers(1, &BUFFERS[0] as *const GLenum);

                        gl::Viewport(0, 0, RECW as i32, RECH as i32);

                        gl::ClearBufferfi(gl::DEPTH_STENCIL, 0, 1.0, 0);
                        gl::ClearBufferfv(gl::COLOR, 0, &CLEAR_COLOR[0] as *const GLfloat);

                        gl::ClearColor(1.0,1.0,1.0,0.0);
                        gl::ClearDepth(1.0);
                        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                        shader.draw(&mut context, DrawMode::Triangles, 6, points, uv);

                        gl::Flush();
                        gl::Finish();

                        //read from the recording buffer
                        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fb);
                        gl::ReadBuffer(gl::COLOR_ATTACHMENT0);

                        //blit to the main screen
                        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
                        gl::BlitFramebuffer(
                            0,0, RECW as GLsizei, RECH as GLsizei,
                            0,0, WIDTH as GLsizei, HEIGHT as GLsizei,
                            gl::COLOR_BUFFER_BIT, gl::LINEAR
                        );

                        //download to RAM
                        gl::ReadnPixels(
                            0,0, RECW as GLsizei, RECH as GLsizei,
                            gl::RGB,gl::UNSIGNED_BYTE,
                            pixels.len() as GLsizei, &mut *pixels as *mut _ as *mut _
                        );

                        //output to stdout
                        stdout.write_all(&*pixels);
                    }
                }

                // println!("{:?}", ::std::time::Instant::now()-start);

                // ::std::thread::sleep(::std::time::Duration::from_millis(300));

            }

        }
    });

    let mut m1 = false;
    let mut mouse_pos = PhysicalPosition::new(0.0,0.0);

    events.run(move |event, _, action| {

        *action = ControlFlow::Wait;

        match event {

            Event::LoopDestroyed |
            Event::WindowEvent { event: WindowEvent::CloseRequested|WindowEvent::Destroyed, .. }
            =>{
                *running.lock().unwrap() = false;
                *action = ControlFlow::Exit;
            }

            Event::WindowEvent { event, .. } => match event {

                WindowEvent::MouseInput { state, button:MouseButton::Left, .. } => {
                    m1 = state==ElementState::Pressed;
                }

                WindowEvent::CursorMoved { position, .. } => {

                    if m1==true {
                        let (dx, dy) = (position.x-mouse_pos.x, position.y-mouse_pos.y);
                        let mut net_delta = cursor_delta.lock().unwrap();
                        net_delta.0 += dx;
                        net_delta.1 += dy;
                    }

                    mouse_pos = position;

                },

                WindowEvent::MouseWheel { delta:MouseScrollDelta::LineDelta(x,y), .. } => {
                    *scroll_delta.lock().unwrap() += y;
                }

                _ => ()

            }

            _ => ()
        }

    });

}
