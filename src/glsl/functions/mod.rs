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

pub trait GenFamily: GenType {
    type BVec: GenBType + GenFamily<BVec = Self::BVec>;
    type UVec: GenUType + GenFamily<UVec = Self::UVec>;
    type IVec: GenIType + GenFamily<IVec = Self::IVec>;
    type Vec: GenFType + GenFamily<Vec = Self::Vec>;
    type DVec: GenDType + GenFamily<DVec = Self::DVec>;
}

macro_rules! impl_gen_fam {

    (@impl $name:ident $bvec:ident:$uvec:ident:$ivec:ident:$vec:ident:$dvec:ident) => {
        impl GenFamily for $name {
            type BVec = $bvec; type UVec = $uvec; type IVec = $ivec;
            type Vec = $vec; type DVec = $dvec;
        }
    };

    ($($bvec:ident:$uvec:ident:$ivec:ident:$vec:ident:$dvec:ident)*) => {
        $(
            impl_gen_fam!(@impl $bvec $bvec:$uvec:$ivec:$vec:$dvec);
            impl_gen_fam!(@impl $uvec $bvec:$uvec:$ivec:$vec:$dvec);
            impl_gen_fam!(@impl $ivec $bvec:$uvec:$ivec:$vec:$dvec);
            impl_gen_fam!(@impl $vec $bvec:$uvec:$ivec:$vec:$dvec);
            impl_gen_fam!(@impl $dvec $bvec:$uvec:$ivec:$vec:$dvec);
        )*
    }
}

impl_gen_fam!{
    gl_bool:uint:int:float:double
    bvec2:uvec2:ivec2:vec2:dvec2
    bvec3:uvec3:ivec3:vec3:dvec3
    bvec4:uvec4:ivec4:vec4:dvec4
}

pub trait GenVec: GenFamily {}

macro_rules! impl_gen_vec {
    ($($vec:ident)*) => { $(impl GenVec for $vec {})* }
}

impl_gen_vec!{
    bvec2 uvec2 ivec2 vec2 dvec2
    bvec3 uvec3 ivec3 vec3 dvec3
    bvec4 uvec4 ivec4 vec4 dvec4
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

pub use self::misc::*;
pub use self::geometry::*;
pub use self::matrix::*;
pub use self::relational::*;

mod misc;
mod geometry;
mod matrix;
mod relational;
