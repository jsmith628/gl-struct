use super::*;

use std::marker::PhantomData;
use std::mem::*;
use std::ptr::*;

use crate::context::*;
use crate::buffer::*;
use crate::glsl::GLSLType;

pub use self::layout::*;
pub use self::vertex::*;
pub use self::attrib::*;
pub use self::attrib_array::*;

pub mod layout;
pub mod vertex;
mod attrib;
mod attrib_array;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct VertexArrayTarget;
display_from_debug!(VertexArrayTarget);

impl<'a,E:Copy,V:Vertex<'a>> Target<VertexArray<'a,E,V>> for VertexArrayTarget {
    fn target_id(self) -> GLenum {0}
    unsafe fn bind(self, obj: &VertexArray<'a,E,V>) { gl::BindVertexArray(obj.id()) }
    unsafe fn unbind(self) { gl::BindVertexArray(0) }
}

impl<'a,'b,T:GLSLType+'b> Target<VertexAttrib<'a,'b,T>> for VertexArrayTarget {
    fn target_id(self) -> GLenum {0}
    unsafe fn bind(self, obj: &VertexAttrib<'a,'b,T>) { gl::BindVertexArray(obj.id()) }
    unsafe fn unbind(self) { gl::BindVertexArray(0) }
}

impl<'a,'b,T:GLSLType+'b> Target<VertexAttribMut<'a,'b,T>> for VertexArrayTarget {
    fn target_id(self) -> GLenum {0}
    unsafe fn bind(self, obj: &VertexAttribMut<'a,'b,T>) { gl::BindVertexArray(obj.id()) }
    unsafe fn unbind(self) { gl::BindVertexArray(0) }
}

pub(self) static mut VERTEX_ARRAY: BindingLocation<VertexArrayTarget> = unsafe {
    BindingLocation::new(VertexArrayTarget)
};

pub(self) static mut ARRAY_BUFFER: BindingLocation<BufferTarget> = unsafe {
    BindingLocation::new(BufferTarget::ArrayBuffer)
};

#[repr(transparent)]
pub struct VertexArray<'a,E:Copy,V:Vertex<'a>> {
    id: GLuint,
    buffers: PhantomData<(&'a Buffer<[E], ReadOnly>, V::Arrays)>
}

impl<'a> VertexArray<'a,!,()> {
    pub fn gen(#[allow(unused_variables)] gl:&GL30) -> GLuint {
        unsafe {
            let mut dest = MaybeUninit::uninit();
            gl::GenVertexArrays(1, dest.as_mut_ptr());
            dest.assume_init()
        }
    }

    pub fn gen_vertex_arrays(#[allow(unused_variables)] gl:&GL30, n:GLuint) -> Box<[GLuint]> {
        if n==0 { return Box::new([]); }
        unsafe {
        let mut dest = Box::new_uninit_slice(n as usize);
            gl::GenVertexArrays(dest.len().try_into().unwrap(), MaybeUninit::first_ptr_mut(&mut *dest));
            dest.assume_init()
        }
    }

    pub fn create(#[allow(unused_variables)] gl:&GL30) -> Self {
        let mut dest = MaybeUninit::<Self>::uninit();
        unsafe {
            if gl::CreateVertexArrays::is_loaded() {
                gl::CreateVertexArrays(1, dest.as_mut_ptr() as *mut _);
            } else {
                gl::GenVertexArrays(1, dest.as_mut_ptr() as *mut _);
                gl::BindVertexArray(dest.get_mut().id());
                gl::BindVertexArray(0);
            }
            dest.assume_init()
        }

    }

    pub fn create_vertex_arrays(#[allow(unused_variables)] gl:&GL30, n:GLuint) -> Box<[Self]> {
        if n==0 { return Box::new([]); }
        let mut dest = Box::<[Self]>::new_uninit_slice(n as usize);
        unsafe {
            if gl::CreateVertexArrays::is_loaded() {
                gl::CreateVertexArrays(dest.len().try_into().unwrap(), dest[0].as_mut_ptr() as *mut GLuint);
            } else {
                gl::GenVertexArrays(dest.len().try_into().unwrap(), dest[0].as_mut_ptr() as *mut GLuint);
                for arr in dest.iter_mut() { gl::BindVertexArray(arr.get_mut().id()) }
                gl::BindVertexArray(0);
            }
            dest.assume_init()
        }
    }

}

impl<'a,E:Copy,V:Vertex<'a>> VertexArray<'a,E,V> {
    #[inline] pub fn id(&self) -> GLuint { self.id }
    #[inline] pub fn gl(&self) -> GL30 { unsafe { assume_supported() } }

    #[inline] pub fn is(id: GLuint) -> bool { unsafe { gl::IsVertexArray(id) != 0 } }

    #[inline] pub fn delete(self) { drop(self); }
    #[inline] pub fn delete_vertex_arrays(arrays: Box<[Self]>) {
        if arrays.len()==0 { return; }
        unsafe {
            let ids: Box<[GLuint]> = transmute(arrays);
            gl::DeleteVertexArrays(ids.len() as GLsizei, &ids[0]);
        }
    }

    #[inline] pub fn attribs<'r>(&'r self) -> V::Attribs where V:VertexRef<'r,'a> { V::attribs(self) }
    #[inline] pub fn attribs_mut<'r>(&'r mut self) -> V::AttribsMut where V:VertexRef<'r,'a> {
        V::attribs_mut(self)
    }

    #[inline] pub fn get_attrib_arrays(&self) -> V::Arrays { V::get_attrib_arrays(self) }
    #[inline] pub fn attrib_arrays(&mut self, arrays: V::Arrays) { V::attrib_arrays(self, arrays) }

    #[inline]
    pub fn append_attrib_arrays<A>(self, arrays: A) -> VertexArray<'a,E,V::Output> where
        V:VertexAppend<'a,A>
    {
        V::append_arrays(self, arrays)
    }

}

impl<'a,V:Vertex<'a>> VertexArray<'a,!,V> {
    #[inline]
    pub fn bind_element_buffer<E:Element,A:Initialized>(
        self, elements: &'a Buffer<[E], A>
    ) -> VertexArray<'a,E,V> {
        let mut dest:VertexArray<'a,E,V> = VertexArray { id: self.id(), buffers: PhantomData };
        dest.bind_element_buffer(elements);
        forget(self);
        dest
    }

    #[inline]
    pub fn bind_element_buffer_from<E:Element>(self, elements: &VertexArray<'a,E,V>) -> VertexArray<'a,E,V> {
        let mut dest:VertexArray<'a,E,V> = VertexArray { id: self.id(), buffers: PhantomData };
        dest.bind_element_buffer_from(elements);
        forget(self);
        dest
    }
}

impl<'a,E:Element,V:Vertex<'a>> VertexArray<'a,E,V> {
    #[inline]
    pub fn bind_element_buffer<A:Initialized>(&mut self, elements: &'a Buffer<[E], A>) {
        unsafe {
            //we don't use the binding locations for ELEMENT_ARRAY_BUFFER here in order to make sure
            //that we don't accidentally unbind the element buffer
            VERTEX_ARRAY.map_bind(
                self, |_| gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, elements.id())
            );
        }
    }

    fn get_element_buffer_id(&self) -> GLuint {
        let mut id = MaybeUninit::uninit();
        unsafe {
            if gl::GetVertexArrayiv::is_loaded() {
                gl::GetVertexArrayiv(self.id(), gl::ELEMENT_ARRAY_BUFFER_BINDING, id.as_mut_ptr());
            } else {
                VERTEX_ARRAY.map_bind(
                    self, |_| gl::GetIntegerv(gl::ELEMENT_ARRAY_BUFFER_BINDING, id.as_mut_ptr())
                );
            }
            id.assume_init() as GLuint
        }
    }

    #[inline]
    pub fn bind_element_buffer_from(&mut self, elements: &VertexArray<'a,E,V>) {
        let id = elements.get_element_buffer_id();
        unsafe {
            VERTEX_ARRAY.map_bind(
                self, |_| gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id)
            );
        }
    }

    #[inline]
    pub fn get_element_buffer(&self) -> Slice<'a,[E],ReadOnly> {
        let id = self.get_element_buffer_id();
        unsafe {
            let size = BufPtr::<()>::new(id, null_mut()).buffer_size();
            Slice::from_raw_parts(id, size / size_of::<E>(), 0)
        }
    }

}


impl<'a,E:Copy,V:Vertex<'a>> Drop for VertexArray<'a,E,V> {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, &self.id()); }
    }
}

// impl<'a,E:Copy,V:Vertex<'a>> Clone for VertexArray<'a,E,V> {
//     fn clone(&self) -> Self {
//
//         //copy over all the array settings
//         let dest: VertexArray<'a,!,V> = VertexArray::create(&self.gl()).append_attrib_arrays(self.get_attrib_arrays());
//         let mut dest:Self = unsafe { transmute(dest) };
//
//         unsafe { gl::BindVertexArray(self.id()); }
//
//         //get the divisors
//         let num_divisors = if gl::VertexAttribDivisor::is_loaded() { V::num_indices() } else { 0 };
//         let divisors = (0..num_divisors).into_iter().map(
//             |i| {
//                 let mut div = MaybeUninit::uninit();
//                 unsafe {
//                     gl::GetVertexAttribiv(i as GLuint, gl::VERTEX_ATTRIB_ARRAY_DIVISOR, div.as_mut_ptr());
//                     div.assume_init()
//                 }
//             }
//         ).collect::<Vec<_>>();
//
//         //get the element array id
//         let buf = unsafe {
//             let mut id = MaybeUninit::uninit();
//             gl::GetIntegerv(gl::ELEMENT_ARRAY_BUFFER_BINDING, id.as_mut_ptr());
//             id.assume_init()
//         };
//
//         unsafe { gl::BindVertexArray((&mut dest).id()); }
//
//         //set the divisors and element array
//         unsafe {
//             for i in 0..num_divisors { gl::VertexAttribDivisor(i as GLuint, divisors[i] as GLuint); }
//             gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buf as GLuint);
//         }
//
//         unsafe { gl::BindVertexArray(0); }
//
//         dest
//     }
// }
