use super::*;
use format::{ClientFormat,PixelType,PixelData,PixelDataMut,PixelRowAlignment};

unsafe impl<'a,F:ClientFormat,T:PixelType<F>,A:BufferAccess> PixelData<F> for Slice<'a,[T],A> {
    #[inline] fn swap_bytes(&self) -> bool {T::swap_bytes()}
    #[inline] fn lsb_first(&self) -> bool {T::lsb_first()}

    #[inline] fn alignment(&self) -> PixelRowAlignment { (align_of::<T>().min(8) as u8).try_into().unwrap() }

    #[inline] fn format_type(&self) -> F {T::format_type()}
    #[inline] fn count(&self) -> usize {Slice::len(self)}
    #[inline] fn size(&self) -> usize {Slice::size(self)}

    #[inline] fn pixels<'b>(
        &'b self, target:&'b mut BindingLocation<RawBuffer>
    ) -> (Option<Binding<'b,RawBuffer>>, *const GLvoid) {
        (Some(target.bind_slice(self)), self.offset() as *const GLvoid)
    }
}

unsafe impl<'a,F:ClientFormat,T:PixelType<F>,A:BufferAccess> PixelData<F> for SliceMut<'a,[T],A> {
    #[inline] fn swap_bytes(&self) -> bool {T::swap_bytes()}
    #[inline] fn lsb_first(&self) -> bool {T::lsb_first()}

    #[inline] fn alignment(&self) -> PixelRowAlignment { (align_of::<T>().min(8) as u8).try_into().unwrap() }

    #[inline] fn format_type(&self) -> F {T::format_type()}
    #[inline] fn count(&self) -> usize {SliceMut::len(self)}
    #[inline] fn size(&self) -> usize {SliceMut::size(self)}

    #[inline] fn pixels<'b>(
        &'b self, target:&'b mut BindingLocation<RawBuffer>
    ) -> (Option<Binding<'b,RawBuffer>>, *const GLvoid) {
        (Some(target.bind_slice_mut(self)), self.offset() as *const GLvoid)
    }
}

unsafe impl<'a,F:ClientFormat,T:PixelType<F>,A:BufferAccess> PixelDataMut<F> for SliceMut<'a,[T],A> {
    #[inline] fn pixels_mut<'b>(
        &'b mut self, target:&'b mut BindingLocation<RawBuffer>
    ) -> (Option<Binding<'b,RawBuffer>>, *mut GLvoid) {
        (Some(target.bind_slice_mut(self)), self.offset() as *mut GLvoid)
    }
}
