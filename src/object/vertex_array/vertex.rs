use super::*;


pub trait VertexRef<'a:'b,'b>: GLSLType {
    type AttribArrays: Copy;
    type VertexAttribs;

    fn vertex_attribs(vaobj: &'b VertexArray<'a,Self>) -> <Self as VertexRef<'a,'b>>::VertexAttribs;

}

pub trait Vertex = for<'a,'b> VertexRef<'a,'b>;

macro_rules! impl_vertex_ref {
    ($($T:ident:$t:ident)*) => {

        impl<'a:'b,'b,$($T:GLSLType+'a),*> VertexRef<'a,'b> for ($($T,)*) {
            type AttribArrays = ($(AttribArray<'a,$T>,)*);
            type VertexAttribs = ($(VertexAttrib<'a,'b,$T>,)*);

            #[allow(unused_variables, unused_assignments)]
            fn vertex_attribs(vaobj: &'b VertexArray<'a,Self>) -> <Self as VertexRef<'a,'b>>::VertexAttribs {
                let mut i = 0;

                ($(
                    VertexAttrib {
                        vaobj: vaobj.id(),
                        index: {
                            let j=i;
                            i+= <$T::AttributeFormat as AttribFormat>::attrib_count();
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

impl<'a:'b,'b> VertexRef<'a,'b> for () {
    type AttribArrays = ();
    type VertexAttribs = ();
    fn vertex_attribs(_: &'b VertexArray<'a,Self>) -> () { () }
}
