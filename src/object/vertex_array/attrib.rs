use super::*;

pub struct VertexAttrib<'a, 'b, T:GLSLType> {
    pub(super) vaobj: GLuint,
    pub(super) index: GLuint,
    pub(super) reference: PhantomData<&'b mut VertexArray<'a,T>>
}

impl<'a,'b,T:GLSLType> VertexAttrib<'a,'b,T> {
    #[inline] pub fn index(&self) -> GLuint { self.index }

    unsafe fn get(&self, pname:GLenum) -> GLint {
        let mut dest = MaybeUninit::uninit();
        if gl::GetVertexArrayIndexediv::is_loaded() {
            gl::GetVertexArrayIndexediv(self.vaobj, self.index, pname, dest.as_mut_ptr());
        } else {
            gl::BindVertexArray(self.vaobj);
            gl::GetVertexAttribiv(self.index, pname, dest.as_mut_ptr());
            gl::BindVertexArray(0);
        }
        dest.assume_init()
    }

    unsafe fn get_64(&self, pname:GLenum) -> GLint64 {
        let mut dest = MaybeUninit::uninit();
        if gl::GetVertexArrayIndexed64iv::is_loaded() {
            gl::GetVertexArrayIndexed64iv(self.vaobj, self.index, pname, dest.as_mut_ptr());
            dest.assume_init()
        } else {
            self.get(pname) as GLint64
        }
    }

    pub fn array_enabled(&self) -> bool { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_ENABLED) != 0 } }
    pub fn array_size(&self) -> usize { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_SIZE) as usize } }
    pub fn array_stride(&self) -> usize { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_STRIDE) as usize } }
    pub fn array_normalized(&self) -> bool { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_NORMALIZED) != 0 } }
    pub fn array_integer(&self) -> bool { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_INTEGER) != 0 } }
    pub fn array_long(&self) -> bool { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_LONG) != 0 } }
    pub fn array_divisor(&self) -> usize { unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_DIVISOR) as usize } }
    pub fn relative_offset(&self) -> usize { unsafe { self.get_64(gl::VERTEX_ATTRIB_RELATIVE_OFFSET) as usize } }

    pub fn array_type(&self) -> AttribType {
        unsafe { (self.get(gl::VERTEX_ATTRIB_ARRAY_TYPE) as GLenum).try_into().unwrap() }
    }

    pub unsafe fn enable_array(&mut self) {
        if gl::EnableVertexArrayAttrib::is_loaded() {
            for i in self.index .. T::AttribFormat::attrib_count() as GLuint {
                gl::EnableVertexArrayAttrib(self.vaobj, i);
            }
        } else {
            gl::BindVertexArray(self.vaobj);
            for i in self.index .. T::AttribFormat::attrib_count() as GLuint {
                gl::EnableVertexAttribArray(i);
            }
            gl::BindVertexArray(0);
        }
    }

    pub unsafe fn disable_array(&mut self) {
        if gl::DisableVertexArrayAttrib::is_loaded() {
            for i in self.index .. T::AttribFormat::attrib_count() as GLuint {
                gl::DisableVertexArrayAttrib(self.vaobj, i);
            }
        } else {
            gl::BindVertexArray(self.vaobj);
            for i in self.index .. T::AttribFormat::attrib_count() as GLuint {
                gl::DisableVertexAttribArray(i);
            }
            gl::BindVertexArray(0);
        }
    }

    pub fn pointer(&mut self, pointer: AttribArray<'a,T>) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, pointer.id());
            gl::BindVertexArray(self.vaobj);

            let fmt = pointer.format();
            let (stride, pointer) = (pointer.stride() as GLsizei, pointer.pointer());

            for i in 0..T::AttribFormat::attrib_count() {
                let index = self.index + i as GLuint;
                let (size, ty, norm) = (fmt.size(i) as GLint, fmt.ty(i) as GLenum, fmt.normalized(i) as GLboolean);
                let ptr = pointer.offset(fmt.offset(i) as isize);

                match (fmt.integer(i), fmt.long(i)) {
                    (false, false) => gl::VertexAttribPointer(index, size, ty, norm, stride, ptr),
                    (true,  false) => gl::VertexAttribIPointer(index, size, ty, stride, ptr),
                    (false, true)  => gl::VertexAttribLPointer(index, size, ty, stride, ptr),
                    (true,  true)  => panic!("Long-integer attribute arrays not currently supported by the GL"),
                }
            }

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    pub fn divisor(&mut self, divisor: GLuint) {
        unsafe {
            gl::BindVertexArray(self.vaobj);
            for i in self.index .. T::AttribFormat::attrib_count() as GLuint {
                gl::VertexAttribDivisor(i, divisor);
            }
            gl::BindVertexArray(0);
        }
    }

}
