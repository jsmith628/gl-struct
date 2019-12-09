
use super::*;
use glsl::GLSLType;
use format::attribute::*;

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub struct AttribArray<'a,A:GLSLType> {
    buf: Slice<'a,[u8],ReadOnly>,
    stride: usize,
    format: A::AttribFormat,
}

impl<'a,A:GLSLType> AttribArray<'a,A> {
    #[inline] pub fn id(&self) -> GLuint { self.buf.id() }
    #[inline] pub fn stride(&self) -> usize { self.stride }
    #[inline] pub fn format(&self) -> A::AttribFormat { self.format }
}

pub unsafe trait SplitAttribs<'a, A:Copy>: Copy {
    fn split<B:Initialized>(buf:Slice<'a,[Self],B>) -> A;
}

macro_rules! impl_split_tuple {

    () => {
        impl_split_tuple!(@loop {} T1:A1 T2:A2 T3:A3 T4:A4 T5:A5 T6:A6 T7:A7 T8:A8 T9:A9 T10:A10 T11:A11 T12:A12);
    };

    (@loop {$($T:ident:$A:ident)*} ) => {};
    (@loop {$($T:ident:$A:ident)*} $T0:ident:$A0:ident $($rest:tt)*) => {
        impl_split_tuple!(@impl $($T:$A)*);
        impl_split_tuple!(@loop {$($T:$A)* $T0:$A0}  $($rest)*);
    };

    (@impl $($T:ident:$A:ident)*) => {
        unsafe impl<'a,$($A:GLSLType,)* $($T:AttribData<$A::AttribFormat>),*> SplitAttribs<'a,($(AttribArray<'a,$A>,)*)> for ($($T,)*) {

            #[allow(unused_variables, unused_mut, unused_assignments)]
            fn split<B:Initialized>(buf:Slice<'a,[Self],B>) -> ($(AttribArray<'a,$A>,)*) {
                let (id, size, offset, stride) = (buf.id(), buf.size(), buf.offset(), size_of::<Self>());
                let mut pos = 0;
                (
                    $(
                        AttribArray {
                            buf: unsafe {
                                let buf = Slice::from_raw_parts(id, size-pos, offset+pos);
                                pos += size_of::<$T>();
                                pos += (pos as *const u8).align_offset(align_of::<$T>());
                                buf
                            },
                            stride: stride,
                            format: $T::format()
                        },
                    )*
                )
            }

        }
    }
}

impl_split_tuple!();
