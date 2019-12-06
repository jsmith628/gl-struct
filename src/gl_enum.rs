use super::*;

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
                    $(::gl::$gl => Ok($name::$item $((supported::<$GL>()?))? ),)*
                    _ => Err(::GLError::InvalidEnum(e, stringify!($name).to_string()))
                }
            }
        }

        impl $crate::GLEnum for $name {}

        glenum!($($tt)*);

    };

}

pub trait GLEnum: Sized + Copy + Eq + Hash + Debug + Display + Into<GLenum> + TryFrom<GLenum, Error=GLError> {}
