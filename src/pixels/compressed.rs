use super::*;
use std::mem::*;
use crate::buffer::GPUCopy;

pub struct Cmpr<F:SpecificCompressed> {
    data: [F::Block]
}

impl<F:SpecificCompressed> Cmpr<F> {

    pub fn format(&self) -> GLenum { F::glenum() }
    pub fn block_width(&self) -> usize { F::block_width() }
    pub fn block_height(&self) -> usize { F::block_height() }
    pub fn block_depth(&self) -> usize { F::block_depth() }
    pub fn block_size(&self) -> usize { size_of::<F::Block>() }

    pub fn size(&self) -> usize { size_of_val(self) }
    pub fn blocks(&self) -> usize { self.data.len() }

    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn len(&self) -> usize {
        self.blocks() *
        self.block_width() as usize *
        self.block_height() as usize *
        self.block_depth() as usize
    }

}

impl<F:SpecificCompressed> GPUCopy for Cmpr<F> {}

impl<F:SpecificCompressed> ToOwned for Cmpr<F> {
    type Owned = Box<Self>;
    fn to_owned(&self) -> Box<Self> {
        let mut uninit = Box::<[MaybeUninit<F::Block>]>::new_uninit_slice(self.blocks());
        unsafe {
            self.data.as_ptr().copy_to(uninit.as_mut_ptr() as *mut _, self.blocks());
            transmute(uninit.assume_init())
        }
    }
}


//Red-Green Texture Compression
pub type RedRgtc1 = Cmpr<COMPRESSED_RED_RGTC1>;
pub type SignedRedRgtc1 = Cmpr<COMPRESSED_SIGNED_RED_RGTC1>;
pub type RgRgtc2 = Cmpr<COMPRESSED_RG_RGTC2>;
pub type SignedRGRgtc2 = Cmpr<COMPRESSED_SIGNED_RG_RGTC2>;

//BPTC
pub type RgbaBptcUnorm = Cmpr<COMPRESSED_RGBA_BPTC_UNORM>;
pub type SrgbAlphaBptcUnorm = Cmpr<COMPRESSED_SRGB_ALPHA_BPTC_UNORM>;
pub type RgbBptcSignedFloat = Cmpr<COMPRESSED_RGB_BPTC_SIGNED_FLOAT>;
pub type RgbBptcUnsignedFloat = Cmpr<COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT>;

//Ericsson Texture Compression
pub type Rgb8Etc2 = Cmpr<COMPRESSED_RGB8_ETC2>;
pub type Srgb8Etc2 = Cmpr<COMPRESSED_SRGB8_ETC2>;
pub type Rgb8PunchthroughAlpha1Etc2 = Cmpr<COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2>;
pub type Srgb8PunchthroughAlpha1Etc2 = Cmpr<COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2>;
pub type Rgba8Etc2Eac = Cmpr<COMPRESSED_RGBA8_ETC2_EAC>;
pub type Sgb8Alpha8Etc2Eac = Cmpr<COMPRESSED_SRGB8_ALPHA8_ETC2_EAC>;
pub type R11Eac = Cmpr<COMPRESSED_R11_EAC>;
pub type SignedR11Eac = Cmpr<COMPRESSED_SIGNED_R11_EAC>;
pub type RG11Eac = Cmpr<COMPRESSED_RG11_EAC>;
pub type SignedRG11Eac = Cmpr<COMPRESSED_SIGNED_RG11_EAC>;
