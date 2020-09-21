use super::*;

impl<P:Pixel> PixelSrc for Vec<P> {
    type Pixels = [P];
    type GL = ();
    fn pixels(&self) -> Pixels<[P],()> { (&**self).pixels() }
}

impl<P:Pixel> PixelDst for Vec<P> {
    fn pixels_mut(&mut self) -> PixelsMut<[P],()> { (&mut **self).pixels_mut() }
}

// impl<P:Pixel> FromPixels for Vec<P> {
//
//     type Hint = Option<usize>;
//
//     unsafe fn from_pixels<G:FnOnce(PixelsMut<[P],()>)>(
//         _: &Self::GL, hint: Option<usize>, size: usize, get: G
//     ) -> Self {
//         let mut vec = Vec::with_capacity(size.max(hint.unwrap_or(0)));
//         get(vec.pixels_mut());
//         vec.set_len(size);
//         vec
//     }
//
// }
