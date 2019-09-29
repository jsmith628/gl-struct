use super::*;

use num_traits::cast::AsPrimitive;

macro_rules! impl_arr_conv {
    ($vec1:ident $vec2:ident $vec3:ident $vec4:ident; $($to:ty),*) => {
        impl<T:Into<$vec1>> From<[T; 2]> for $vec2 {
            fn from(arr: [T; 2]) -> Self {
                let [a, b] = arr;
                $vec2{value:[a.into(), b.into()]}
            }
        }

        impl<T:Into<$vec1>> From<[T; 3]> for $vec3 {
            fn from(arr: [T; 3]) -> Self {
                let [a, b, c] = arr;
                $vec3{value:[a.into(), b.into(), c.into()]}
            }
        }

        impl<T:Into<$vec1>> From<[T; 4]> for $vec4 {
            fn from(arr: [T; 4]) -> Self {
                let [a, b, c, d] = arr;
                $vec4{value:[a.into(), b.into(), c.into(), d.into()]}
            }
        }

        $(
            impl From<$vec2> for [$to; 2] {
                fn from(arr: $vec2) -> Self { [arr[0].into(), arr[1].into()] }
            }

            impl From<$vec3> for [$to; 3] {
                fn from(arr: $vec3) -> Self { [arr[0].into(), arr[1].into(), arr[2].into()] }
            }

            impl From<$vec4> for [$to; 4] {
                fn from(arr: $vec4) -> Self { [arr[0].into(), arr[1].into(), arr[2].into(), arr[3].into()] }
            }
        )*

    }
}

impl_arr_conv!(gl_bool bvec2 bvec3 bvec4; bool, GLboolean);
impl_arr_conv!(uint    uvec2 uvec3 uvec4; u32, u64, i64, u128, i128, f64);
impl_arr_conv!(int     ivec2 ivec3 ivec4; i32, i64, i128, f64);
impl_arr_conv!(float    vec2  vec3  vec4; f32, f64);
impl_arr_conv!(double  dvec2 dvec3 dvec4; f64);

impl_arr_conv!(vec2 mat2x2 mat3x2 mat4x2; vec2, dvec2, [f32;2], [f64;2]);
impl_arr_conv!(vec3 mat2x3 mat3x3 mat4x3; vec3, dvec3, [f32;3], [f64;3]);
impl_arr_conv!(vec4 mat2x4 mat3x4 mat4x4; vec4, dvec4, [f32;4], [f64;4]);

impl_arr_conv!(dvec2 dmat2x2 dmat3x2 dmat4x2; dvec2, [f64;2]);
impl_arr_conv!(dvec3 dmat2x3 dmat3x3 dmat4x3; dvec3, [f64;3]);
impl_arr_conv!(dvec4 dmat2x4 dmat3x4 dmat4x4; dvec4, [f64;4]);

macro_rules! impl_conv {
    ($($ty1:ident:$ty2:ident)*) => {
        $(
            impl From<$ty2> for $ty1 {
                fn from(obj:$ty2) -> $ty1 { $ty1 { value: obj.into() } }
            }

            impl AsPrimitive<$ty1> for $ty2 {
                fn as_(self) -> $ty1 { $ty1 { value: self.into() } }
            }
        )*
    }
}

impl_conv!{

    dvec2:uvec2 dvec3:uvec3 dvec4:uvec4
    dvec2:ivec2 dvec3:ivec3 dvec4:ivec4
    dvec2:vec2  dvec3:vec3  dvec4:vec4

    dmat2x2:mat2x2 dmat3x2:mat3x2 dmat4x2:mat4x2
    dmat2x3:mat2x3 dmat3x3:mat3x3 dmat4x3:mat4x3
    dmat2x4:mat2x4 dmat3x4:mat3x4 dmat4x4:mat4x4

}

macro_rules! implicit_conv {

    () => {};

    (
        $avec2:ident as $bvec2:ident,
        $avec3:ident as $bvec3:ident,
        $avec4:ident as $bvec4:ident; $($rest:tt)*
    ) => {
        impl AsPrimitive<$bvec2> for $avec2 {
            fn as_(self) -> $bvec2 { $bvec2 { value: [self[0].as_(), self[1].as_()] } }
        }

        impl AsPrimitive<$bvec3> for $avec3 {
            fn as_(self) -> $bvec3 {
                $bvec3 { value: [self[0].as_(), self[1].as_(), self[2].as_()] }
            }
        }

        impl AsPrimitive<$bvec4> for $avec4 {
            fn as_(self) -> $bvec4 {
                $bvec4 { value: [self[0].as_(), self[1].as_(), self[2].as_(), self[3].as_()] }
            }
        }
        implicit_conv!($($rest)*);
    }
}

implicit_conv! {
    ivec2 as ivec2, ivec3 as ivec3, ivec4 as ivec4;
    ivec2 as uvec2, ivec3 as uvec3, ivec4 as uvec4;
    ivec2 as vec2,  ivec3 as vec3,  ivec4 as vec4;

    uvec2 as uvec2, uvec3 as uvec3, uvec4 as uvec4;
    uvec2 as vec2,  uvec3 as vec3,  uvec4 as vec4;

    vec2 as vec2, vec3 as vec3, vec4 as vec4;
    mat2x2 as mat2x2, mat3x2 as mat3x2, mat4x2 as mat4x2;
    mat2x3 as mat2x3, mat3x3 as mat3x3, mat4x3 as mat4x3;
    mat2x4 as mat2x4, mat3x4 as mat3x4, mat4x4 as mat4x4;
}
