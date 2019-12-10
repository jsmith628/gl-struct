use super::*;

use std::marker::PhantomData;
use std::mem::*;

use object::buffer::AttribArray;
use format::attribute::*;
use glsl::GLSLType;

pub use self::attrib::*;
pub use self::vertex::*;

mod attrib;
mod vertex;

#[repr(transparent)]
pub struct VertexArray<'a,V:Vertex<'a>> {
    id: GLuint,
    buffers: PhantomData<(&'a Buffer<GLuint, ReadOnly>, V::Arrays)>
}

impl<'a> VertexArray<'a,()> {
    pub fn gen(#[allow(unused_variables)] gl:&GL30) -> Self {
        let mut dest = MaybeUninit::uninit();
        unsafe {
            gl::GenVertexArrays(1, dest.as_mut_ptr() as *mut _);
            dest.assume_init()
        }
    }

    pub fn gen_vertex_arrays(#[allow(unused_variables)] gl:&GL30, n:GLuint) -> Box<[Self]> {
        if n==0 { return Box::new([]); }
        let mut dest = Box::new_uninit_slice(n as usize);
        unsafe {
            gl::GenVertexArrays(dest.len().try_into().unwrap(), dest[0].as_mut_ptr() as *mut GLuint);
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

impl<'a,V:Vertex<'a>> VertexArray<'a,V> {
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
    pub fn append_attrib_arrays<V2:Vertex<'a>>(self, arrays: V2::Arrays) -> VertexArray<'a,V::Output> where
        V:VertexAppend<'a,V2>
    {
        V::append_arrays(self, arrays)
    }


}

impl<'a,V:Vertex<'a>> Drop for VertexArray<'a,V> {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, &self.id()); }
    }
}
