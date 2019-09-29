use super::*;

macro_rules! impl_index {
    ($($ty:ident:$item:ident)*) => {
        $(

            impl GenType for $ty {
                type Component = $item;

                fn coord(&self, i:uint) -> &$item {&self[i as usize]}
                fn coord_mut(&mut self, i:uint) -> &mut $item {&mut self[i as usize]}
                fn length(&self) -> uint {self.len() as uint}
            }

            impl $ty {
                pub fn len(&self) -> usize {self.value.len()}
                pub fn iter(&self) -> ::std::slice::Iter<$item> {self.value.iter()}
                pub fn iter_mut(&mut self) -> ::std::slice::IterMut<$item> {self.value.iter_mut()}
            }

            impl<I:SliceIndex<[$item]>> Index<I> for $ty {
                type Output = I::Output;
                fn index(&self, i:I) -> &Self::Output { &self.value[i] }
            }

            impl<I:SliceIndex<[$item]>> IndexMut<I> for $ty {
                fn index_mut(&mut self, i:I) -> &mut Self::Output { &mut self.value[i] }
            }

        )*
    }
}

impl_index!(
    bvec2:gl_bool bvec3:gl_bool bvec4:gl_bool
    uvec2:uint uvec3:uint uvec4:uint
    ivec2:int ivec3:int ivec4:int

    vec2:float vec3:float vec4:float
    mat2x2:vec2 mat3x2:vec2 mat4x2:vec2
    mat2x3:vec3 mat3x3:vec3 mat4x3:vec3
    mat2x4:vec4 mat3x4:vec4 mat4x4:vec4

    dvec2:double dvec3:double dvec4:double
    dmat2x2:dvec2 dmat3x2:dvec2 dmat4x2:dvec2
    dmat2x3:dvec3 dmat3x3:dvec3 dmat4x3:dvec3
    dmat2x4:dvec4 dmat3x4:dvec4 dmat4x4:dvec4
);

macro_rules! impl_entrywise_op {

    ({} $($ty:tt)*) => {};

    (
        {$Trait:ident.$fun:ident $op:tt $TraitAssign:ident.$fun_assign:ident $op2:tt, $($others:tt)*}
        $($ty:ident:$scalar:ident)*
    ) => {
        $(

            impl $TraitAssign for $ty {
                fn $fun_assign(&mut self, rhs:$ty) {
                    for i in 0..self.len() { self[i] $op2 rhs[i]; }
                }
            }

            impl $TraitAssign<$scalar> for $ty {
                fn $fun_assign(&mut self, rhs:$scalar) {
                    for i in 0..self.len() { self[i] $op2 rhs; }
                }
            }

            impl $Trait for $ty {
                type Output = Self;
                fn $fun(mut self, rhs:Self) -> Self { self $op2 rhs; self }
            }

            impl $Trait<$ty> for $scalar {
                type Output = $ty;
                fn $fun(self, rhs:$ty) -> $ty { rhs $op self }
            }

            impl $Trait<$scalar> for $ty {
                type Output = Self;
                fn $fun(mut self, rhs:$scalar) -> Self { self $op2 rhs; self }
            }
        )*

        impl_entrywise_op!({$($others)*} $($ty:$scalar)*);
    };
}

impl_entrywise_op!{
    {
        Add.add + AddAssign.add_assign +=,
        Sub.sub - SubAssign.sub_assign -=,
        Div.div / DivAssign.div_assign /=,
    }
    uvec2:uint uvec3:uint uvec4:uint
    ivec2:int  ivec3:int  ivec4:int

    vec2:float vec3:float vec4:float
    mat2x2:float mat3x2:float mat4x2:float
    mat2x3:float mat3x3:float mat4x3:float
    mat2x4:float mat3x4:float mat4x4:float

    dvec2:double dvec3:double dvec4:double
    dmat2x2:double dmat3x2:double dmat4x2:double
    dmat2x3:double dmat3x3:double dmat4x3:double
    dmat2x4:double dmat3x4:double dmat4x4:double
}

impl_entrywise_op!{
    { Mul.mul * MulAssign.mul_assign *=, }
    uvec2:uint   uvec3:uint   uvec4:uint
    ivec2:int    ivec3:int    ivec4:int
    vec2:float   vec3:float   vec4:float
    dvec2:double dvec3:double dvec4:double
}

impl_entrywise_op!{
    {
        Rem.rem % RemAssign.rem_assign %=,
        Shl.shl << ShlAssign.shl_assign <<=,
        Shr.shr >> ShrAssign.shr_assign >>=,
        BitAnd.bitand & BitAndAssign.bitand_assign &=,
        BitOr.bitor | BitOrAssign.bitor_assign |=,
        BitXor.bitxor ^ BitXorAssign.bitxor_assign ^=,
    }
    uvec2:uint uvec3:uint uvec4:uint
    ivec2:int  ivec3:int  ivec4:int
}

macro_rules! impl_unary_op {
    ($Trait:ident.$fun:ident $op:tt; $($ty:ident)*) => {
        $(
            impl $Trait for $ty {
                type Output = $ty;
                fn $fun(mut self) -> $ty {
                    for i in 0..self.len() {self[i] = $op self[i];}
                    self
                }
            }
        )*
    }
}

impl_unary_op!{ Neg.neg -;
    ivec2 ivec3 ivec4

    vec2 vec3 vec4
    mat2x2 mat3x2 mat4x2
    mat2x3 mat3x3 mat4x3
    mat2x4 mat3x4 mat4x4

    dvec2 dvec3 dvec4
    dmat2x2 dmat3x2 dmat4x2
    dmat2x3 dmat3x3 dmat4x3
    dmat2x4 dmat3x4 dmat4x4
}

impl_unary_op!{ Not.not !;
    uvec2 uvec3 uvec4
    ivec2 ivec3 ivec4
}

macro_rules! impl_zero {
    ($($ty:ident)*) => {
        $(
            impl Zero for $ty {
                fn zero() -> Self {
                    unsafe {
                        let mut dest = MaybeUninit::<Self>::uninit();
                        dest.get_mut().set_zero();
                        dest.assume_init()
                    }
                }

                fn set_zero(&mut self) { for i in 0..self.len() { self[i].set_zero(); } }

                fn is_zero(&self) -> bool{
                    for i in 0..self.len() { if !self[i].is_zero() {return false;} }
                    true
                }
            }
        )*
    }
}

impl_zero!(
    uvec2 uvec3 uvec4 ivec2 ivec3 ivec4 vec2 vec3 vec4 dvec2 dvec3 dvec4
    mat2x2 mat3x2 mat4x2 mat2x3 mat3x3 mat4x3 mat2x4 mat3x4 mat4x4
    dmat2x2 dmat3x2 dmat4x2 dmat2x3 dmat3x3 dmat4x3 dmat2x4 dmat3x4 dmat4x4
);

macro_rules! impl_one {
    ($($ty:ident)*) => {
        $(
            impl One for $ty {
                fn one() -> Self {
                    unsafe {
                        let mut dest = MaybeUninit::<Self>::uninit();
                        dest.get_mut().set_one();
                        dest.assume_init()
                    }
                }

                fn set_one(&mut self) {
                    for i in 0..self.len() {
                        for j in 0..self[i].len() {
                            self[i][j] = if i==j {1.0} else {0.0};
                        }
                    }
                }

                fn is_one(&self) -> bool{
                    for i in 0..self.len() {
                        for j in 0..self[i].len() {
                            if (i==j && !self[i][j].is_zero()) || (i!=j && !self[i][j].is_one()) {
                                return false;
                            }
                        }
                    }
                    true
                }
            }
        )*
    }
}

impl_one!(mat2x2 mat3x3 mat4x4 dmat2x2 dmat3x3 dmat4x4);

macro_rules! impl_matrix_scalar_mul {
    ($scalar:ident; $($mat:ident)*) => {
        $(
            impl Mul<$scalar> for $mat {
                type Output = $mat;
                fn mul(mut self, rhs:$scalar) -> $mat { self *= rhs; self}
            }

            impl MulAssign<$scalar> for $mat {
                fn mul_assign(&mut self, rhs:$scalar) {
                    for i in 0..self.len() { self[i] *= rhs; }
                }
            }
        )*
    }
}

impl_matrix_scalar_mul!(float; mat2x2 mat2x3 mat2x4 mat3x2 mat3x3 mat3x4 mat4x2 mat4x3 mat4x4);
impl_matrix_scalar_mul!(double; dmat2x2 dmat2x3 dmat2x4 dmat3x2 dmat3x3 dmat3x4 dmat4x2 dmat4x3 dmat4x4);

macro_rules! impl_matrix_vector_mul {

    () => {};

    ($mat:ident: :$vec:ident $($rest:tt)*) => {
        impl MulAssign<$mat> for $vec {
            fn mul_assign(&mut self, rhs:$mat) { *self = *self * rhs; }
        }

        impl_matrix_vector_mul!($mat:$vec:$vec $($rest)*);
    };

    ($mat:ident:$vec1:ident:$vec2:ident $($rest:tt)*) => {
        impl Mul<$vec1> for $mat {
            type Output = $vec2;
            fn mul(self, rhs:$vec1) -> $vec2 {
                let mut out = rhs[0] * self[0];
                for i in 1..rhs.len() {out += rhs[i] * self[i];}
                out
            }
        }

        impl Mul<$mat> for $vec2 {
            type Output = $vec1;
            fn mul(self, rhs:$mat) -> $vec1 {
                unsafe {
                    let mut out = MaybeUninit::<$vec1>::uninit();
                    for i in 0..rhs[0].len() {
                        out.get_mut()[i] = self[0] * rhs[i][0];
                        for j in 1..rhs.len() {
                            out.get_mut()[i] += self[j] * rhs[i][j];
                        }
                    }
                    out.assume_init()
                }
            }
        }

        impl_matrix_vector_mul!($($rest)*);
    };


}

impl_matrix_vector_mul!{
    mat2x2:    :vec2 mat2x3:vec2:vec3 mat2x4:vec2:vec4
    mat3x2:vec3:vec2 mat3x3:    :vec3 mat3x4:vec3:vec4
    mat4x2:vec4:vec2 mat4x3:vec4:vec3 mat4x4:    :vec4

    dmat2x2:     :dvec2 dmat2x3:dvec2:dvec3 dmat2x4:dvec2:dvec4
    dmat3x2:dvec3:dvec2 dmat3x3:     :dvec3 dmat3x4:dvec3:dvec4
    dmat4x2:dvec4:dvec2 dmat4x3:dvec4:dvec3 dmat4x4:     :dvec4
}


macro_rules! impl_matrix_mul {
    () => {};

    ($mat:ident: : $($rest:tt)*) => {
        impl MulAssign for $mat {
            fn mul_assign(&mut self, rhs:Self) { *self = *self * rhs; }
        }

        impl_matrix_mul!($mat:$mat:$mat $($rest)*);
    };

    ($mat1:ident:$mat2:ident:$out:ident $($rest:tt)*) => {
        impl Mul<$mat2> for $mat1 {
            type Output = $out;
            fn mul(self, rhs:$mat2) -> $out {
                unsafe {
                    let mut out = MaybeUninit::<$out>::uninit();
                    for i in 0..rhs.len() {
                        out.get_mut()[i] = self * rhs[i];
                    }
                    out.assume_init()
                }
            }
        }

        impl_matrix_mul!($($rest)*);
    }
}

impl_matrix_mul!{

    mat2x2:      :       mat2x2:mat3x2:mat3x2 mat2x2:mat4x2:mat4x2
    mat2x3:mat2x2:mat2x3 mat2x3:mat3x2:mat3x3 mat2x3:mat4x2:mat4x3
    mat2x4:mat2x2:mat2x4 mat2x4:mat3x2:mat3x4 mat2x4:mat4x2:mat4x4

    mat3x2:mat2x3:mat2x2 mat3x2:mat3x3:mat3x2 mat3x2:mat4x3:mat4x2
    mat3x3:mat2x3:mat2x3 mat3x3:      :       mat3x3:mat4x3:mat4x3
    mat3x4:mat2x3:mat2x4 mat3x4:mat3x3:mat3x4 mat3x4:mat4x3:mat4x4

    mat4x2:mat2x4:mat2x2 mat4x2:mat3x4:mat3x2 mat4x2:mat4x4:mat4x2
    mat4x3:mat2x4:mat2x3 mat4x3:mat3x4:mat3x3 mat4x3:mat4x4:mat4x3
    mat4x4:mat2x4:mat2x4 mat4x4:mat3x4:mat3x4 mat4x4:      :

    dmat2x2:       :        dmat2x2:dmat3x2:dmat3x2 dmat2x2:dmat4x2:dmat4x2
    dmat2x3:dmat2x2:dmat2x3 dmat2x3:dmat3x2:dmat3x3 dmat2x3:dmat4x2:dmat4x3
    dmat2x4:dmat2x2:dmat2x4 dmat2x4:dmat3x2:dmat3x4 dmat2x4:dmat4x2:dmat4x4

    dmat3x2:dmat2x3:dmat2x2 dmat3x2:dmat3x3:dmat3x2 dmat3x2:dmat4x3:dmat4x2
    dmat3x3:dmat2x3:dmat2x3 dmat3x3:       :        dmat3x3:dmat4x3:dmat4x3
    dmat3x4:dmat2x3:dmat2x4 dmat3x4:dmat3x3:dmat3x4 dmat3x4:dmat4x3:dmat4x4

    dmat4x2:dmat2x4:dmat2x2 dmat4x2:dmat3x4:dmat3x2 dmat4x2:dmat4x4:dmat4x2
    dmat4x3:dmat2x4:dmat2x3 dmat4x3:dmat3x4:dmat3x3 dmat4x3:dmat4x4:dmat4x3
    dmat4x4:dmat2x4:dmat2x4 dmat4x4:dmat3x4:dmat3x4 dmat4x4:       :

}
