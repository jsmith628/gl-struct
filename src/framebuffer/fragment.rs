use super::*;

pub trait Fragment<'a>: Sized + 'a {}

impl<'a> Fragment<'a> for ! {}

pub struct Layered<T:TextureType, F>(T,F);
pub struct Multisampled<MS:MultisampleFormat, F>(MS,F);
