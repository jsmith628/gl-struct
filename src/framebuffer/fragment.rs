use super::*;

pub trait Fragment<'a>: Sized + 'a {}

impl<'a> Fragment<'a> for ! {}
