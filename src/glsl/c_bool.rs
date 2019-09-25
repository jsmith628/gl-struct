use super::*;

#[repr(align(4))]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default)]
pub struct c_bool(GLuint);

macro_rules! impl_c_bool {
    ($($Trait:ident.$fun:ident $op:tt),*) => {$(
        impl $Trait<Self> for c_bool { type Output = Self; #[inline] fn $fun(self, r:Self) -> Self {c_bool(self.0 $op r.0)} }
        impl $Trait<bool> for c_bool { type Output = Self; #[inline] fn $fun(self, r:bool) -> Self {c_bool(self.0 $op r as u32)} }
        impl $Trait<c_bool> for bool { type Output = c_bool; #[inline] fn $fun(self, r:c_bool) -> c_bool {c_bool(self as u32 $op r.0)} }
    )*}
}

macro_rules! impl_c_bool_assign {
    ($($Trait:ident.$fun:ident $op:tt),*) => {$(
        impl $Trait<Self> for c_bool { #[inline] fn $fun(&mut self, r:Self) {self.0 $op r.0;} }
        impl $Trait<bool> for c_bool { #[inline] fn $fun(&mut self, r:bool) {self.0 $op r as u32;} }
        impl $Trait<c_bool> for bool { #[inline] fn $fun(&mut self, r:c_bool) {*self $op r.0>0;} }
    )*}
}

impl_c_bool!(BitAnd.bitand &, BitOr.bitor |, BitXor.bitxor &);
impl_c_bool_assign!(BitAndAssign.bitand_assign &=, BitOrAssign.bitor_assign |=, BitXorAssign.bitxor_assign &=);

impl From<bool> for c_bool { #[inline] fn from(b: bool) -> Self {c_bool(b as GLuint)} }
impl From<c_bool> for bool { #[inline] fn from(b: c_bool) -> Self {b.0>0} }
impl From<GLuint> for c_bool { #[inline] fn from(b: GLuint) -> Self {c_bool(b)} }
impl From<c_bool> for GLuint { #[inline] fn from(b: c_bool) -> Self {b.0} }
impl From<GLboolean> for c_bool { #[inline] fn from(b: GLboolean) -> Self {c_bool(b as GLuint)} }
impl From<c_bool> for GLboolean { #[inline] fn from(b: c_bool) -> Self {b.0 as GLboolean} }
