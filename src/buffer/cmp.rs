use super::*;


use std::cmp::*;

macro_rules! read_map {
    ($self:ident $rhs:ident $fun:ident) => {
        unsafe {
            let data1 = $self._read_into_box();
            let data2 = $rhs._read_into_box();
            map_dealloc(data1, |ptr1| map_dealloc(data2, |ptr2| (&*ptr1).$fun(&*ptr2)))
        }
    }
}

//
//PartialEq
//

impl<'a, 'b, T, U, A, B> PartialEq<Slice<'b,U,B>> for Slice<'a,T,A> where
    T:PartialEq<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn eq(&self, rhs:&Slice<'b,U,B>) -> bool { read_map!(self rhs eq) }
    fn ne(&self, rhs:&Slice<'b,U,B>) -> bool { read_map!(self rhs ne) }
}

impl<'a, 'b, T, U, A, B> PartialEq<SliceMut<'b,U,B>> for Slice<'a,T,A> where
    T:PartialEq<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn eq(&self, rhs:&SliceMut<'b,U,B>) -> bool { self.eq(&rhs.as_immut()) }
    fn ne(&self, rhs:&SliceMut<'b,U,B>) -> bool { self.ne(&rhs.as_immut()) }
}

impl<'a, T, U, A, B> PartialEq<Buffer<U,B>> for Slice<'a,T,A> where
    T:PartialEq<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn eq(&self, rhs:&Buffer<U,B>) -> bool { self.eq(&rhs.as_slice()) }
    fn ne(&self, rhs:&Buffer<U,B>) -> bool { self.ne(&rhs.as_slice()) }
}

impl<'a, 'b, T, U, A, B> PartialEq<Slice<'b,U,B>> for SliceMut<'a,T,A> where
    T:PartialEq<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn eq(&self, rhs:&Slice<'b,U,B>) -> bool { self.as_immut().eq(rhs) }
    fn ne(&self, rhs:&Slice<'b,U,B>) -> bool { self.as_immut().ne(rhs) }
}

impl<'a, T, U, A, B> PartialEq<Buffer<U,B>> for SliceMut<'a,T,A> where
    T:PartialEq<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn eq(&self, rhs:&Buffer<U,B>) -> bool { self.as_immut().eq(&rhs.as_slice()) }
    fn ne(&self, rhs:&Buffer<U,B>) -> bool { self.as_immut().ne(&rhs.as_slice()) }
}

impl<'a, 'b, T, U, A, B> PartialEq<SliceMut<'b,U,B>> for SliceMut<'a,T,A> where
    T:PartialEq<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn eq(&self, rhs:&SliceMut<'b,U,B>) -> bool { self.as_immut().eq(&rhs.as_immut()) }
    fn ne(&self, rhs:&SliceMut<'b,U,B>) -> bool { self.as_immut().ne(&rhs.as_immut()) }
}

impl<'b, T, U, A, B> PartialEq<Slice<'b,U,B>> for Buffer<T,A> where
    T:PartialEq<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn eq(&self, rhs:&Slice<'b,U,B>) -> bool { self.as_slice().eq(rhs) }
    fn ne(&self, rhs:&Slice<'b,U,B>) -> bool { self.as_slice().ne(rhs) }
}

impl<'b, T, U, A, B> PartialEq<SliceMut<'b,U,B>> for Buffer<T,A> where
    T:PartialEq<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn eq(&self, rhs:&SliceMut<'b,U,B>) -> bool { self.as_slice().eq(&rhs.as_immut()) }
    fn ne(&self, rhs:&SliceMut<'b,U,B>) -> bool { self.as_slice().ne(&rhs.as_immut()) }
}

impl<T, U, A, B> PartialEq<Buffer<U,B>> for Buffer<T,A> where
    T:PartialEq<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn eq(&self, rhs:&Buffer<U,B>) -> bool { self.as_slice().eq(&rhs.as_slice()) }
    fn ne(&self, rhs:&Buffer<U,B>) -> bool { self.as_slice().ne(&rhs.as_slice()) }
}

//
//PartialOrd
//

impl<'a, 'b, T, U, A, B> PartialOrd<Slice<'b,U,B>> for Slice<'a,T,A> where
    T:PartialOrd<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn partial_cmp(&self, rhs:&Slice<'b,U,B>) -> Option<Ordering> { read_map!(self rhs partial_cmp) }
}

impl<'a, 'b, T, U, A, B> PartialOrd<SliceMut<'b,U,B>> for Slice<'a,T,A> where
    T:PartialOrd<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn partial_cmp(&self, rhs:&SliceMut<'b,U,B>) -> Option<Ordering> { self.partial_cmp(&rhs.as_immut()) }
}

impl<'a, T, U, A, B> PartialOrd<Buffer<U,B>> for Slice<'a,T,A> where
    T:PartialOrd<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn partial_cmp(&self, rhs:&Buffer<U,B>) -> Option<Ordering> { self.partial_cmp(&rhs.as_slice()) }
}

impl<'a, 'b, T, U, A, B> PartialOrd<Slice<'b,U,B>> for SliceMut<'a,T,A> where
    T:PartialOrd<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn partial_cmp(&self, rhs:&Slice<'b,U,B>) -> Option<Ordering> { self.as_immut().partial_cmp(rhs) }
}

impl<'a, 'b, T, U, A, B> PartialOrd<SliceMut<'b,U,B>> for SliceMut<'a,T,A> where
    T:PartialOrd<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn partial_cmp(&self, rhs:&SliceMut<'b,U,B>) -> Option<Ordering> { self.as_immut().partial_cmp(&rhs.as_immut()) }
}

impl<'a, T, U, A, B> PartialOrd<Buffer<U,B>> for SliceMut<'a,T,A> where
    T:PartialOrd<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn partial_cmp(&self, rhs:&Buffer<U,B>) -> Option<Ordering> { self.as_immut().partial_cmp(&rhs.as_slice()) }
}

impl<'b, T, U, A, B> PartialOrd<Slice<'b,U,B>> for Buffer<T,A> where
    T:PartialOrd<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn partial_cmp(&self, rhs:&Slice<'b,U,B>) -> Option<Ordering> { self.as_slice().partial_cmp(rhs) }
}

impl<'b, T, U, A, B> PartialOrd<SliceMut<'b,U,B>> for Buffer<T,A> where
    T:PartialOrd<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn partial_cmp(&self, rhs:&SliceMut<'b,U,B>) -> Option<Ordering> { self.as_slice().partial_cmp(&rhs.as_immut()) }
}

impl<T, U, A, B> PartialOrd<Buffer<U,B>> for Buffer<T,A> where
    T:PartialOrd<U>+?Sized, U:?Sized, A:BufferAccess, B:BufferAccess
{
    fn partial_cmp(&self, rhs:&Buffer<U,B>) -> Option<Ordering> { self.as_slice().partial_cmp(&rhs.as_slice()) }
}

//
//Eq and Ord
//

impl<'a, T:Eq+?Sized, A:BufferAccess> Eq for Slice<'a,T,A> {}
impl<'a, T:Ord+?Sized, A:BufferAccess> Ord for Slice<'a,T,A> {
    fn cmp(&self, rhs:&Self) -> Ordering { read_map!(self rhs cmp) }
}

impl<'a, T:Eq+?Sized, A:BufferAccess> Eq for SliceMut<'a,T,A> {}
impl<'a, T:Ord+?Sized, A:BufferAccess> Ord for SliceMut<'a,T,A> {
    fn cmp(&self, rhs:&Self) -> Ordering { self.as_immut().cmp(&rhs.as_immut()) }
}

impl<T:Eq+?Sized, A:BufferAccess> Eq for Buffer<T,A> {}
impl<T:Ord+?Sized, A:BufferAccess> Ord for Buffer<T,A> {
    fn cmp(&self, rhs:&Self) -> Ordering { self.as_slice().cmp(&rhs.as_slice()) }
}
