
use super::*;
use object::buffer::GPUCopy;
use glsl::*;
use std::mem::transmute;

pub unsafe trait AttribFormat: Sized + Clone + Copy + PartialEq + Eq + Hash + Debug {
    fn size(self) -> usize;
    fn attrib_count(self) -> usize {1}
    unsafe fn bind_attribute(self, attr_id: GLuint, stride: usize, offset: usize);
    unsafe fn set_attribute(self, attr_id: GLuint, data: *const GLvoid);
}

pub trait AttributeData<T:GLSLType>: Sized + Copy {
    fn format() -> T::AttributeFormat;
}

pub trait AttributeValue<T:GLSLType>: GPUCopy { fn format(&self) -> T::AttributeFormat; }

impl<A:AttributeData<T>, T:GLSLType> AttributeValue<T> for A {
    #[inline] fn format(&self) -> T::AttributeFormat {A::format()}
}

pub type IntFormat = IntType;

unsafe impl AttribFormat for IntFormat {

    #[inline] fn size(self) -> usize { self.size_of() }

    #[inline]
    unsafe fn bind_attribute(self, attr_id: GLuint, stride: usize, offset: usize){
        gl::VertexAttribIPointer(attr_id, 1, self.into(), stride as GLsizei, transmute(offset as GLintptr));
    }

    #[inline]
    unsafe fn set_attribute(self, attr_id: GLuint, data: *const GLvoid){
        FloatFormat::FromInt(self, false).set_attribute(attr_id, data);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FloatFormat {
    Float(FloatType),
    FromInt(IntFormat, bool),
    Fixed,
    Double
}

impl FloatFormat {
    fn gl_type(self) -> GLenum {
        match self {
            FloatFormat::Float(ty) => ty.into(),
            FloatFormat::FromInt(f, _) => f.into(),
            FloatFormat::Double => gl::DOUBLE,
            FloatFormat::Fixed => gl::FIXED
        }
    }

    #[inline]
    fn normalized(self) -> bool {
        match self {
            FloatFormat::FromInt(_, b) => b,
            _ => false
        }
    }

}

unsafe impl AttribFormat for FloatFormat {
    fn size(self) -> usize {
        match self {
            FloatFormat::Float(ty) => ty.size_of(),
            FloatFormat::Fixed => 4,
            FloatFormat::Double => 8,
            FloatFormat::FromInt(f, _) => f.size()
        }
    }

    #[inline]
    unsafe fn bind_attribute(self, attr_id: GLuint, stride: usize, offset: usize){
        gl::VertexAttribPointer(attr_id, 1, self.gl_type(), self.normalized() as GLboolean, stride as GLsizei, transmute(offset as GLintptr));
    }

    #[inline]
    unsafe fn set_attribute(self, attr_id: GLuint, data: *const GLvoid){
        VecFormat::VecN(self, 1).set_attribute(attr_id, data);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DoubleFormat;

unsafe impl AttribFormat for DoubleFormat {
    #[inline] fn size(self) -> usize { 8 }

    #[inline]
    unsafe fn bind_attribute(self, attr_id: GLuint, stride: usize, offset: usize){
        gl::VertexAttribLPointer(attr_id, 1, gl::DOUBLE, stride as GLsizei, transmute(offset as GLintptr));
    }

    #[inline]
    unsafe fn set_attribute(self, attr_id: GLuint, data: *const GLvoid){
        DVecFormat::DVecN(1).set_attribute(attr_id, data);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[allow(non_camel_case_types)]
pub enum VecFormat {
    VecN(FloatFormat, usize),
    Int2_10_10_10Rev(bool),
    UInt2_10_10_10Rev(bool),
    UInt10F_11F_11FRev(bool)
}

impl VecFormat {
    fn gl_type(self) -> GLenum {
        match self {
            VecFormat::VecN(f, _) => f.gl_type(),
            VecFormat::Int2_10_10_10Rev(_) => gl::INT_2_10_10_10_REV,
            VecFormat::UInt2_10_10_10Rev(_) => gl::UNSIGNED_INT_2_10_10_10_REV,
            VecFormat::UInt10F_11F_11FRev(_) => gl::UNSIGNED_INT_10F_11F_11F_REV,
        }
    }

    fn elements(self) -> usize {
        match self {
            VecFormat::VecN(_, c) => c,
            VecFormat::Int2_10_10_10Rev(_) => 4,
            VecFormat::UInt2_10_10_10Rev(_) => 4,
            VecFormat::UInt10F_11F_11FRev(_) => 3,
        }
    }

    fn normalized(self) -> bool {
        match self {
            VecFormat::VecN(f, _) => f.normalized(),
            VecFormat::Int2_10_10_10Rev(b) => b,
            VecFormat::UInt2_10_10_10Rev(b) => b,
            VecFormat::UInt10F_11F_11FRev(b) => b,
        }
    }

}

unsafe impl AttribFormat for VecFormat {
    fn size(self) -> usize {
        match self {
            VecFormat::VecN(f, c) => c * f.size(),
            VecFormat::Int2_10_10_10Rev(_) => 4,
            VecFormat::UInt2_10_10_10Rev(_) => 4,
            VecFormat::UInt10F_11F_11FRev(_) => 4
        }
    }

    #[inline]
    unsafe fn bind_attribute(self, attr_id: GLuint, stride: usize, offset: usize){
        gl::VertexAttribPointer(attr_id, 4.min(self.elements() as GLint), self.gl_type(), self.normalized() as GLboolean, stride as GLsizei, transmute(offset as GLintptr));
    }

    #[inline]
    unsafe fn set_attribute(self, attr_id: GLuint, data: *const GLvoid){
        match self {
            VecFormat::VecN(f, c) => {
                if c==0 {panic!("Zero size vecs are invalid");}
                match f {
                    FloatFormat::Float(FloatType::Half) => {
                        match c {
                            1 => gl::VertexAttrib1sv(attr_id, transmute(data)),
                            2 => gl::VertexAttrib2sv(attr_id, transmute(data)),
                            3 => gl::VertexAttrib3sv(attr_id, transmute(data)),
                            _ => gl::VertexAttrib4sv(attr_id, transmute(data)),
                        }
                    },
                    FloatFormat::Float(FloatType::Float) => {
                        match c {
                            1 => gl::VertexAttrib1fv(attr_id, transmute(data)),
                            2 => gl::VertexAttrib2fv(attr_id, transmute(data)),
                            3 => gl::VertexAttrib3fv(attr_id, transmute(data)),
                            _ => gl::VertexAttrib4fv(attr_id, transmute(data)),
                        }
                    },
                    FloatFormat::Double => {
                        match c {
                            1 => gl::VertexAttrib1dv(attr_id, transmute(data)),
                            2 => gl::VertexAttrib2dv(attr_id, transmute(data)),
                            3 => gl::VertexAttrib3dv(attr_id, transmute(data)),
                            _ => gl::VertexAttrib4dv(attr_id, transmute(data)),
                        }
                    },
                    FloatFormat::Fixed => unimplemented!(),
                    FloatFormat::FromInt(z, normalized) => {
                        unsafe fn to_vec4<G:Copy>(ptr: *const GLvoid, count:usize, zero: G, one: G) -> [G;4] {
                            let p: *const G = transmute(ptr);
                            if count >=4 {
                                [*p, *p.offset(1), *p.offset(2), *p.offset(3)]
                            } else {
                                let mut arr = [zero, zero, zero, one];
                                for i in 0..count { arr[i] = *p.offset(i as isize);}
                                arr
                            }
                        }

                        if normalized {
                            match z {
                                IntFormat::Byte => {
                                    let arr = to_vec4::<GLbyte>(data, c, 0, 0);
                                    gl::VertexAttrib4Nbv(attr_id, &arr[0] as *const GLbyte);
                                },
                                IntFormat::UByte => {
                                    let arr = to_vec4::<GLubyte>(data, c, 0, 0);
                                    gl::VertexAttrib4Nubv(attr_id, &arr[0] as *const GLubyte);
                                },
                                IntFormat::Short => {
                                    let arr = to_vec4::<GLshort>(data, c, 0, 0);
                                    gl::VertexAttrib4Nsv(attr_id, &arr[0] as *const GLshort);
                                },
                                IntFormat::UShort => {
                                    let arr = to_vec4::<GLushort>(data, c, 0, 0);
                                    gl::VertexAttrib4Nusv(attr_id, &arr[0] as *const GLushort);
                                },
                                IntFormat::Int => {
                                    let arr = to_vec4::<GLint>(data, c, 0, 0);
                                    gl::VertexAttrib4Niv(attr_id, &arr[0] as *const GLint);
                                },
                                IntFormat::UInt => {
                                    let arr = to_vec4::<GLuint>(data, c, 0, 0);
                                    gl::VertexAttrib4Nuiv(attr_id, &arr[0] as *const GLuint);
                                },
                            }
                        } else {
                            match z {
                                IntFormat::Byte => {
                                    let arr = to_vec4::<GLbyte>(data, c, 0, 0);
                                    gl::VertexAttribI4bv(attr_id, &arr[0] as *const GLbyte);
                                },
                                IntFormat::UByte => {
                                    let arr = to_vec4::<GLubyte>(data, c, 0, 0);
                                    gl::VertexAttribI4ubv(attr_id, &arr[0] as *const GLubyte);
                                },
                                IntFormat::Short => {
                                    let arr = to_vec4::<GLshort>(data, c, 0, 0);
                                    gl::VertexAttribI4sv(attr_id, &arr[0] as *const GLshort);
                                },
                                IntFormat::UShort => {
                                    let arr = to_vec4::<GLushort>(data, c, 0, 0);
                                    gl::VertexAttribI4usv(attr_id, &arr[0] as *const GLushort);
                                },
                                IntFormat::Int => {
                                    match c {
                                        1 => gl::VertexAttribI1iv(attr_id, transmute(data)),
                                        2 => gl::VertexAttribI2iv(attr_id, transmute(data)),
                                        3 => gl::VertexAttribI3iv(attr_id, transmute(data)),
                                        _ => gl::VertexAttribI4iv(attr_id, transmute(data)),
                                    }
                                },
                                IntFormat::UInt => {
                                    match c {
                                        1 => gl::VertexAttribI1uiv(attr_id, transmute(data)),
                                        2 => gl::VertexAttribI2uiv(attr_id, transmute(data)),
                                        3 => gl::VertexAttribI3uiv(attr_id, transmute(data)),
                                        _ => gl::VertexAttribI4uiv(attr_id, transmute(data)),
                                    }
                                },
                            }
                        }
                    }
                }
            },
            _ => gl::VertexAttribP4uiv(attr_id, self.gl_type(), self.normalized() as GLboolean, transmute(data))
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum IVecFormat {
    IVecN(IntFormat, usize)
}

impl IVecFormat {
    #[inline] fn int_format(self) -> IntFormat { match self { IVecFormat::IVecN(f, _) => f } }
    #[inline] fn elements(self) -> usize { match self { IVecFormat::IVecN(_,c) => c } }
    #[inline] fn gl_type(self) -> GLenum { self.int_format().into() }

}

unsafe impl AttribFormat for IVecFormat {
    #[inline] fn size(self) -> usize { self.elements() * self.int_format().size() }

    #[inline]
    unsafe fn bind_attribute(self, attr_id: GLuint, stride: usize, offset: usize){
        gl::VertexAttribIPointer(attr_id, 4.min(self.elements() as GLint), self.gl_type(), stride as GLsizei, transmute(offset as GLintptr));
    }

    #[inline]
    unsafe fn set_attribute(self, attr_id: GLuint, data: *const GLvoid){
        VecFormat::VecN(FloatFormat::FromInt(self.int_format(), false), self.elements()).set_attribute(attr_id, data);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DVecFormat {
    DVecN(usize),
}

impl DVecFormat {
    #[inline] fn elements(self) -> usize { match self { DVecFormat::DVecN(c) => c, } }
}

unsafe impl AttribFormat for DVecFormat {
    #[inline] fn size(self) -> usize { self.elements() * 8}
    // #[inline] fn attrib_count(self) -> usize { match self { DVecN(c) => if c>2 {2} else {1} } }

    #[inline]
    unsafe fn bind_attribute(self, attr_id: GLuint, stride: usize, offset: usize){
        gl::VertexAttribLPointer(attr_id, 4.min(self.elements() as GLint), gl::DOUBLE, stride as GLsizei, transmute(offset as GLintptr));
    }

    #[inline]
    unsafe fn set_attribute(self, attr_id: GLuint, data: *const GLvoid){
        match self {
            DVecFormat::DVecN(c) => {
                match c {
                    0 => panic!("Zero size vecs are invalid"),
                    1 => gl::VertexAttribL1dv(attr_id, transmute(data)),
                    2 => gl::VertexAttribL2dv(attr_id, transmute(data)),
                    3 => gl::VertexAttribL3dv(attr_id, transmute(data)),
                    _ => gl::VertexAttribL4dv(attr_id, transmute(data)),
                }
            }
        }
    }
}

unsafe impl AttribFormat for ! {
    #[inline] fn size(self) -> usize { unreachable!() }
    #[inline] fn attrib_count(self) -> usize { unreachable!() }
    #[inline] unsafe fn bind_attribute(self, _attr_id: GLuint, _stride: usize, _offset: usize){ unreachable!() }
    #[inline] unsafe fn set_attribute(self, _attr_id: GLuint, _data: *const GLvoid){ unreachable!() }
}

pub type Mat2Format = [VecFormat; 2];
pub type Mat3Format = [VecFormat; 3];
pub type Mat4Format = [VecFormat; 4];

pub type DMat2Format = [DVecFormat; 2];
pub type DMat3Format = [DVecFormat; 3];
pub type DMat4Format = [DVecFormat; 4];

macro_rules! array_format {
    ($($num:tt)*) => {
        $(
            unsafe impl<F:AttribFormat> AttribFormat for [F; $num] {
                #[inline] fn size(self) -> usize { self.iter().map(|f| f.size()).sum() }
                #[inline] fn attrib_count(self) -> usize { $num }

                #[inline]
                unsafe fn bind_attribute(self, attr_id: GLuint, stride: usize, offset: usize) {
                    for i in 0..$num {
                        self[i].bind_attribute(attr_id + (i as GLuint)*(self[i].attrib_count() as GLuint), stride, offset + i*self[i].size());
                    }
                }

                #[inline]
                unsafe fn set_attribute(self, attr_id: GLuint, data: *const GLvoid){
                    for i in 0..$num {
                        self[i].set_attribute(attr_id + (i as GLuint)*(self[i].attrib_count() as GLuint), data.offset((i*self[i].size()) as isize));
                    }
                }
            }
        )*
    }
}

//lol no... it you need attribute arrays longer than 32 elements, you *probably* should try something else...
array_format!{
    01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
}

macro_rules! impl_attr_data {

    (@Int $prim:ident $value:expr) => {
        impl AttributeData<gl_bool> for $prim { fn format() -> IntFormat { $value }}
        impl AttributeData<int> for $prim { fn format() -> IntFormat { $value }}
        impl AttributeData<uint> for $prim { fn format() -> IntFormat { $value }}
        impl AttributeData<float> for $prim { fn format() -> FloatFormat { FloatFormat::FromInt($value, false) }}
    };

    (@IVec $F:ident $size:tt) => { IVecFormat::IVecN($F::format(), $size) };
    (@Vec $F:ident $size:tt) => { VecFormat::VecN($F::format(), $size) };
    (@DVec $F:ident $size:tt) => { DVecFormat::DVecN($size) };
    (@Mat $F:ident $size:tt) => { [$F::format(); $size] };

    (@$arr:ident $vec1:ident $vec2:ident $vec3:ident $vec4:ident) => {
        impl<F:AttributeData<$vec1>> AttributeData<$vec1> for [F; 1] { fn format() -> <$vec1 as GLSLType>::AttributeFormat { F::format()}}
        impl<F:AttributeData<$vec1>> AttributeData<$vec2> for [F; 2] { fn format() -> <$vec2 as GLSLType>::AttributeFormat { impl_attr_data!(@$arr F 2) } }
        impl<F:AttributeData<$vec1>> AttributeData<$vec3> for [F; 3] { fn format() -> <$vec3 as GLSLType>::AttributeFormat { impl_attr_data!(@$arr F 3) } }
        impl<F:AttributeData<$vec1>> AttributeData<$vec4> for [F; 4] { fn format() -> <$vec4 as GLSLType>::AttributeFormat { impl_attr_data!(@$arr F 4) } }
    };

}

impl_attr_data!(@Int bool IntFormat::UByte);
impl_attr_data!(@Int gl_bool IntFormat::UInt);
impl_attr_data!(@Int i8 IntFormat::Byte);
impl_attr_data!(@Int u8 IntFormat::UByte);
impl_attr_data!(@Int i16 IntFormat::Short);
impl_attr_data!(@Int u16 IntFormat::UShort);
impl_attr_data!(@Int i32 IntFormat::Int);
impl_attr_data!(@Int u32 IntFormat::UInt);

impl AttributeData<float> for f32 { fn format() -> FloatFormat { FloatFormat::Float(FloatType::Float) }}
impl AttributeData<float> for f64 { fn format() -> FloatFormat { FloatFormat::Double }}

impl AttributeData<double> for f64 { fn format() -> DoubleFormat {DoubleFormat}}

impl_attr_data!(@IVec gl_bool bvec2 bvec3 bvec4);
impl_attr_data!(@IVec uint uvec2 uvec3 uvec4);
impl_attr_data!(@IVec int ivec2 ivec3 ivec4);
impl_attr_data!(@Vec float vec2 vec3 vec4);
impl_attr_data!(@Mat vec2 mat2 mat3x2 mat4x2);
impl_attr_data!(@Mat vec3 mat2x3 mat3 mat4x3);
impl_attr_data!(@Mat vec4 mat2x4 mat3x4 mat4);

impl_attr_data!(@DVec double dvec2 dvec3 dvec4);
impl_attr_data!(@Mat dvec2 dmat2 dmat3x2 dmat4x2);
impl_attr_data!(@Mat dvec3 dmat2x3 dmat3 dmat4x3);
impl_attr_data!(@Mat dvec4 dmat2x4 dmat3x4 dmat4);
