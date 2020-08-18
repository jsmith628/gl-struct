use super::*;

pub trait Transpose: GLSLType {
    type Output: Transpose<Output=Self>;
    fn transpose(self) -> Self::Output;
}

macro_rules! impl_trans {

    () => {};

    (@impl $mat:ident $trans:ident) => {
        impl Transpose for $mat {
            type Output = $trans;
            fn transpose(self) -> $trans {
                unsafe {
                    let mut dest = MaybeUninit::<$trans>::uninit();
                    for i in 0..self.len() {
                        for j in 0..self[i].len() {
                            dest.get_mut()[j][i] = self[j][i];
                        }
                    }
                    dest.assume_init()
                }
            }
        }
    };

    ($mat:ident::$dmat:ident $($rest:tt)*) => {
        impl_trans!(@impl $mat $mat);
        impl_trans!(@impl $dmat $dmat);
        impl_trans!($($rest)*);
    };

    ($mat:ident:$trans:ident $($rest:tt)*) => {
        impl_trans!(@impl $mat $trans);
        impl_trans!(@impl $trans $mat);
        impl_trans!($($rest)*);
    };
}

impl_trans!(
    mat2x2::dmat2x2 mat2x3: mat3x2   mat2x4: mat4x2
    dmat3x2:dmat2x3 mat3x3::dmat3x3  mat3x4: mat4x3
    dmat4x2:dmat2x4 dmat4x3:dmat3x4  mat4x4::dmat4x4
);

pub trait OuterProduct<RHS:GenFloatType>: GenFloatType {
    type Output: Mat;
    fn outer_product(self, rhs:RHS) -> <Self as OuterProduct<RHS>>::Output;
}

macro_rules! impl_outer_product {
    ($($ty1:ident:$ty2:ident:$mat:ident)*) => {
        $(
            impl OuterProduct<$ty2> for $ty1 {
                type Output = $mat;
                fn outer_product(self, rhs:$ty2) -> $mat {
                    unsafe {
                        let mut dest = MaybeUninit::<$mat>::uninit();
                        for i in 0..self.len() {
                            for j in 0..rhs.len() {
                                dest.get_mut()[j][i] = self[i] * rhs[j];
                            }
                        }
                        dest.assume_init()
                    }
                }
            }
        )*
    }
}

impl_outer_product!{
    vec2:vec2:mat2x2 vec2:vec3:mat3x2 vec2:vec4:mat4x2
    vec3:vec2:mat2x3 vec3:vec3:mat3x3 vec3:vec4:mat4x3
    vec4:vec2:mat2x4 vec4:vec3:mat3x4 vec4:vec4:mat4x4

    dvec2:dvec2:dmat2x2 dvec2:dvec3:dmat3x2 dvec2:dvec4:dmat4x2
    dvec3:dvec2:dmat2x3 dvec3:dvec3:dmat3x3 dvec3:dvec4:dmat4x3
    dvec4:dvec2:dmat2x4 dvec4:dvec3:dmat3x4 dvec4:dvec4:dmat4x4
}

#[allow(non_snake_case)]
pub fn matrixCompMult<M:Mat>(mut x:M, y:M) -> M {
    for i in 0..M::COUNT { *x.coord_mut(i) *= *y.coord(i); } x
}

#[allow(non_snake_case)]
pub fn outerProduct<V1:OuterProduct<V2>, V2:GenFloatType>(c:V1, r:V2) -> <V1 as OuterProduct<V2>>::Output {
    c.outer_product(r)
}

pub fn transpose<M:Mat>(m:M) -> <M as Transpose>::Output { m.transpose() }

pub fn determinant<M:Mat>(m:M) -> Scalar<M> {

    //yes, this is bad
    #[allow(clippy::many_single_char_names)]
    fn cofactor_det_3d<M:Mat>(m:M, i:usize, j:usize, k:usize, r:usize) -> Scalar<M> {
        m[i][i+r]*m[j][j+r]*m[k][k+r] + m[j][i+r]*m[k][j+r]*m[i][k+r] + m[k][i+r]*m[i][j+r]*m[j][k+r] -
        m[i][k+r]*m[j][j+r]*m[k][i+r] - m[j][k+r]*m[k][j+r]*m[i][i+r] - m[k][k+r]*m[i][j+r]*m[j][i+r]
    }

    if M::COUNT == 2 {
        m[0][0]*m[1][1] - m[0][1]*m[0][1]
    } else if M::COUNT==3 {
        cofactor_det_3d(m, 0,1,2, 0)
    } else if M::COUNT==4 {
        m[0][0]*cofactor_det_3d(m, 1,2,3, 1) -
        m[2][0]*cofactor_det_3d(m, 0,2,3, 1) +
        m[3][0]*cofactor_det_3d(m, 0,1,3, 1) -
        m[4][0]*cofactor_det_3d(m, 0,1,2, 1)
    } else {
        unimplemented!()
    }
}

pub fn inverse<M:SquareMat>(mut a:M) -> M {

    fn row_mul<M:SquareMat>(a: &mut M, b:&mut M, row:usize, col:usize, factor:Scalar<M>) {
        for k in col..(M::COUNT as usize) {
            a[k as usize][row] *= factor;
            b[k as usize][row] *= factor;
        }
    }

    fn row_sum<M:SquareMat>(a: &mut M, b:&mut M, row:usize, col:usize, factor:Scalar<M>, dest:usize) {
        for k in col..(M::COUNT as usize) {
            let (a_r, b_r) = (a[k][row], b[k][row]);
            a[k][dest] += factor*a_r;
            b[k][dest] += factor*b_r;
        }
    }

    fn swap_rows<M:SquareMat>(a: &mut M, b:&mut M, row1:usize, row2:usize, col:usize) {
        for k in col..(M::COUNT as usize) {
            let temp = (a[k][row1], b[k][row1]);
            a[k][row1] = a[k][row2];
            b[k][row1] = b[k][row2];
            a[k][row2] = temp.0;
            b[k][row2] = temp.1;
        }
    }

    let size = M::COUNT as usize;
    let mut b = One::one();

    //Gauss-Jordon elimination
    let mut i = 0;
    let mut j = 0;
    while i<size && j<size {

        if a[j][i] != Zero::zero() {
            let pivot = a[j][i];
            if pivot != One::one() { row_mul(&mut a, &mut b, i, j, Scalar::<M>::one()/pivot); }
            for k in 0..size {
                if k!=i && a[j][k] != Zero::zero() {
                    let pivot2 = -a[j][k];
                    row_sum(&mut a, &mut b, i, j, pivot2, k);
                }
            }

            i += 1;
            j += 1;
        } else {
            let mut cont = false;
            for k in (i+1)..size {
                if a[j][k] != Zero::zero() {
                    swap_rows(&mut a, &mut b, i, k, j);
                    cont = true;
                }
            }
            if !cont {j += 1;}
        }
    }

    b
}
