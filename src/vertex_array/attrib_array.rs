
use super::*;
use glsl::*;

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub struct AttribArray<'a,A:GLSLType> {
    id: GLuint,
    format: A::AttribFormat,
    stride: usize,
    pointer: usize,
    buf: PhantomData<Slice<'a,[u8],ReadOnly>>
}

impl<'a,A:GLSLType> AttribArray<'a,A> {

    #[inline] pub fn id(&self) -> GLuint { self.id }
    #[inline] pub fn format(&self) -> A::AttribFormat { self.format }
    #[inline] pub fn stride(&self) -> usize { self.stride }
    #[inline] pub fn offset(&self) -> usize { self.pointer }
    #[inline] pub fn pointer(&self) -> *const GLvoid { self.pointer as *const _ }

    pub fn split(self) -> A::Split where A:SplitAttribs<'a> { A::split_array(self) }

    pub fn normalize(self) -> AttribArray<'a, A::Normalized> where A:NormalizeAttrib {
        AttribArray {
            id: self.id, stride: self.stride, pointer: self.pointer,
            format: A::normalize_format(self.format),
            buf: PhantomData
        }
    }

    pub fn cast<B:GLSLType>(self) -> AttribArray<'a,B> where B::AttribFormat: From<A::AttribFormat> {
        AttribArray {
            format: self.format.into(), id: self.id,
            stride: self.stride, pointer: self.pointer,
            buf: PhantomData
        }
    }

    pub unsafe fn from_raw_parts(buf:GLuint, fmt:A::AttribFormat, stride:usize, ptr:usize) -> Self {
        AttribArray { id:buf, format:fmt, stride:stride, pointer:ptr, buf:PhantomData }
    }

}

impl<'a> AttribArray<'a,()> {
    pub fn void() -> AttribArray<'a,()> {
        //basically, void types are always used to mark that an index is not used by the shader,
        //so it really doesn't matter what we put in here since we'll just be disabling the array anyway.
        AttribArray {
            id: 0, format: (), stride: 0, pointer: 0, buf: PhantomData
        }
    }
}

impl<'b, A:GLSLType, T, B:Initialized> From<Slice<'b,[T],B>> for AttribArray<'b,A>
where T: AttribData<GLSL=A, Format=A::AttribFormat>
{
    fn from(buf: Slice<'b,[T],B>) -> Self {
        AttribArray {
            format: T::format(), id: buf.id(),
            stride: size_of::<T>(), pointer: buf.offset(),
            buf: PhantomData
        }
    }
}

impl<'a,'b,A1:GLSLType,A2:GLSLType> From<&'a AttribArray<'b,A1>> for AttribArray<'b,A2>
where A2::AttribFormat: From<A1::AttribFormat>
{
    fn from(arr: &'a AttribArray<'b,A1>) -> AttribArray<'b,A2> { arr.cast() }
}

impl<'a,A:GLSLType> Target<AttribArray<'a,A>> for BufferTarget {
    fn target_id(self) -> GLenum { self as GLenum }
    unsafe fn bind(self, buf:&AttribArray<'a,A>) { gl::BindBuffer(self.into(), buf.id()) }
    unsafe fn unbind(self) { gl::BindBuffer(self.into(), 0) }
}

pub trait NormalizeAttrib: GLSLType {
    type Normalized: GLSLType;
    fn normalize_format(fmt: Self::AttribFormat) -> <Self::Normalized as GLSLType>::AttribFormat;
}

macro_rules! impl_vec_normalize {
    ($($ivec:ident => $vec:ident;)*) => {
        $(
            impl NormalizeAttrib for $ivec {
                type Normalized = $vec;
                fn normalize_format(fmt: Self::AttribFormat) -> <Self::Normalized as GLSLType>::AttribFormat {
                    fmt.normalize()
                }
            }
        )*
    }
}

impl_vec_normalize!{
    int => float;
    ivec2 => vec2;
    ivec3 => vec3;
    ivec4 => vec4;

    uint => float;
    uvec2 => vec2;
    uvec3 => vec3;
    uvec4 => vec4;
}

pub trait SplitAttribs<'a>: GLSLType {
    type Split;
    fn split_array(array: AttribArray<'a,Self>) -> Self::Split;
}

macro_rules! impl_tuple_attrib {

    ($($T:ident:$t:ident)*) => {

        impl<$($T:NormalizeAttrib),*> NormalizeAttrib for ($($T,)*) {

            type Normalized = ($($T::Normalized,)*);

            fn normalize_format(fmt: Self::AttribFormat) -> <Self::Normalized as GLSLType>::AttribFormat {
                let ($($t,)*) = fmt;
                (
                    $(OffsetFormat {offset: $t.offset, fmt:$T::normalize_format($t.fmt)},)*
                )
            }

        }

        impl<'a, $($T:GLSLType),*> SplitAttribs<'a> for ($($T,)*) {

            type Split = ($(AttribArray<'a,$T>,)*);

            #[allow(unused_variables, unused_mut, unused_assignments)]
            fn split_array(array: AttribArray<'a,Self>) -> Self::Split {
                let (id, stride, base_offset) = (array.id(), array.stride(), array.offset());
                let ($($t,)*) = array.format();
                (
                    $(
                        AttribArray {
                            id: id, stride: stride,
                            format: $t.fmt, pointer: base_offset+$t.offset,
                            buf: PhantomData
                        },
                    )*
                )
            }

        }
    }
}

impl_tuple!(impl_tuple_attrib);

macro_rules! impl_split_array {
    ($($n:literal)*) => {
        $(

            impl<T:NormalizeAttrib> NormalizeAttrib for [T; $n] {
                type Normalized = [T::Normalized; $n];

                fn normalize_format(fmt: Self::AttribFormat) -> <Self::Normalized as GLSLType>::AttribFormat {
                    arr![
                        for i in 0..$n {
                            OffsetFormat {
                                offset: fmt[i].offset,
                                fmt: T::normalize_format(fmt[i].fmt)
                            }
                        }
                    ]
                }

            }

            impl<'a, T:GLSLType> SplitAttribs<'a> for [T; $n] {

                type Split = [AttribArray<'a,T>; $n];

                #[allow(unused_variables, unused_mut, unused_assignments)]
                fn split_array(array: AttribArray<'a,Self>) -> Self::Split {
                    let (id, stride, base_offset) = (array.id(), array.stride(), array.offset());
                    let format = array.format();

                    arr![
                        for i in 0..$n {
                            AttribArray {
                                id: id, stride: stride,
                                format: format[i].fmt,
                                pointer: base_offset + format[i].offset,
                                buf: PhantomData
                            }
                        }
                    ]

                }

            }

        )*
    }
}

impl_split_array! {
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
}

macro_rules! impl_split_matrix {

    ($($mat:ident => [$vec:ident; $n:literal];)*) => {

        $(
            impl<'a> SplitAttribs<'a> for $mat {

                type Split = [AttribArray<'a,$vec>; $n];

                #[allow(unused_variables, unused_mut, unused_assignments)]
                fn split_array(array: AttribArray<'a,Self>) -> Self::Split {
                    SplitAttribs::split_array(
                        AttribArray::<'a, [$vec; $n]>::from(&array)
                    )
                }

            }
        )*

    }


}

impl_split_matrix!{
    mat2x2 => [vec2; 2];
    mat2x3 => [vec3; 2];
    mat2x4 => [vec4; 2];
    mat3x2 => [vec2; 3];
    mat3x3 => [vec3; 3];
    mat3x4 => [vec4; 3];
    mat4x2 => [vec2; 4];
    mat4x3 => [vec3; 4];
    mat4x4 => [vec4; 4];

    dmat2x2 => [dvec2; 2];
    dmat2x3 => [dvec3; 2];
    dmat2x4 => [dvec4; 2];
    dmat3x2 => [dvec2; 3];
    dmat3x3 => [dvec3; 3];
    dmat3x4 => [dvec4; 3];
    dmat4x2 => [dvec2; 4];
    dmat4x3 => [dvec3; 4];
    dmat4x4 => [dvec4; 4];
}
