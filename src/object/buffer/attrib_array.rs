
use super::*;
use glsl::GLSLType;
use format::attribute::*;

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

    #[inline]
    pub unsafe fn from_raw_parts(fmt:A::AttribFormat, id:GLuint, stride:usize, ptr:usize) -> Self {
        AttribArray { id:id, format:fmt, stride:stride, pointer:ptr, buf:PhantomData }
    }

}

pub unsafe trait SplitAttribs<'a, A:Copy>: Copy {
    fn split<B:Initialized>(buf:Slice<'a,[Self],B>) -> A;
}

unsafe impl<'a, A:GLSLType, T:AttribData<A::AttribFormat>> SplitAttribs<'a,AttribArray<'a,A>> for T {
    fn split<B:Initialized>(buf:Slice<'a,[T],B>) -> AttribArray<'a,A> {
        unsafe {
            AttribArray::from_raw_parts(T::format(), buf.id(), size_of::<T>(), buf.offset())
        }
    }
}

macro_rules! impl_split_tuple {

    () => {
        impl_split_tuple!(@loop {T1:A1} T2:A2 T3:A3 T4:A4 T5:A5 T6:A6 T7:A7 T8:A8 T9:A9 T10:A10 T11:A11 T12:A12);
    };

    (@loop {$($T:ident:$A:ident)*} ) => { impl_split_tuple!(@impl $($T:$A)*); };
    (@loop {$($T:ident:$A:ident)*} $T0:ident:$A0:ident $($rest:tt)*) => {
        impl_split_tuple!(@impl $($T:$A)*);
        impl_split_tuple!(@loop {$($T:$A)* $T0:$A0}  $($rest)*);
    };

    (@impl $($T:ident:$A:ident)*) => {
        unsafe impl<'a,$($A:GLSLType,)* $($T:AttribData<$A::AttribFormat>),*> SplitAttribs<'a,($(AttribArray<'a,$A>,)*)> for ($($T,)*) {

            #[allow(unused_variables, unused_mut, unused_assignments)]
            fn split<B:Initialized>(buf:Slice<'a,[Self],B>) -> ($(AttribArray<'a,$A>,)*) {
                let (id, stride) = (buf.id(), size_of::<Self>());
                let mut pointer = buf.offset();
                (
                    $(
                        unsafe {
                            let arr = AttribArray::from_raw_parts($T::format(), id, stride, pointer);
                            pointer += size_of::<$T>();
                            pointer += (pointer as *const u8).align_offset(align_of::<$T>());
                            arr
                        },
                    )*
                )
            }

        }
    }
}

impl_split_tuple!();
