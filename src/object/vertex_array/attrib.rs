use super::*;

pub struct VertexAttrib<'a, 'b, T:GLSLType> {
    pub(super) vaobj: GLuint,
    pub(super) index: GLuint,
    pub(super) reference: PhantomData<&'b mut VertexArray<'a,T>>
}

impl<'a,'b,T:GLSLType> VertexAttrib<'a,'b,T> {
    #[inline] pub fn index(&self) -> GLuint { self.index }
    #[inline] pub fn num_indices(&self) -> usize { T::AttribFormat::attrib_count() }

    unsafe fn get(&self, pname:GLenum, i:GLuint) -> GLint {
        let mut dest = MaybeUninit::uninit();
        if gl::GetVertexArrayIndexediv::is_loaded() {
            gl::GetVertexArrayIndexediv(self.vaobj, self.index+i, pname, dest.as_mut_ptr());
        } else {
            gl::BindVertexArray(self.vaobj);
            gl::GetVertexAttribiv(self.index+i, pname, dest.as_mut_ptr());
            gl::BindVertexArray(0);
        }
        dest.assume_init()
    }

    unsafe fn get_pointer(&self, i:GLuint) -> *const GLvoid {
        if gl::GetVertexArrayIndexed64iv::is_loaded() {
            let mut dest = MaybeUninit::uninit();
            gl::GetVertexArrayIndexed64iv(
                self.vaobj, self.index+i, gl::VERTEX_ATTRIB_RELATIVE_OFFSET, dest.as_mut_ptr()
            );
            dest.assume_init() as usize as *const GLvoid
        } else {
            let mut dest = MaybeUninit::uninit();
            gl::BindVertexArray(self.vaobj);
            gl::GetVertexAttribPointerv(self.index+i, gl::VERTEX_ATTRIB_ARRAY_POINTER, dest.as_mut_ptr());
            gl::BindVertexArray(0);
            dest.assume_init()
        }
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

    pub fn array_enabled(&self) -> bool {
        unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_ENABLED, 0) != 0 }
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

    pub fn get_format(&self) -> T::AttribFormat {

        let ptr = unsafe { self.get_pointer(0) };

        let layouts = (0 .. self.num_indices() as GLuint).into_iter().map(
            |i| unsafe {
                AttribLayout {
                    offset: self.get_pointer(i).offset_from(ptr) as usize,
                    size: self.get(gl::VERTEX_ATTRIB_ARRAY_SIZE, i) as GLenum,
                    ty: (self.get(gl::VERTEX_ATTRIB_ARRAY_TYPE, i) as GLenum).try_into().unwrap(),
                    normalized: self.get(gl::VERTEX_ATTRIB_ARRAY_NORMALIZED,i) != 0
                }
            }
        ).collect::<Vec<_>>();

        T::AttribFormat::from_layouts(&*layouts).unwrap()

    }

    pub fn get_array(&self) -> AttribArray<'a,T> {
        unsafe {
            AttribArray::from_raw_parts(
                self.get_format(),
                self.get(gl::VERTEX_ATTRIB_ARRAY_BUFFER_BINDING, 0) as GLuint,
                self.get(gl::VERTEX_ATTRIB_ARRAY_STRIDE, 0) as usize,
                self.get_pointer(0) as usize
            )
        }
    }

    pub fn get_divisor(&self) -> GLuint {
        unsafe { self.get(gl::VERTEX_ATTRIB_ARRAY_DIVISOR, 0) as GLuint }
    }

}
