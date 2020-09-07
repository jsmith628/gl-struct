use super::*;

macro_rules! component_wrapping_fn {

    ($gen_type:ident;) => {};

    ($gen_type:ident; $name:ident:$fun:ident($v:ident $(, $args:ident)*) $($rest:tt)*) => {

        pub fn $name<V:$gen_type>(mut $v:V, $($args:V),*) -> V {
            for i in 0..V::COUNT {
                *$v.coord_mut(i) = $v.coord(i).$fun($(*$args.coord(i)),*);
            }
            $v
        }

        component_wrapping_fn!($gen_type; $($rest)*);
    };

    ($gen_type:ident; $name:ident:$fun:ident $($rest:tt)*) => {
        component_wrapping_fn!($gen_type; $name:$fun(x) $($rest)*);
    };
}

component_wrapping_fn!{GenFType;
    radians:to_radians degrees:to_degrees
    sin:sin cos:cos tan:tan asin:asin acos:acos atan:atan atan2:atan2(y,x)
    sinh:sinh cosh:cosh tanh:tanh asinh:asinh acosh:acosh atanh:atanh

    pow:powf(x,y) exp:exp log:ln exp2:exp2 log2:log2

    fma:mul_add(a,b,c)

}

pub fn inversesqrt<V:GenFType>(mut x:V) -> V {
    for i in 0..V::COUNT {
        *x.coord_mut(i) = x.coord(i).sqrt().recip();
    }
    x
}

component_wrapping_fn!{GenFloatType;
    floor:floor ceil:ceil trunc:trunc fract:fract round:round sqrt:sqrt
}

#[allow(non_snake_case)]
pub fn roundEven<V:GenFloatType>(mut x:V) -> V {
    for i in 0..V::COUNT {
        let two = V::Component::one() + One::one();
        if x.coord(i).fract() == two.recip() {
            if x.coord(i).floor() % two == Zero::zero() {
                *x.coord_mut(i) = x.coord(i).floor();
            } else {
                *x.coord_mut(i) = x.coord(i).ceil();
            }
        } else {
            *x.coord_mut(i) = x.coord(i).round();
        }
    }
    x
}

component_wrapping_fn!{GenSignType; abs:abs sign:signum}

pub fn isnan<V:GenFloatType+GenFamily>(x:V) -> V::BVec {
    unsafe {
        let mut dest = MaybeUninit::<V::BVec>::uninit();
        for i in 0..V::COUNT {
            *dest.assume_init_mut().coord_mut(i) = (x.coord(i).is_nan()).into();
        }
        dest.assume_init()
    }
}

pub fn isinf<V:GenFloatType+GenFamily>(x:V) -> V::BVec {
    unsafe {
        let mut dest = MaybeUninit::<V::BVec>::uninit();
        for i in 0..V::COUNT {
            *dest.assume_init_mut().coord_mut(i) = (x.coord(i).is_infinite()).into();
        }
        dest.assume_init()
    }
}

#[allow(non_snake_case)]
pub fn floatBitsToInt<V:GenFType+GenFamily>(x:V) -> V::IVec {
    unsafe {
        let mut dest = MaybeUninit::<V::IVec>::uninit();
        for i in 0..V::COUNT {
            *dest.assume_init_mut().coord_mut(i) = (*x.coord(i)).to_bits() as i32;
        }
        dest.assume_init()
    }
}

#[allow(non_snake_case)]
pub fn floatBitsToUint<V:GenFType+GenFamily>(x:V) -> V::UVec {
    unsafe {
        let mut dest = MaybeUninit::<V::UVec>::uninit();
        for i in 0..V::COUNT {
            *dest.assume_init_mut().coord_mut(i) = (*x.coord(i)).to_bits();
        }
        dest.assume_init()
    }
}

#[allow(non_snake_case)]
pub fn intBitsToFloat<V:GenIType+GenFamily>(x:V) -> V::Vec {
    unsafe {
        let mut dest = MaybeUninit::<V::Vec>::uninit();
        for i in 0..V::COUNT {
            *dest.assume_init_mut().coord_mut(i) = f32::from_bits(*x.coord(i) as u32);
        }
        dest.assume_init()
    }
}

#[allow(non_snake_case)]
pub fn uintBitsToFloat<V:GenUType+GenFamily>(x:V) -> V::UVec {
    unsafe {
        let mut dest = MaybeUninit::<V::UVec>::uninit();
        for i in 0..V::COUNT {
            *dest.assume_init_mut().coord_mut(i) = transmute(*x.coord(i));
        }
        dest.assume_init()
    }
}
