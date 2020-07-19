use super::*;

pub trait GLEnum: Sized + Copy + Eq + Hash + Debug + Display + Into<GLenum> + TryFrom<GLenum, Error=GLError> {}
