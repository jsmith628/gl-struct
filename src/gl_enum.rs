use super::*;

macro_rules! glenum {

    ({$($kw:tt)*} enum $name:ident {$($(#[$attr:meta])* $item:ident),*} $($tt:tt)*) => {
        glenum!({#[allow(non_camel_case_types)] $($kw)*} enum $name {$($(#[$attr])* [$item $item stringify!($item)]),*} $($tt)*);
    };

    ({$($kw:tt)*} enum $name:ident {$($(#[$attr:meta])* [$item:ident $gl:ident $pretty:expr] ),*} $($tt:tt)*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        $($kw)* enum $name {
            $(
                $(#[$attr])*
                $item = ::gl::$gl as isize
            ),*
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match self {
                    $($name::$item => write!(f, $pretty)),*
                }
            }
        }

        impl From<$name> for gl::types::GLenum {
            fn from(e: $name) -> ::gl::types::GLenum {
                match e {
                    $($name::$item => ::gl::$gl),*
                }
            }
        }

        impl ::std::convert::TryFrom<::gl::types::GLenum> for $name {
            type Error = GLError;
            fn try_from(e: ::gl::types::GLenum) -> Result<$name, GLError>{
                match e {
                    $(::gl::$gl => Ok($name::$item),)*
                    _ => Err(::GLError::InvalidEnum(e, stringify!($name).to_string()))
                }
            }
        }

        impl $crate::GLEnum for $name {}

        glenum!($($tt)*);

    };

    ({$($kws:tt)*} #[$attr:meta] $($tt:tt)*) => { glenum!({$($kws)* #[$attr]} $($tt)*); };
    ({$($kws:tt)*} $kw:ident($($path:tt)*) $($tt:tt)*) => { glenum!({$($kws)* $kw($($path)*)} $($tt)*); };
    ({$($kws:tt)*} $kw:ident $($tt:tt)*) => { glenum!({$($kws)* $kw} $($tt)*); };

    () => {};
    (# $($tt:tt)*) => {glenum!({} # $($tt)*);};
    ($kw:ident $($tt:tt)*) => {glenum!({} $kw $($tt)*);};

}

pub trait GLEnum: Sized + Copy + Eq + Hash + Debug + Display + Into<GLenum> + TryFrom<GLenum, Error=GLError> {}
