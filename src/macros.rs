

macro_rules! display_from_debug {
    ($name:ty) => {
        impl ::std::fmt::Display for $name {
            #[inline]
            fn fmt(&self,f:  &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                ::std::fmt::Debug::fmt(self, f)
            }
        }
    }
}


///A helper macro for contructing by-value stack arrays from a list comprehension
macro_rules! arr {
    (for $i:ident in 0..$n:literal { $expr:expr }) => { arr![for $i in 0..($n) { $expr }]};
    (for $i:ident in 0..=$n:literal { $expr:expr }) => { arr![for $i in 0..($n+1) { $expr }]};
    (for $i:ident in 0..=($n:expr) { $expr:expr }) => { arr![for $i in 0..($n+1) { $expr }]};
    (for $i:ident in 0..($n:expr) { $expr:expr }) => {
        {
            //create a MaybeUninit containint the array
            let mut arr = ::std::mem::MaybeUninit::<[_;$n]>::uninit();

            //loop over the array and assign each entry according to the index
            for $i in 0..$n {

                //compute the value here because we don't want the unsafe block to transfer
                let val = $expr;

                //we use write() here because we don't want to drop the previous value
                #[allow(unused_unsafe)]
                unsafe { ::std::ptr::write(&mut (*arr.as_mut_ptr())[$i], val); }

            }

            #[allow(unused_unsafe)]
            unsafe { arr.assume_init() }
        }
    }
}

///a helper macro for looping over generic tuples
macro_rules! impl_tuple {

    //the start of the loop
    ($callback:ident) => {impl_tuple!({A:a B:b C:c D:d E:e F:f G:g H:h I:i J:j K:k} L:l $callback);};
    ($callback:ident @with_last) => {
        impl_tuple!({A:a B:b C:c D:d E:e F:f G:g H:h I:i K:k J:j} L:l $callback @with_last);
    };

    //the end of the loop
    ({} $callback:ident) => {};
    // ({} $T0:ident:$t0:ident $callback:ident ) => {};
    ({} $callback:ident @$($options:tt)*) => {};

    ({$($A:ident:$a:ident)*} $T0:ident:$t0:ident $callback:ident) => {
        $callback!($($A:$a)* $T0:$t0);
        impl_tuple!({} $($A:$a)* $callback);
    };

    ({$($A:ident:$a:ident)*} $T0:ident:$t0:ident $callback:ident @with_last) => {
        $callback!({$($A:$a)*} $T0:$t0);
        impl_tuple!({} $($A:$a)* $callback @with_last);
    };

    //find the last generic in order to remove it from the list
    ({$($A:ident:$a:ident)*} $T0:ident:$t0:ident $T1:ident:$t1:ident $($rest:tt)*) => {
        impl_tuple!({$($A:$a)* $T0:$t0} $T1:$t1 $($rest)*);
    };
}


macro_rules! glenum {

    () => {};

    (
        $(#[$attr0:meta])*
        $vis:vis enum $name:ident {
            $($(#[$attr:meta])* $gl:ident),*
        }
        $($tt:tt)*
    ) => {
        glenum!{
            $(#[$attr0])*
            #[allow(non_camel_case_types)]
            $vis enum $name {
                $($(#[$attr])* [$gl $gl stringify!($gl)]),*
            } $($tt)*
        }
    };

    (
        $(#[$attr0:meta])*
        $vis:vis enum $name:ident {
            $($(#[$attr:meta])* [$item:ident $gl:ident]),*
        }
        $($tt:tt)*
    ) => {
        glenum!{
            $(#[$attr0])*
            $vis enum $name {
                $($(#[$attr])* [$item $gl stringify!($item)]),*
            } $($tt)*
        }
    };

    (
        $(#[$attr0:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$attr:meta])*
                [$item:ident $(($GL:ty; $pat:ident))?  $gl:ident $pretty:expr]
            ),*
        }
        $($tt:tt)*
    ) => {

        $(#[$attr0])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        #[repr(u32)]
        $vis enum $name {
            $(
                $(#[$attr])*
                $item $(($GL))? = ::gl::$gl as u32
            ),*
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match self {
                    $($name::$item $(($pat))? => write!(f, $pretty)),*
                }
            }
        }

        impl From<$name> for gl::types::GLenum {
            fn from(e: $name) -> ::gl::types::GLenum {
                match e {
                    $($name::$item $(($pat))? => ::gl::$gl),*
                }
            }
        }

        impl ::std::convert::TryFrom<::gl::types::GLenum> for $name {
            type Error = GLError;
            fn try_from(e: ::gl::types::GLenum) -> Result<$name, GLError>{
                match e {
                    $(::gl::$gl => Ok($name::$item $((supported::<$GL>().unwrap()))? ),)*
                    _ => Err(::GLError::InvalidEnum(e, stringify!($name).to_string()))
                }
            }
        }

        impl $crate::GLEnum for $name {}

        glenum!($($tt)*);

    };

    ($item:item $($rest:tt)*) => { $item }

}
