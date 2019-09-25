use super::*;

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
