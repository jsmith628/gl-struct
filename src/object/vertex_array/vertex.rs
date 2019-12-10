use super::*;


pub trait Vertex<'a>: GLSLType {
    type AttribArrays: Copy;

    fn num_indices() -> usize;

    fn attrib_arrays<'r>(vaobj: &'r VertexArray<'a,Self>) -> Self::AttribArrays;
    fn attrib_array_pointers<'r>(vaobj: &'r mut VertexArray<'a,Self>, pointers: Self::AttribArrays);

}


pub trait VertexRef<'a,'b:'a>: Vertex<'b> {
    type VertexAttribs;
    type VertexAttribsMut;

    fn attribs(vaobj: &'a VertexArray<'b,Self>) -> Self::VertexAttribs;
    fn attribs_mut(vaobj: &'a mut VertexArray<'b,Self>) -> Self::VertexAttribsMut;

}

pub trait VertexAppend<'a, V:Vertex<'a>>: Vertex<'a> {
    type Output: Vertex<'a>;
    fn append_pointers(vaobj: VertexArray<'a,Self>, pointers: V::AttribArrays) -> VertexArray<'a,Self::Output>;
}

macro_rules! impl_append {
    (@next {$($T1:ident:$t1:ident)*}) => {};
    (@next {$($T1:ident:$t1:ident)*} $T2:ident:$t2:ident $($rest:tt)*) => {
        impl_append!({$($T1:$t1)* $T2:$t2 } $($rest)*);
    };

    ({$($T1:ident:$t1:ident)*} $($T2:ident:$t2:ident)*) => {

        impl<'a,$($T1:GLSLType+'a,)* $($T2:GLSLType+'a),*> VertexAppend<'a,($($T2,)*)> for ($($T1,)*) {
            type Output = ($($T1,)* $($T2,)*);

            #[allow(unused_variables, non_snake_case)]
            fn append_pointers(
                vaobj: VertexArray<'a,Self>, pointers: <($($T2,)*) as Vertex<'a>>::AttribArrays
            ) -> VertexArray<'a,Self::Output> {
                let mut dest = VertexArray { id: vaobj.id(), buffers: PhantomData };
                forget(vaobj);

                let ($($t1,)* $(mut $t2,)*) = Self::Output::attribs_mut(&mut dest);
                let ($($T2,)*) = pointers;
                $($t2.pointer($T2);)*

                dest
            }

        }

        impl_append!(@next {$($T1:$t1)*} $($T2:$t2)*);
    };
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

        impl_append!({} $($T:$t)*);

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

impl<'a> VertexAppend<'a,()> for () {
    type Output = ();
    fn append_pointers(vaobj: VertexArray<'a,()>, _: ()) -> VertexArray<'a,()> {vaobj}
}
