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
