use super::*;

#[derive(Clone,Copy)]
pub struct ClientImage<B:?Sized> {
    dim: [usize;3],
    pixels: B
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ImageError {
    SizeOverflow([usize;3]),
    InvalidDimensions([usize;3], usize),
    NotBlockAligned([usize;3], [usize;3]),
    GLVersion(GLVersionError)
}

impl ::std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ImageError::SizeOverflow([x,y,z]) => write!(
                f, "Overflow in computing buffer size for a {}x{}x{} image", x, y, z
            ),
            ImageError::InvalidDimensions([x,y,z], buffer_size) => write!(
                f,
                "Invalid dimensions for buffer size. a {}x{}x{} image requires a
                 {} byte buffer but a {} byte was given instead",
                 x, y, z, x*y*z, buffer_size
            ),
            ImageError::NotBlockAligned([x,y,z], [b_x,b_y,b_z]) => write!(
                f, "Image dimensions {}x{}x{} not divisible by compressed block dimensions {}x{}x{}",
                x,y,z, b_x,b_y,b_z
            ),
            ImageError::GLVersion(e) => write!(f, "{}", e)
        }
    }
}


impl<B> ClientImage<B> {

    pub unsafe fn new_unchecked(dim: [usize;3], pixels: B) -> Self {
        ClientImage { dim, pixels }
    }

    pub fn dim(&self) -> [usize; 3] { self.dim }

    pub fn width(&self) -> usize { self.dim()[0] }
    pub fn height(&self) -> usize { self.dim()[1] }
    pub fn depth(&self) -> usize { self.dim()[2] }

}

//Use specialization to try to get the OpenGL version necessary to create a ClientImage
//
//This is kinda an ugly system, but it's sort of necessary for a complicated set of reasons:
//Basically, in order for the PixelSrc trait to be a safe, we need the PixelSrc trait to
//guarantee that a given GL version is supported when accessing its pixels, thus requiring a GLVersion
//object whenever calling pixels() or pixels_mut(). However, in order to do length/bounds checks
//we _also_ need to get the length of the backing buffer from the reference returned by those methods,
//but because of the aforementioned problem, we need a GLVersion object in order to do that.
//Additionally, this is made worse by the fact that we don't even ultimately _need_ a GL version
//to be loaded to check the length of a Buffer or Buffer slice.
//
//Now, one possible solution to this would be to simple add a len() or count() method to PixelSrc
//and be done with it. However, this opens up the possibility of the len/count method conflicting
//with the _actual_ length provided by the pixel reference.
//
//Another possible solution is instead to, in effect, just assume that GL version is loaded,
//since we ultimately don't need it anyway. However, this leads to yet another problem where
//_theoretically_ a particularly devious implementor could decide to actually _use_ the GLVersion
//object provided in pixels()/pixels_mut() to do something completely non-trivial, in which case,
//we actually _can't_ assume this...
//
//To fix this rare edge case, we simply add another variant to the ImageError for version errors
//and assume version support for all normal PixelSrc types and just do a version check for anything
//else
//
trait TryGetGL: PixelSrc { fn get_gl() -> Result<Self::GL,ImageError>; }

impl<B:PixelSrc<GL=()>> TryGetGL for B { fn get_gl() -> Result<Self::GL,ImageError> { Ok(()) } }

impl<P:Pixel,A:BufferStorage> TryGetGL for Buffer<[P],A> {
    fn get_gl() -> Result<Self::GL,ImageError> { Ok(unsafe {assume_supported()}) }
}
impl<'a,P:Pixel,A:BufferStorage> TryGetGL for Slice<'a,[P],A> {
    fn get_gl() -> Result<Self::GL,ImageError> { Ok(unsafe {assume_supported()}) }
}
impl<'a,P:Pixel,A:BufferStorage> TryGetGL for SliceMut<'a,[P],A> {
    fn get_gl() -> Result<Self::GL,ImageError> { Ok(unsafe {assume_supported()}) }
}
impl<F:SpecificCompressed,A:BufferStorage> TryGetGL for Buffer<CompressedPixels<F>,A>  {
    fn get_gl() -> Result<Self::GL,ImageError> { Ok(unsafe {assume_supported()}) }
}
impl<'a,F:SpecificCompressed,A:BufferStorage> TryGetGL for Slice<'a,CompressedPixels<F>,A> {
    fn get_gl() -> Result<Self::GL,ImageError> { Ok(unsafe {assume_supported()}) }
}
impl<'a,F:SpecificCompressed,A:BufferStorage> TryGetGL for SliceMut<'a,CompressedPixels<F>,A> {
    fn get_gl() -> Result<Self::GL,ImageError> { Ok(unsafe {assume_supported()}) }
}

impl<B:PixelSrc> TryGetGL for B {
    default fn get_gl() -> Result<Self::GL,ImageError> {
        crate::version::supported().map_err(|e| ImageError::GLVersion(e))
    }
}

//TODO: creation methods for compressed data
impl<P, B:PixelSrc<Pixels=[P]>> ClientImage<B> {

    pub fn try_new(dim: [usize;3], pixels: B) -> Result<Self,ImageError> {
        //compute the array size required to store that many pixels while making sure the value
        //does not overflow
        let count = dim[0].checked_mul(dim[1]).and_then(|m| m.checked_mul(dim[2]));

        //if we did not overflow
        if let Some(n) = count {

            //get a reference to the backing slice or GL buffer and make sure it has the exact
            //length required to store the pixels for this image
            let len = pixels.pixels(B::get_gl()?).len();
            if n==len {
                Ok( unsafe {Self::new_unchecked(dim, pixels)} )
            } else {
                Err(ImageError::InvalidDimensions(dim, len))
            }

        } else {
            Err(ImageError::SizeOverflow(dim))
        }
    }

    pub fn new(dim: [usize;3], pixels: B) -> Self {
        Self::try_new(dim, pixels).unwrap()
    }

}

impl<P,B:PixelSrc<Pixels=[P]>> UncompressedImage for ClientImage<B> { type Pixel = P; }
impl<F:SpecificCompressed,B:PixelSrc<Pixels=CompressedPixels<F>>> CompressedImage for ClientImage<B> {
    type Format = F;
}

impl<B:PixelSrc> ImageSrc for ClientImage<B> {
    type Pixels = B::Pixels;
    type GL = B::GL;
    fn image(&self, gl:&Self::GL) -> ImagePtr<Self::Pixels> { unimplemented!() }
}

impl<B:PixelDst> ImageDst for ClientImage<B> {
    fn image_mut(&mut self, gl:&Self::GL) -> ImagePtrMut<Self::Pixels> { unimplemented!() }
}

unsafe impl<B:FromPixels> OwnedImage for ClientImage<B> {
    type Hint = B::Hint;

    unsafe fn from_gl<G:FnOnce(PixelStore, PixelsMut<Self::Pixels>)>(
        gl:&B::GL, hint:B::Hint, dim: [usize;3], get:G
    ) -> Self {
        let settings = Default::default();
        ClientImage {
            dim, pixels: B::from_pixels(gl, hint, pixel_count(dim), |ptr| get(settings, ptr))
        }
    }
}
