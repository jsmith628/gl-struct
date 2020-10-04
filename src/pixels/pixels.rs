use super::*;

#[derive(Derivative)]
#[derivative(Clone(bound=""), Copy(bound=""))]
pub struct Pixels<'a,P:?Sized,GL> {
    //is either a rust reference or OpenGL buffer
    pixels: GLRef<'a,P,ReadOnly>,

    //idrk how to represent that GL is less "owned" and more "required", but I guess this mostly works..
    gl: PhantomData<MaybeUninit<GL>>
}

impl<'a,'b:'a,P:?Sized,GL> From<&'a Pixels<'b,P,GL>> for Pixels<'b,P,GL> {
    fn from(r:&'a Pixels<'b,P,GL>) -> Self { Self { pixels:r.pixels.into(), gl:PhantomData } }
}

impl<'a,'b:'a,P:?Sized,GL> From<&'a PixelsMut<'b,P,GL>> for Pixels<'a,P,GL> {
    fn from(r:&'a PixelsMut<'b,P,GL>) -> Self { Self { pixels: (&r.pixels).into(), gl:PhantomData } }
}

impl<'a,P:?Sized,GL> From<PixelsMut<'a,P,GL>> for Pixels<'a,P,GL> {
    fn from(r:PixelsMut<'a,P,GL>) -> Self { Self { pixels:r.pixels.into(), gl:PhantomData } }
}

impl<'a,P:?Sized> Pixels<'a,P,()> {
    pub fn from_ref(slice: &'a P) -> Self {
        Self { pixels: GLRef::Ref(&slice), gl: PhantomData }
    }
}

impl<'a,P:?Sized> Pixels<'a,P,GL_ARB_pixel_buffer_object> {
    pub fn from_buf<A:BufferStorage>(slice: Slice<'a,P,A>) -> Self {
        Self { pixels: GLRef::Buf(slice.downgrade()), gl: PhantomData }
    }
}

impl<'a,P:?Sized,GL> Pixels<'a,P,GL> {
    pub fn size(&self) -> usize { self.pixels.size() }
    pub fn borrow(&self) -> GLRef<P,ReadOnly> { self.pixels }
}

impl<'a,P,GL> Pixels<'a,[P],GL> {
    pub fn is_empty(&self) -> bool { self.pixels.is_empty() }
    pub fn len(&self) -> usize { self.pixels.len() }
}

impl<'a,F:SpecificCompressed,GL> Pixels<'a,Cmpr<F>,GL> {
    pub fn is_empty(&self) -> bool { self.pixels.is_empty() }
    pub fn len(&self) -> usize { self.pixels.len() }
}

impl<'a,P:?Sized,GL1:GLVersion> Pixels<'a,P,GL1> {

    pub fn lock<GL2:Supports<GL1>>(self) -> Pixels<'a,P,GL2> {
        Pixels { pixels:self.pixels, gl:PhantomData }
    }

    pub fn unlock<GL2:Supports<GL1>>(self, _gl: &GL2) -> Pixels<'a,P,()> {
        Pixels { pixels:self.pixels, gl:PhantomData }
    }

}

pub struct PixelsMut<'a,P:?Sized,GL> {
    //is either a rust reference or OpenGL buffer
    pixels: GLMut<'a,P,ReadOnly>,

    //idrk how to represent that GL is less "owned" and more "required", but I guess this mostly works..
    gl: PhantomData<MaybeUninit<GL>>
}

impl<'a,'b:'a,P:?Sized,GL> From<&'a mut PixelsMut<'b,P,GL>> for PixelsMut<'a,P,GL> {
    fn from(r:&'a mut PixelsMut<'b,P,GL>) -> Self { Self { pixels:(&mut r.pixels).into(), gl:PhantomData } }
}

impl<'a,P:?Sized> PixelsMut<'a,P,()> {
    pub fn from_mut(slice: &'a mut P) -> Self {
        Self { pixels: GLMut::Mut(slice), gl: PhantomData }
    }
}

impl<'a,P:?Sized> PixelsMut<'a,P,GL_ARB_pixel_buffer_object> {
    pub fn from_buf<A:BufferStorage>(slice: SliceMut<'a,P,A>) -> Self {
        Self { pixels: GLMut::Buf(slice.downgrade()), gl: PhantomData }
    }
}

impl<'a,P:?Sized,GL> PixelsMut<'a,P,GL> {
    pub fn size(&self) -> usize { self.pixels.size() }
    pub fn borrow(&self) -> GLRef<P,ReadOnly> { (&self.pixels).into() }
    pub fn borrow_mut(&mut self) -> GLMut<P,ReadOnly> { (&mut self.pixels).into() }
}

impl<'a,P,GL> PixelsMut<'a,[P],GL> {
    pub fn is_empty(&self) -> bool { self.pixels.is_empty() }
    pub fn len(&self) -> usize { self.pixels.len() }
}

impl<'a,F:SpecificCompressed,GL> PixelsMut<'a,Cmpr<F>,GL> {
    pub fn is_empty(&self) -> bool { self.pixels.is_empty() }
    pub fn len(&self) -> usize { self.pixels.len() }
}

impl<'a,P:?Sized,GL1:GLVersion> PixelsMut<'a,P,GL1> {

    pub fn lock<GL2:Supports<GL1>>(self) -> PixelsMut<'a,P,GL2> {
        PixelsMut { pixels:self.pixels, gl: PhantomData }
    }

    pub fn unlock<GL2:Supports<GL1>>(self, _gl: &GL2) -> PixelsMut<'a,P,()> {
        PixelsMut { pixels:self.pixels, gl: PhantomData }
    }

}
