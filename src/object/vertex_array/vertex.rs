use super::*;


pub trait Vertex<'a>: GLSLType {
    type AttribArrays: Copy;

    fn num_indices() -> usize;

    fn attrib_arrays<'r>(vaobj: &'r VertexArray<'a,Self>) -> <Self as Vertex<'a>>::AttribArrays;
    fn attrib_array_pointers<'r>(vaobj: &'r mut VertexArray<'a,Self>, pointers: <Self as Vertex<'a>>::AttribArrays);

}


pub trait VertexRef<'a,'b:'a>: Vertex<'b> {
    type VertexAttribs;
    type VertexAttribsMut;

    fn attribs(vaobj: &'a VertexArray<'b,Self>) -> <Self as VertexRef<'a,'b>>::VertexAttribs;
    fn attribs_mut(vaobj: &'a mut VertexArray<'b,Self>) -> <Self as VertexRef<'a,'b>>::VertexAttribsMut;

}

macro_rules! impl_vertex_ref {
    ($($T:ident:$t:ident)*) => {

        impl<'a,$($T:GLSLType+'a),*> Vertex<'a> for ($($T,)*) {
            type AttribArrays = ($(AttribArray<'a,$T>,)*);

            #[inline] fn num_indices() -> usize { 0 $( + $T::AttribFormat::attrib_count())* }

            fn attrib_arrays<'r>(vaobj: &'r VertexArray<'a,Self>) -> Self::AttribArrays {
                let ($($t,)*) = Self::attribs(vaobj);
                ($($t.get_array(),)*)
            }

            #[allow(non_snake_case)]
            fn attrib_array_pointers<'r>(vaobj: &'r mut VertexArray<'a,Self>, pointers: Self::AttribArrays) {
                let ($(mut $t,)*) = Self::attribs_mut(vaobj);
                let ($($T,)*) = pointers;
                $($t.pointer($T);)*
            }

        }

        impl<'a,'b:'a,$($T:GLSLType+'b),*> VertexRef<'a,'b> for ($($T,)*) {
            type VertexAttribs = ($(VertexAttrib<'a,'b,$T>,)*);
            type VertexAttribsMut = ($(VertexAttribMut<'a,'b,$T>,)*);

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

        }

    };
}

impl_tuple!(impl_vertex_ref);

impl<'a> Vertex<'a> for () {
    type AttribArrays = ();
    fn num_indices() -> usize { 0 }
    fn attrib_arrays<'r>(_: &'r VertexArray<'a,Self>) -> () { () }
    fn attrib_array_pointers<'r>(_: &'r mut VertexArray<'a,Self>, _: ()) {}
}

impl<'a,'b:'a> VertexRef<'a,'b> for () {
    type VertexAttribs = ();
    type VertexAttribsMut = ();
    fn attribs(_: &'a VertexArray<'b,Self>) -> () { () }
    fn attribs_mut(_: &'a mut VertexArray<'b,Self>) -> () { () }
}
