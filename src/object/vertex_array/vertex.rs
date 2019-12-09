use super::*;


pub trait VertexRef<'a,'b:'a>: GLSLType {
    type AttribArrays: Copy;
    type VertexAttribs;

    fn num_indices() -> usize;
    fn vertex_attribs(vaobj: &'a VertexArray<'b,Self>) -> <Self as VertexRef<'a,'b>>::VertexAttribs;

}

pub trait Vertex = for<'a,'b> VertexRef<'a,'b>;

macro_rules! impl_vertex_ref {
    ($($T:ident:$t:ident)*) => {

        impl<'a,'b:'a,$($T:GLSLType+'b),*> VertexRef<'a,'b> for ($($T,)*) {
            type AttribArrays = ($(AttribArray<'b,$T>,)*);
            type VertexAttribs = ($(VertexAttrib<'a,'b,$T>,)*);

            #[inline] fn num_indices() -> usize { 0 $( + $T::AttribFormat::attrib_count())* }

            #[allow(unused_variables, unused_assignments)]
            fn vertex_attribs(vaobj: &'a VertexArray<'b,Self>) -> <Self as VertexRef<'a,'b>>::VertexAttribs {
                let mut i = 0;

                ($(
                    VertexAttrib {
                        vaobj: vaobj.id(),
                        index: {
                            let j=i;
                            i+= <$T::AttribFormat as AttribFormat>::attrib_count();
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

impl<'a,'b:'a> VertexRef<'a,'b> for () {
    type AttribArrays = ();
    type VertexAttribs = ();

    fn num_indices() -> usize { 0 }
    fn vertex_attribs(_: &'a VertexArray<'b,Self>) -> () { () }
}
