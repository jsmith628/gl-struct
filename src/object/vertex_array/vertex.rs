use super::*;


pub trait VertexRef<'a,'b:'a>: GLSLType {
    type AttribArrays: Copy;
    type VertexAttribs;
    type VertexAttribsMut;

    fn num_indices() -> usize;

    fn attribs(vaobj: &'a VertexArray<'b,Self>) -> <Self as VertexRef<'a,'b>>::VertexAttribs;
    fn attribs_mut(vaobj: &'a mut VertexArray<'b,Self>) -> <Self as VertexRef<'a,'b>>::VertexAttribsMut;

    fn attrib_arrays(vaobj: &'a VertexArray<'b,Self>) -> <Self as VertexRef<'a,'b>>::AttribArrays;
    fn attrib_array_pointers(vaobj: &'a mut VertexArray<'b,Self>, pointers: <Self as VertexRef<'a,'b>>::AttribArrays);

}

pub trait Vertex = for<'a,'b> VertexRef<'a,'b>;

macro_rules! impl_vertex_ref {
    ($($T:ident:$t:ident)*) => {

        impl<'a,'b:'a,$($T:GLSLType+'b),*> VertexRef<'a,'b> for ($($T,)*) {
            type AttribArrays = ($(AttribArray<'b,$T>,)*);
            type VertexAttribs = ($(VertexAttrib<'a,'b,$T>,)*);
            type VertexAttribsMut = ($(VertexAttribMut<'a,'b,$T>,)*);

            #[inline] fn num_indices() -> usize { 0 $( + $T::AttribFormat::attrib_count())* }

            #[allow(unused_variables, unused_assignments)]
            fn attribs(vaobj: &'a VertexArray<'b,Self>) -> Self::VertexAttribs {
                let mut i = 0;
                ($(
                    VertexAttrib {
                        vaobj: vaobj.id(),
                        index: {
                            let j=i;
                            i += <$T::AttribFormat as AttribFormat>::attrib_count();
                            j as GLuint
                        },
                        reference: PhantomData
                    }
                ,)*)

            }

            #[allow(unused_variables, unused_assignments)]
            fn attribs_mut(vaobj: &'a mut VertexArray<'b,Self>) -> Self::VertexAttribsMut {
                let mut i = 0;
                ($(
                    VertexAttribMut {
                        vaobj: vaobj.id(),
                        index: {
                            let j=i;
                            i += <$T::AttribFormat as AttribFormat>::attrib_count();
                            j as GLuint
                        },
                        reference: PhantomData
                    }
                ,)*)

            }

            fn attrib_arrays(vaobj: &'a VertexArray<'b,Self>) -> Self::AttribArrays {
                let ($($t,)*) = Self::attribs(vaobj);
                ($($t.get_array(),)*)
            }

            #[allow(non_snake_case)]
            fn attrib_array_pointers(vaobj: &'a mut VertexArray<'b,Self>, pointers: Self::AttribArrays) {
                let ($(mut $t,)*) = Self::attribs_mut(vaobj);
                let ($($T,)*) = pointers;
                $($t.pointer($T);)*
            }

        }

    };
}

impl_tuple!(impl_vertex_ref);

impl<'a,'b:'a> VertexRef<'a,'b> for () {
    type AttribArrays = ();
    type VertexAttribs = ();
    type VertexAttribsMut = ();

    fn num_indices() -> usize { 0 }
    fn attribs(_: &'a VertexArray<'b,Self>) -> () { () }
    fn attribs_mut(_: &'a mut VertexArray<'b,Self>) -> () { () }
    fn attrib_arrays(_: &'a VertexArray<'b,Self>) -> () { () }
    fn attrib_array_pointers(_: &'a mut VertexArray<'b,Self>, _: ()) {}
}
