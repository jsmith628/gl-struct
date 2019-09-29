use super::*;

pub trait GenType: GLSLType {
    type Component: GenType;

    fn coord(&self, i:uint) -> &Self::Component;
    fn coord_mut(&mut self, i:uint) -> &mut Self::Component;
    fn length(&self) -> uint;
}

pub trait GenBType = GenType<Component=gl_bool>;
pub trait GenUType = GenType<Component=uint> + GenOrdType;

pub trait GenEqType = GenType + PartialEq where <Self as GenType>::Component: PartialEq;

pub trait GenOrdType = GenEqType + Zero +
    Add<Output=Self> + AddAssign +
    Mul<Output=Self> + MulAssign +
    Mul<<Self as GenType>::Component, Output=Self> + MulAssign<<Self as GenType>::Component>
    where <Self as GenType>::Component: Mul<Self, Output=Self> + PartialOrd + num_traits::NumAssign;

pub trait GenSignType = GenOrdType +
    Add<Output=Self> + AddAssign +
    Mul<Output=Self> + MulAssign +
    Sub<Output=Self> + SubAssign + Neg<Output=Self> +
    Mul<<Self as GenType>::Component, Output=Self> + MulAssign<<Self as GenType>::Component>
    where <Self as GenType>::Component: Mul<Self, Output=Self> + num_traits::Signed;

pub trait GenIType = GenType<Component=int> + GenSignType;

pub trait GenFloatType = GenType + GenSignType +
    Div<Output=Self> + DivAssign +
    Div<<Self as GenType>::Component, Output=Self> + DivAssign<<Self as GenType>::Component>
    where <Self as GenType>::Component: num_traits::Float;

pub trait GenFType = GenType<Component=float> + GenFloatType;
pub trait GenDType = GenType<Component=double> + GenFloatType;

pub type Scalar<V> = <<V as GenType>::Component as GenType>::Component;

macro_rules! impl_gen_type_scalar {
    ($($ty:ident)*) => {
        $(
            impl GenType for $ty {
                type Component = $ty;
                fn coord(&self, _:uint) -> &Self::Component {self}
                fn coord_mut(&mut self, _:uint) -> &mut Self::Component {self}
                fn length(&self) -> uint {1}
            }
        )*
    }
}

impl_gen_type_scalar!(gl_bool uint int float double);

pub trait Transpose: GLSLType {
    type Output: Transpose<Output=Self>;
    fn transpose(self) -> Self::Output;
}

pub trait MatOps = GenType + Transpose + Zero +
    Add<Output=Self> + AddAssign +
    Sub<Output=Self> + SubAssign + Neg<Output=Self> +
    Mul<<<Self as GenType>::Component as GenType>::Component, Output=Self> +
    MulAssign<<<Self as GenType>::Component as GenType>::Component> +
    Div<<<Self as GenType>::Component as GenType>::Component, Output=Self> +
    DivAssign<<<Self as GenType>::Component as GenType>::Component> +
    Index<usize, Output = <Self as GenType>::Component> + IndexMut<usize>
where
    <Self as GenType>::Component: GenFloatType +
        Index<usize, Output=<<Self as GenType>::Component as GenType>::Component> +
        IndexMut<usize>;

pub trait Mat = MatOps where <Self as Transpose>::Output: MatOps;
pub trait SquareMat = Mat + Transpose<Output=Self> + One;

macro_rules! impl_mat {

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
        impl_mat!(@impl $mat $mat);
        impl_mat!(@impl $dmat $dmat);
        impl_mat!($($rest)*);
    };

    ($mat:ident:$trans:ident $($rest:tt)*) => {
        impl_mat!(@impl $mat $trans);
        impl_mat!(@impl $trans $mat);
        impl_mat!($($rest)*);
    };
}

impl_mat!(
    mat2x2::dmat2x2 mat2x3: mat3x2   mat2x4: mat4x2
    dmat3x2:dmat2x3 mat3x3::dmat3x3  mat3x4: mat4x3
    dmat4x2:dmat2x4 dmat4x3:dmat3x4  mat4x4::dmat4x4
);


pub trait CrossProduct: GenFloatType {
    fn cross(self, rhs:Self) -> Self;
}

macro_rules! impl_cross {
    ($ty:ident) => {
        impl CrossProduct for $ty {
            fn cross(self, rhs:Self) -> Self {
                $ty {
                    value: [
                        self[1]*rhs[2] - rhs[1]*self[2],
                        self[2]*rhs[0] - rhs[2]*self[0],
                        self[0]*rhs[1] - rhs[0]*self[1]
                    ]
                }
            }
        }
    }
}

impl_cross!(vec3);
impl_cross!(dvec3);

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


macro_rules! component_wrapping_fn {
    ($gen_type:ident; $($name:ident:$fun:ident)*) => {
        $(
            pub fn $name<V:$gen_type>(mut v:V) -> V {
                for i in 0..v.length() { *v.coord_mut(i as u32) = v.coord(i as u32).$fun(); }
                return v;
            }
        )*
    }
}

component_wrapping_fn!{GenFType;
    radians:to_radians degrees:to_degrees
    sin:sin cos:cos tan:tan asin:asin acos:acos atan:atan
    sinh:sinh cosh:cosh tanh:tanh asinh:asinh acosh:acosh atanh:atanh

    exp:exp log:ln exp2:exp2 log2:log2

}

component_wrapping_fn!{GenFloatType;
    floor:floor ceil:ceil trunc:trunc fract:fract round:round sqrt:sqrt
}

component_wrapping_fn!{GenSignType; abs:abs sign:signum}

//
//Pack and Unpack functions
//

//
//Geometric Functions
//

pub fn length<V:GenFloatType>(v:V) -> V::Component { dot(v,v).sqrt() }

pub fn distance<V:GenFloatType>(p0:V, p1:V) -> V::Component { length(p0-p1) }

pub fn dot<V:GenFloatType>(x:V, y:V) -> V::Component {
    (0..x.length()).fold(num_traits::Zero::zero(), |d,i| d+ *x.coord(i)**y.coord(i))
}

pub fn cross<V:CrossProduct>(x:V, y:V) -> V { x.cross(y) }

pub fn normalize<V:GenFloatType>(x:V) -> V { x / length(x) }

#[allow(non_snake_case)]
pub fn faceForward<V:GenFloatType>(N:V, I:V, Nref:V) -> V {
    if dot(Nref,I) < Zero::zero() { N } else { -N }
}

#[allow(non_snake_case)]
pub fn reflect<V:GenFloatType>(I:V, N:V) -> V {
    I - ((V::Component::one()+One::one())*dot(N,I))*N
}

#[allow(non_snake_case)]
pub fn refract<V:GenFloatType>(I:V, N:V, eta: V::Component) -> V {
    let k:V::Component = V::Component::one() - eta*eta*(V::Component::one() - dot(N,I)*dot(N,I));
    if k < Zero::zero() {
        Zero::zero()
    } else {
        eta*I - (eta*dot(N,I) + k.sqrt())*N
    }
}

//
//Matrix Functions
//

#[allow(non_snake_case)]
pub fn matrixCompMult<M:Mat>(mut x:M, y:M) -> M {
    for i in 0..x.length() { *x.coord_mut(i) *= *y.coord(i); } x
}

#[allow(non_snake_case)]
pub fn outerProduct<V1:OuterProduct<V2>, V2:GenFloatType>(c:V1, r:V2) -> <V1 as OuterProduct<V2>>::Output {
    c.outer_product(r)
}

pub fn transpose<M:Mat>(m:M) -> <M as Transpose>::Output { m.transpose() }

pub fn determinant<M:Mat>(m:M) -> Scalar<M> {

    fn cofactor_det_3d<M:Mat>(m:M, i:usize, j:usize, k:usize, r:usize) -> Scalar<M> {
        m[i][i+r]*m[j][j+r]*m[k][k+r] + m[j][i+r]*m[k][j+r]*m[i][k+r] + m[k][i+r]*m[i][j+r]*m[j][k+r] -
        m[i][k+r]*m[j][j+r]*m[k][i+r] - m[j][k+r]*m[k][j+r]*m[i][i+r] - m[k][k+r]*m[i][j+r]*m[j][i+r]
    }

    if m.length() == 2 {
        m[0][0]*m[1][1] - m[0][1]*m[0][1]
    } else if m.length()==3 {
        cofactor_det_3d(m, 0,1,2, 0)
    } else if m.length()==4 {
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
        for k in col..(a.length() as usize) {
            a[k as usize][row] *= factor;
            b[k as usize][row] *= factor;
        }
    }

    fn row_sum<M:SquareMat>(a: &mut M, b:&mut M, row:usize, col:usize, factor:Scalar<M>, dest:usize) {
        for k in col..(a.length() as usize) {
            let (a_r, b_r) = (a[k][row], b[k][row]);
            a[k][dest] += factor*a_r;
            b[k][dest] += factor*b_r;
        }
    }

    fn swap_rows<M:SquareMat>(a: &mut M, b:&mut M, row1:usize, row2:usize, col:usize) {
        for k in col..(a.length() as usize) {
            let temp = (a[k][row1], b[k][row1]);
            a[k][row1] = a[k][row2];
            b[k][row1] = b[k][row2];
            a[k][row2] = temp.0;
            b[k][row2] = temp.1;
        }
    }

    let size = a.length() as usize;
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

    return b;
}

//
//Vector Relational Functions
//

pub fn any<V:GenBType>(x:V) -> gl_bool {
    for i in 0..x.length() { if bool::from(*x.coord(i)) { return true.into(); } }
    return false.into();
}

pub fn all<V:GenBType>(x:V) -> gl_bool {
    for i in 0..x.length() { if bool::from(!(*x.coord(i))) { return false.into(); } }
    return true.into();
}

pub fn not<V:GenBType>(mut x:V) -> V {
    for i in 0..x.length() { *x.coord_mut(i) = !*x.coord(i); }
    return x;
}
