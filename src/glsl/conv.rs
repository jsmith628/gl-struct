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

macro_rules! impl_mat_conv {
    ($scalar:ident; $($mat:ident:$c:literal:$r:literal)*) => {
        $(
            impl From<[$scalar;$r*$c]> for $mat {
                fn from(arr:[$scalar;$r*$c]) -> $mat {
                    unsafe { transmute::<_,[[$scalar;$r];$c]>(arr) }.into()
                }
            }

            impl From<$mat> for [$scalar;$r*$c] {
                fn from(mat:$mat) -> [$scalar;$r*$c] {
                    unsafe { transmute::<[[$scalar;$r];$c],_>(mat.into()) }
                }
            }
        )*
    }
}

impl_mat_conv!{float;
    mat2x2:2:2 mat2x3:2:3 mat2x4:2:4
    mat3x2:3:2 mat3x3:3:3 mat3x4:3:4
    mat4x2:4:2 mat4x3:4:3 mat4x4:4:4
}

impl_mat_conv!{double;
    dmat2x2:2:2 dmat2x3:2:3 dmat2x4:2:4
    dmat3x2:3:2 dmat3x3:3:3 dmat3x4:3:4
    dmat4x2:4:2 dmat4x3:4:3 dmat4x4:4:4
}

macro_rules! impl_scalar_conv {
    ($scalar:ident; $($vec:ident)*; $($mat:ident)*) => {
        $(
            impl From<$scalar> for $vec {
                fn from(x:$scalar) -> $vec {
                    unsafe {
                        let mut dest = MaybeUninit::<$vec>::uninit();
                        for i in 0..dest.assume_init_ref().len() { dest.assume_init_mut()[i] = x; }
                        dest.assume_init()
                    }
                }
            }
        )*

        $(
            impl From<$scalar> for $mat {
                fn from(x:$scalar) -> $mat {
                    unsafe {
                        let mut dest = MaybeUninit::<$mat>::uninit();
                        for i in 0..dest.assume_init_ref().len() {
                            for j in 0..dest.assume_init_ref()[i].len() {
                                dest.assume_init_mut()[i][j] = if i==j { x } else { Zero::zero() };
                            }
                        }
                        dest.assume_init()
                    }
                }
            }
        )*
    }
}

impl_scalar_conv!(gl_bool; bvec2 bvec3 bvec4;);
impl_scalar_conv!(uint; uvec2 uvec3 uvec4;);
impl_scalar_conv!(int; ivec2 ivec3 ivec4;);
impl_scalar_conv!(float; vec2 vec3 vec4; mat2 mat3 mat4);
impl_scalar_conv!(double; dvec2 dvec3 dvec4; dmat2 dmat3 dmat4);

// trait CopyIntoSlice<T> {
//     const LEN:u32;
//     fn copy_into_slice(self, dest: &mut [T]);
// }
//
// impl<T:GenType<Component=T>,V:GenType<Component=T>> CopyIntoSlice<T> for V {
//     const LEN:u32 = V::COUNT;
//     default fn copy_into_slice(self, dest: &mut [T]) {
//         for i in 0..V::COUNT {
//             dest[i as usize] = *self.coord(i);
//         }
//     }
// }
//
// impl<T:GenType<Component=T>> CopyIntoSlice<T> for T {
//     fn copy_into_slice(self, dest: &mut [T]) { dest[0] = self; }
// }
//
// macro_rules! impl_copy_slice {
//     () => {};
//     (@next $T0:ident:$t0:ident $($T:ident:$t:ident)*) => {impl_copy_slice!($($T:$t)*);};
//     ($($T:ident:$t:ident)*) => {
//
//         impl<T, $($T:CopyIntoSlice<T>),*> CopyIntoSlice<T> for ($($T,)*) {
//             const LEN:u32 = 1;
//
//             #[allow(unused_assignments)]
//             fn copy_into_slice(self, dest: &mut [T]) {
//                 let ($($t,)*) = self;
//                 let mut index = 0;
//                 $(
//                     $t.copy_into_slice(&mut dest[index..]);
//                     index += $T::LEN as usize;
//                 )*
//             }
//         }
//
//         impl_copy_slice!(@next $($T:$t)*);
//     }
// }
//
// impl_copy_slice!(A:a B:b C:c D:d E:e F:f G:g H:h I:i J:j K:k L:l M:m N:n O:o P:p);
//
//
// macro_rules! impl_tuple_conv {
//
//     (2 @num $($tt:tt)*) => {impl_tuple_conv!([,,] $($tt)*);};
//     (3 @num $($tt:tt)*) => {impl_tuple_conv!([,,,] $($tt)*);};
//     (4 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,] $($tt)*);};
//     (5 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,,] $($tt)*);};
//     (6 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,,,] $($tt)*);};
//     (7 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,,,,] $($tt)*);};
//     (8 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,,,,,] $($tt)*);};
//     (9 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,,,,,,] $($tt)*);};
//     (10 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,,,,,,,] $($tt)*);};
//     (11 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,,,,,,,,] $($tt)*);};
//     (12 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,,,,,,,,,] $($tt)*);};
//     (13 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,,,,,,,,,,] $($tt)*);};
//     (14 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,,,,,,,,,,,] $($tt)*);};
//     (15 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,,,,,,,,,,,,] $($tt)*);};
//     (16 @num $($tt:tt)*) => {impl_tuple_conv!([,,,,,,,,,,,,,,,,] $($tt)*);};
//
//     ($n:tt $m:tt @map $($tt:tt)*) => {impl_tuple_conv!($n $m [] @map $($tt)*);};
//     ([] $m:tt [$($l:tt)*] @map $($tt:tt)*) => {impl_tuple_conv!([$($l)*] $($tt)*);};
//     ([$n0:tt $($n:tt)*] $m:tt [$($l:tt)*] @map $($tt:tt)*) => {
//         impl_tuple_conv!([$($n)*] $m [$m $($l)*] @map $($tt)*);
//     };
//
//     ($n:tt {$($code:tt)*} @swap_unwrap $($tt:tt)*) => { impl_tuple_conv!($($code)* $n $($tt)*); };
//
//     ($v1:ident:$v2:ident:$v3:ident:$v4:ident;) => {};
//     ($v1:ident:$v2:ident:$v3:ident:$v4:ident; $t0:ident:$n:tt:$m:tt $($rest:tt)*) => {
//         impl_tuple_conv!($m @num {$n @num} @swap_unwrap @map {$v1:$v2:$v3:$v4 {}} @swap_unwrap () @gen $t0 ($n*$m) @impl);
//         impl_tuple_conv!($v1:$v2:$v3:$v4; $($rest)*);
//     };
//     ($v1:ident:$v2:ident:$v3:ident:$v4:ident; $t0:ident:$n:tt $($rest:tt)*) => {
//         impl_tuple_conv!($n @num {$v1:$v2:$v3:$v4 {}} @swap_unwrap () @gen $t0 $n @impl);
//         impl_tuple_conv!($v1:$v2:$v3:$v4; $($rest)*);
//     };
//
//     ($v1:ident:$v2:ident:$v3:ident:$v4:ident {$($ty:tt)*} [,,,, $($n:tt)*] ($($tuple:tt)*) @gen $($tt:tt)*) => {
//         impl_tuple_conv!($v1:$v2:$v3:$v4 {$($ty)*}
//             [$($n)*] ($($tuple)* $v4,) @gen
//             [, $($n)*] ($($tuple)* $v3,) @gen
//             [,, $($n)*] ($($tuple)* $v2,) @gen
//             [,,, $($n)*] ($($tuple)* $v1,) @gen
//             $($tt)*
//         );
//     };
//
//     ($v1:ident:$v2:ident:$v3:ident:$v4:ident {$($ty:tt)*} [,,, $($n:tt)*] ($($tuple:tt)*) @gen $($tt:tt)*) => {
//         impl_tuple_conv!($v1:$v2:$v3:$v4  {$($ty)*}
//             [$($n)*] ($($tuple)* $v3,) @gen
//             [, $($n)*] ($($tuple)* $v2,) @gen
//             [,, $($n)*] ($($tuple)* $v1,) @gen
//             $($tt)*
//         );
//     };
//
//     ($v1:ident:$v2:ident:$v3:ident:$v4:ident {$($ty:tt)*} [,, $($n:tt)*] ($($tuple:tt)*) @gen $($tt:tt)*) => {
//         impl_tuple_conv!($v1:$v2:$v3:$v4  {$($ty)*}
//             [$($n)*] ($($tuple)* $v2,) @gen
//             [, $($n)*] ($($tuple)* $v1,) @gen
//             $($tt)*
//         );
//     };
//
//     ($v1:ident:$v2:ident:$v3:ident:$v4:ident {$($ty:tt)*} [, $($n:tt)*] ($($tuple:tt)*) @gen $($tt:tt)*) => {
//         impl_tuple_conv!($v1:$v2:$v3:$v4  {$($ty)*}
//             [$($n)*] ($($tuple)* $v1,) @gen
//             $($tt)*
//         );
//     };
//
//     ($v1:ident:$v2:ident:$v3:ident:$v4:ident {$($ty:tt)*} [[$($m:tt)*] $($n:tt)*] ($($tuple:tt)*) @gen $($tt:tt)*) => {
//         impl_tuple_conv!($v1:$v2:$v3:$v4 {$($ty)*} [$($m)* $($n)*] ($($tuple)*) @gen $($tt)*);
//     };
//
//     ($v1:ident:$v2:ident:$v3:ident:$v4:ident {$($ty:tt)*} [] ($($tuple:tt)*) @gen $($tt:tt)*) => {
//         impl_tuple_conv!($v1:$v2:$v3:$v4 {$($ty)* ($($tuple)*),} $($tt)*);
//     };
//
//     ($v1:ident:$v2:ident:$v3:ident:$v4:ident {$($ty:ty,)*} $name:ident $num:tt @impl ) => {
//         $(
//             impl From<$ty> for $name {
//                 fn from(obj:$ty) -> $name {
//                     unsafe {
//                         let mut dest = MaybeUninit::<[$v1;$num]>::uninit();
//                         obj.copy_into_slice(dest.assume_init_mut());
//                         return dest.assume_init().into();
//                     }
//                 }
//             }
//         )*
//     }
//
// }
//
// impl_tuple_conv!(gl_bool:bvec2:bvec3:bvec4; bvec2:2 bvec3:3 bvec4:4);
// impl_tuple_conv!(uint:uvec2:uvec3:uvec4; uvec2:2 uvec3:3 uvec4:4);
// impl_tuple_conv!(int:ivec2:ivec3:ivec4; ivec2:2 ivec3:3 ivec4:4);
//
// impl_tuple_conv!{float:vec2:vec3:vec4;
//     vec2:2 vec3:3 vec4:4
//     mat2x2:2:2 mat2x3:2:3 mat2x4:2:4
//     mat3x2:3:2 mat3x3:3:3 mat3x4:3:4
//     mat4x2:4:2 mat4x3:4:3 mat4x4:4:4
// }
//
// impl_tuple_conv!{double:dvec2:dvec3:dvec4;
//     dvec2:2 dvec3:3 dvec4:4
//     dmat2x2:2:2 dmat2x3:2:3 dmat2x4:2:4
//     dmat3x2:3:2 dmat3x3:3:3 dmat3x4:3:4
//     dmat4x2:4:2 dmat4x3:4:3 dmat4x4:4:4
// }
