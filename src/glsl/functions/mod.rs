use super::*;

pub trait GenType: GLSLType {
    type Component: GenType;
    const COUNT: uint;

    fn coord(&self, i:uint) -> &Self::Component;
    fn coord_mut(&mut self, i:uint) -> &mut Self::Component;
    fn count(&self) -> uint {Self::COUNT}
}


macro_rules! impl_gen_type {
    ($($ty:ident:$item:ident:$COUNT:tt)*) => {
        $(
            impl GenType for $ty {
                type Component = $item;
                const COUNT: uint = $COUNT;
                fn coord(&self, i:uint) -> &$item {&self[i as usize]}
                fn coord_mut(&mut self, i:uint) -> &mut $item {&mut self[i as usize]}
            }
        )*
    }
}

impl_gen_type!{
    bvec2:gl_bool:2 bvec3:gl_bool:3 bvec4:gl_bool:4
    uvec2:uint:2 uvec3:uint:3 uvec4:uint:4
    ivec2:int:2 ivec3:int:3 ivec4:int:4

    vec2:float:2 vec3:float:3 vec4:float:4
    mat2x2:vec2:2 mat3x2:vec2:3 mat4x2:vec2:4
    mat2x3:vec3:2 mat3x3:vec3:3 mat4x3:vec3:4
    mat2x4:vec4:2 mat3x4:vec4:3 mat4x4:vec4:4

    dvec2:double:2 dvec3:double:3 dvec4:double:4
    dmat2x2:dvec2:2 dmat3x2:dvec2:3 dmat4x2:dvec2:4
    dmat2x3:dvec3:2 dmat3x3:dvec3:3 dmat4x3:dvec3:4
    dmat2x4:dvec4:2 dmat3x4:dvec4:3 dmat4x4:dvec4:4
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
                const COUNT:uint = 1;
                fn coord(&self, _:uint) -> &Self::Component {self}
                fn coord_mut(&mut self, _:uint) -> &mut Self::Component {self}
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
