
use super::*;
use glsl::GLSLType;

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

    pub unsafe fn from_raw_parts(fmt:A::AttribFormat, id:GLuint, stride:usize, ptr:usize) -> Self {
        AttribArray { id:id, format:fmt, stride:stride, pointer:ptr, buf:PhantomData }
    }

    pub fn from_slice<T, B:Initialized>(buf: Slice<'a,[T],B>) -> Self
    where T: AttribData<GLSL=A, Format=A::AttribFormat>
    {
        buf.into()
    }

}

impl<'b, A:GLSLType, T, B:Initialized> From<Slice<'b,[T],B>> for AttribArray<'b,A>
where T: AttribData<GLSL=A, Format=A::AttribFormat>
{
    fn from(buf: Slice<'b,[T],B>) -> Self {
        unsafe {
            Self::from_raw_parts(T::format(), buf.id(), size_of::<T>(), buf.offset())
        }
    }
}

impl<'a,'b,A1:GLSLType,A2:GLSLType> From<&'a AttribArray<'b,A1>> for AttribArray<'b,A2>
where A2::AttribFormat: From<A1::AttribFormat>
{
    fn from(arr: &'a AttribArray<'b,A1>) -> Self {
        unsafe {
            Self::from_raw_parts(From::from(arr.format()), arr.id(), arr.stride(), arr.offset())
        }
    }
}

pub unsafe trait SplitAttribs<'a>: Copy {
    type AttribArrays;
    fn split_array(array: AttribArray<'a,Self>) -> Self::AttribArrays where Self: GLSLType;
    fn split_buffer<B:Initialized>(buf: Slice<'a,[Self],B>) -> Self::AttribArrays;
}

macro_rules! impl_split_tuple {

    ($($T:ident:$a:ident)*) => {
        unsafe impl<'a, $($T:AttribData),*> SplitAttribs<'a> for ($($T,)*) {

            type AttribArrays = ($(AttribArray<'a,$T::GLSL>,)*);

            #[allow(unused_variables, unused_mut, unused_assignments)]
            fn split_array(array: AttribArray<'a,Self>) -> Self::AttribArrays where Self: GLSLType {
                let (id, stride) = (array.id(), array.stride());
                let mut pointer = array.offset();
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

            #[allow(unused_variables, unused_mut, unused_assignments)]
            fn split_buffer<B:Initialized>(buf:Slice<'a,[Self],B>) -> Self::AttribArrays {
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

impl_tuple!(
    {A0:a0 A1:a1 A2:a2 A3:a3 A4:a4 A5:a5 A6:a6 A7:a7 A8:a8 A9:a9 AA:aa AB:ab AC:ac AD:ad AE:ae} AF:af
    impl_split_tuple
);

macro_rules! impl_split_array {
    ($($n:literal)*) => {
        $(
            unsafe impl<'a, T:AttribData> SplitAttribs<'a> for [T; $n] {

                type AttribArrays = [AttribArray<'a,T::GLSL>; $n];

                #[allow(unused_variables, unused_mut, unused_assignments)]
                fn split_array(array: AttribArray<'a,Self>) -> Self::AttribArrays where Self: GLSLType {
                    let (id, stride, base_offset) = (array.id(), array.stride(), array.offset());
                    let format = <[T; $n] as AttribData>::format();

                    let mut attribs = MaybeUninit::<Self::AttribArrays>::uninit();
                    for i in 0..$n {
                        unsafe {
                            attribs.get_mut()[i] = AttribArray::from_raw_parts(
                                format[i].fmt, id, stride, base_offset + format[i].offset
                            );
                        }
                    }

                    unsafe { attribs.assume_init() }

                }

                #[allow(unused_variables, unused_mut, unused_assignments)]
                fn split_buffer<B:Initialized>(buf:Slice<'a,[Self],B>) -> Self::AttribArrays {
                    let (id, stride, base_offset) = (buf.id(), size_of::<Self>(), buf.offset());
                    let format = <[T; $n] as AttribData>::format();

                    let mut attribs = MaybeUninit::<Self::AttribArrays>::uninit();
                    for i in 0..$n {
                        unsafe {
                            attribs.get_mut()[i] = AttribArray::from_raw_parts(
                                format[i].fmt, id, stride, base_offset + format[i].offset
                            );
                        }
                    }

                    unsafe { attribs.assume_init() }
                }

            }

        )*
    }
}

impl_split_array! {
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
}
