use super::*;


pub trait Vertex<'a>: Sized {
    type Arrays: Copy;

    fn num_indices() -> usize;

    fn get_attrib_arrays<'r,El:Copy>(vaobj: &'r VertexArray<'a,El,Self>) -> Self::Arrays;
    fn attrib_arrays<'r,El:Copy>(vaobj: &'r mut VertexArray<'a,El,Self>, pointers: Self::Arrays);

}


pub trait VertexRef<'a,'b:'a>: Vertex<'b> {
    type Attribs;
    type AttribsMut;

    fn attribs<El:Copy>(vaobj: &'a VertexArray<'b,El,Self>) -> Self::Attribs;
    fn attribs_mut<El:Copy>(vaobj: &'a mut VertexArray<'b,El,Self>) -> Self::AttribsMut;

}

pub trait VertexAppend<'a, V:Vertex<'a>>: Vertex<'a> {
    type Output: Vertex<'a>;

    fn append_arrays<El:Copy>(
        vaobj: VertexArray<'a,El,Self>, pointers: V::Arrays
    ) -> VertexArray<'a,El,Self::Output>;

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
            fn append_arrays<El:Copy>(
                vaobj: VertexArray<'a,El,Self>, pointers: <($($T2,)*) as Vertex<'a>>::Arrays
            ) -> VertexArray<'a,El,Self::Output> {
                let mut dest = VertexArray { id: vaobj.id(), buffers: PhantomData };
                forget(vaobj);

                let ($($t1,)* $(mut $t2,)*) = Self::Output::attribs_mut(&mut dest);
                let ($($T2,)*) = pointers;
                $(
                    $t2.enable_array();
                    $t2.pointer($T2);
                )*

                dest
            }

        }

        impl_append!(@next {$($T1:$t1)*} $($T2:$t2)*);
    };
}

macro_rules! impl_vertex_ref {

    ($($T:ident:$t:ident)*) => {

        impl<'a,$($T:GLSLType+'a),*> Vertex<'a> for ($($T,)*) {
            type Arrays = ($(AttribArray<'a,$T>,)*);

            #[inline] fn num_indices() -> usize { 0 $( + $T::AttribFormat::attrib_count())* }

            fn get_attrib_arrays<'r,El:Copy>(vaobj: &'r VertexArray<'a,El,Self>) -> Self::Arrays {
                let ($($t,)*) = Self::attribs(vaobj);
                ($($t.get_array(),)*)
            }

            #[allow(non_snake_case)]
            fn attrib_arrays<'r,El:Copy>(vaobj: &'r mut VertexArray<'a,El,Self>, pointers: Self::Arrays) {
                let ($(mut $t,)*) = Self::attribs_mut(vaobj);
                let ($($T,)*) = pointers;
                $($t.pointer($T);)*
            }

        }

        impl<'a,'b:'a,$($T:GLSLType+'b),*> VertexRef<'a,'b> for ($($T,)*) {
            type Attribs = ($(VertexAttrib<'a,'b,$T>,)*);
            type AttribsMut = ($(VertexAttribMut<'a,'b,$T>,)*);

            #[allow(unused_variables, unused_assignments)]
            fn attribs<El:Copy>(vaobj: &'a VertexArray<'b,El,Self>) -> Self::Attribs {
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
            fn attribs_mut<El:Copy>(vaobj: &'a mut VertexArray<'b,El,Self>) -> Self::AttribsMut {
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

        impl_append!(@next {} $($T:$t)*);

    };
}

impl_tuple!(impl_vertex_ref);

impl<'a> Vertex<'a> for () {
    type Arrays = ();
    fn num_indices() -> usize { 0 }
    fn get_attrib_arrays<'r,El:Copy>(_: &'r VertexArray<'a,El,Self>) -> () { () }
    fn attrib_arrays<'r,El:Copy>(_: &'r mut VertexArray<'a,El,Self>, _: ()) {}
}

impl<'a,'b:'a> VertexRef<'a,'b> for () {
    type Attribs = ();
    type AttribsMut = ();
    fn attribs<El:Copy>(_: &'a VertexArray<'b,El,Self>) -> () { () }
    fn attribs_mut<El:Copy>(_: &'a mut VertexArray<'b,El,Self>) -> () { () }
}

impl<'a,V:Vertex<'a>> VertexAppend<'a,V> for () {
    type Output = V;
    fn append_arrays<El:Copy>(vaobj: VertexArray<'a,El,()>, arrays: V::Arrays) -> VertexArray<'a,El,V> {
        let mut dest = VertexArray { id: vaobj.id(), buffers: PhantomData };
        forget(vaobj);
        V::attrib_arrays(&mut dest, arrays);
        dest
    }
}
