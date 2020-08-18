use super::*;

macro_rules! wrap_cmp {
    ($gen:ident; $($name:ident $op:tt)*) => {
        $(
            #[allow(non_snake_case)]
            pub fn $name<V:$gen+GenVec>(x:V, y:V) -> V::BVec {
                unsafe {
                    let mut dest = MaybeUninit::<V::BVec>::uninit();
                    for i in 0..V::COUNT {
                        *dest.get_mut().coord_mut(i) = (x.coord(i) $op y.coord(i)).into();
                    }
                    dest.assume_init()
                }
            }
        )*
    };
}

wrap_cmp!(GenOrdType; lessThan < lessThanEqual <= greaterThan > greaterThanEqual >= );
wrap_cmp!(GenEqType; equal == notEqual != );

pub fn any<V:GenBType>(x:V) -> gl_bool {
    for i in 0..V::COUNT { if bool::from(*x.coord(i)) { return true.into(); } }
    false.into()
}

pub fn all<V:GenBType>(x:V) -> gl_bool {
    for i in 0..V::COUNT { if bool::from(!(*x.coord(i))) { return false.into(); } }
    true.into()
}

pub fn not<V:GenBType>(mut x:V) -> V {
    for i in 0..V::COUNT { *x.coord_mut(i) = !*x.coord(i); }
    x
}
