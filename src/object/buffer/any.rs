use super::*;

use std::any::*;

impl<A:BufferAccess> Buffer<dyn Any + 'static, A> {
    pub fn downcast<T:Any>(self) -> Result<Buffer<T,A>, Self> {
        unsafe {
            if let Some(cast) = (&mut *self.ptr.dangling_mut()).downcast_mut() {
                let new = BufPtr::new(self.id(), cast);
                forget(self);
                Ok(Buffer{ptr: new, access: PhantomData})
            } else {
                Err(self)
            }
        }
    }
}

impl<'a, A:BufferAccess> Slice<'a, dyn Any + 'static, A> {
    pub fn downcast<T:Any>(self) -> Result<Slice<'a,T,A>, Self> {
        unsafe {
            if let Some(cast) = (&mut *self.ptr.dangling_mut()).downcast_mut() {
                let new = BufPtr::new(self.id(), cast);
                let offset = self.offset();
                forget(self);
                Ok(Slice{ptr: new, offset: offset, buf: PhantomData})
            } else {
                Err(self)
            }
        }
    }
}

impl<'a, A:BufferAccess> SliceMut<'a, dyn Any + 'static, A> {
    pub fn downcast<T:Any>(self) -> Result<SliceMut<'a,T,A>, Self> {
        unsafe {
            if let Some(cast) = (&mut *self.ptr.dangling_mut()).downcast_mut() {
                let new = BufPtr::new(self.id(), cast);
                let offset = self.offset();
                forget(self);
                Ok(SliceMut{ptr: new, offset: offset, buf: PhantomData})
            } else {
                Err(self)
            }
        }
    }
}

impl<'a, A:BufferAccess> Map<'a, dyn Any + 'static, A> {
    pub fn downcast<T:Any>(self) -> Result<Map<'a,T,A>, Self> {
        unsafe {
            if let Some(cast) = (&mut *self.ptr).downcast_mut() {
                let offset = self.offset;
                let id = self.id;
                forget(self);
                Ok(Map{ptr: cast, offset: offset, id: id, buf: PhantomData})
            } else {
                Err(self)
            }
        }
    }
}
