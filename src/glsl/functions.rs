use super::*;

macro_rules! component_wrapping_fn {
    ($gen_type:ident; $($name:ident:$fun:ident)*) => {
        $(
            pub fn $name<V:$gen_type>(mut v:V) -> V {
                for i in 0..v.length() { *v.coord_mut(i as u32) = v.coord(i as u32).$fun(); }
                return v;
            }
        )*
    }
}

component_wrapping_fn!{GenFType;
    radians:to_radians degrees:to_degrees
    sin:sin cos:cos tan:tan asin:asin acos:acos atan:atan
    sinh:sinh cosh:cosh tanh:tanh asinh:asinh acosh:acosh atanh:atanh

    exp:exp log:ln exp2:exp2 log2:log2 

}
