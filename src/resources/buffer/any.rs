use super::*;

use std::any::*;

impl<A:BufferAccess> Buffer<dyn Any + 'static, A> {
    pub fn downcast<T:Any>(self) -> Result<Buffer<T,A>, Self> {
        unsafe {
            if let Some(cast) = (&mut *self.ptr).downcast_mut() {
                let mut new = BufPtr { rust_mut: cast};
                new.buf = self.id();
                forget(self);
                Ok(Buffer{ptr: new.rust_mut, access: PhantomData})
            } else {
                Err(self)
            }
        }
    }
}
