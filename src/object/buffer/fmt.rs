use super::*;

use std::fmt::*;

macro_rules! map_fmt {
    ($ty:ident) => {

        impl<'a, T:?Sized+$ty, A:BufferAccess> $ty for Slice<'a,T,A> {
            fn fmt(&self, f:&mut Formatter) -> Result {
                unsafe { map_dealloc(self._read_into_box(), |ptr| $ty::fmt(&*ptr, f)) }
            }
        }

        impl<'a, T:?Sized+$ty, A:BufferAccess> $ty for SliceMut<'a,T,A> {
            fn fmt(&self, f:&mut Formatter) -> Result { $ty::fmt(&self.as_immut(), f) }
        }

        impl<'a, T:?Sized+$ty, A:ReadAccess> $ty for Map<'a,T,A> {
            fn fmt(&self, f:&mut Formatter) -> Result { $ty::fmt(&**self, f) }
        }

        impl<T:?Sized+$ty, A:BufferAccess> $ty for Buffer<T,A> {
            fn fmt(&self, f:&mut Formatter) -> Result { $ty::fmt(&self.as_slice(), f) }
        }

    }
}

map_fmt!(Debug);
map_fmt!(Display);


impl<'a, T:?Sized, A:BufferAccess> Pointer for Map<'a,T,A> {
    fn fmt(&self, f:&mut Formatter) -> Result { Pointer::fmt(&Map::as_ptr(self), f) }
}