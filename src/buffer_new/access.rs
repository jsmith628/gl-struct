use super::*;

use trait_arith::{Boolean, True, False};
use std::ops::{BitOr, BitOrAssign};
use std::convert::TryFrom;

glenum!{
    ///Possible hints for buffer data initialization with [glBufferData()](gl::BufferData())
    pub enum BufferUsage {
        [StreamDraw STREAM_DRAW "Stream:Draw"],
        [StreamRead STREAM_READ "Stream:Read"],
        [StreamCopy STREAM_COPY "Stream:Copy"],
        [StaticDraw STATIC_DRAW "Static:Draw"],
        [StaticRead STATIC_READ "Static:Read"],
        [StaticCopy STATIC_COPY "Static:Copy"],
        [DynamicDraw DYNAMIC_DRAW "Dynamic:Draw"],
        [DynamicRead DYNAMIC_READ "Dynamic:Read"],
        [DynamicCopy DYNAMIC_COPY "Dynamic:Copy"]
    }

}

impl Default for BufferUsage { #[inline] fn default() -> Self {BufferUsage::DynamicDraw} }

bitflags! {

    ///Access flags for [glBufferStorage](gl::BufferStorage())
    #[derive(Default)]
    pub struct StorageFlags: GLbitfield {
        const DYNAMIC_STORAGE_BIT = gl::DYNAMIC_STORAGE_BIT;
        const MAP_READ_BIT = gl::MAP_READ_BIT;
        const MAP_WRITE_BIT = gl::MAP_WRITE_BIT;
        const MAP_PERSISTENT_BIT = gl::MAP_PERSISTENT_BIT;
        const MAP_COHERENT_BIT = gl::MAP_COHERENT_BIT;
        const CLIENT_STORAGE_BIT = gl::CLIENT_STORAGE_BIT;
    }
}

impl StorageFlags {
    #[inline] pub fn from_access<A:BufferAccess>() -> Self {
        let mut flags = A::extra_storage_flags();
        if A::Read::VALUE {
            flags |=
                StorageFlags::MAP_READ_BIT |
                StorageFlags::MAP_PERSISTENT_BIT
        };
        if A::Write::VALUE {
            flags |=
                StorageFlags::MAP_WRITE_BIT |
                StorageFlags::DYNAMIC_STORAGE_BIT
        };
        flags
    }
}

bitflags! {
    ///Access flags for [glBufferStorage](gl::BufferStorage())
    #[derive(Default)]
    pub struct MapRangeFlags: GLbitfield {
        const MAP_READ_BIT = gl::MAP_READ_BIT;
        const MAP_WRITE_BIT = gl::MAP_WRITE_BIT;
        const MAP_PERSISTENT_BIT = gl::MAP_PERSISTENT_BIT;
        const MAP_COHERENT_BIT = gl::MAP_COHERENT_BIT;
        const MAP_INVALIDATE_RANGE_BIT = gl::MAP_INVALIDATE_RANGE_BIT;
        const MAP_INVALIDATE_BUFFER_BIT = gl::MAP_INVALIDATE_BUFFER_BIT;
        const MAP_FLUSH_EXPLICIT_BIT = gl::MAP_FLUSH_EXPLICIT_BIT;
        const MAP_UNSYNCHRONIZED_BIT = gl::MAP_UNSYNCHRONIZED_BIT;
    }
}

impl MapRangeFlags {
    #[inline] pub fn from_access<A:BufferAccess>() -> Self {
        let mut flags = A::extra_map_range_flags();
        if A::Read::VALUE { flags |= MapRangeFlags::MAP_READ_BIT };
        if A::Write::VALUE { flags |= MapRangeFlags::MAP_WRITE_BIT };
        flags
    }
}

//the bits that are valid for both glBufferStorage and glMapBufferRange
const MAP_STORAGE_BITS_MASK: GLbitfield = gl::MAP_READ_BIT | gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT;

impl TryFrom<StorageFlags> for MapRangeFlags {
    type Error = GLError;
    #[inline]
    fn try_from(bits:StorageFlags) -> Result<Self,Self::Error> {
        match MapRangeFlags::from_bits(bits.bits()) {
            Some(b) => Ok(b),
            None => Err(GLError::InvalidBits(bits.bits()&!MAP_STORAGE_BITS_MASK, "MapRangeFlags".to_string()))
        }
    }
}

impl TryFrom<MapRangeFlags> for StorageFlags {
    type Error = GLError;
    #[inline]
    fn try_from(bits:MapRangeFlags) -> Result<Self,Self::Error> {
        match StorageFlags::from_bits(bits.bits()) {
            Some(b) => Ok(b),
            None => Err(GLError::InvalidBits(bits.bits()&!MAP_STORAGE_BITS_MASK, "MapRangeFlags".to_string()))
        }
    }
}

glenum! {
    ///Access flags for [glMapBuffer](gl::MapBuffer())
    pub enum MapAccess {
        [ReadOnly READ_ONLY "Read-only"],
        [WriteOnly WRITE_ONLY "Write-only"],
        [ReadWrite READ_WRITE "Read-Write"]
    }
}

impl MapAccess {
    #[inline] pub fn from_access<A:BufferAccess>() -> Self {
        let mut flags = A::extra_map_access();
        if A::Read::VALUE { flags |= MapAccess::ReadOnly };
        if A::Write::VALUE { flags |= MapAccess::WriteOnly };
        flags
    }
}

impl BitOrAssign for MapAccess { #[inline] fn bitor_assign(&mut self, rhs:Self) {*self=*self|rhs;} }
impl BitOr for MapAccess {
    type Output = Self;
    #[inline] fn bitor(self, rhs:Self) -> Self {
        match self {
            MapAccess::ReadOnly => match rhs {
                MapAccess::ReadOnly => MapAccess::ReadOnly,
                _ => MapAccess::ReadWrite
            },
            MapAccess::WriteOnly => match rhs {
                MapAccess::WriteOnly => MapAccess::WriteOnly,
                _ => MapAccess::ReadWrite
            },
            MapAccess::ReadWrite => MapAccess::ReadWrite
        }
    }
}

impl From<MapAccess> for MapRangeFlags {
    #[inline] fn from(bits:MapAccess) -> Self {
        MapRangeFlags::from_bits(bits as GLbitfield).unwrap()
    }
}

impl From<MapAccess> for StorageFlags {
    #[inline] fn from(bits:MapAccess) -> Self {
        StorageFlags::from_bits(bits as GLbitfield).unwrap()
    }
}

///
///Trait-level control over buffer creation and mapping access flags
///
///This particular system exists in lieu of a runtime solution in order to provide proper
///restriction of (Buffer)[super::Buf] features at compile-time. This gives a way to make sure that
///all available functions for a given buffer satisfy the OpenGL API restrictions on use. In particular,
///it guarrantees that a Buffer initialized with [glBufferStorage](super::Buf::storage)
///using only [MAP_READ_BIT](StorageFlags::MAP_READ_BIT) will never attempt to mutate its
///contents client-side.
///
///To construct the flags and enums themselves, the prefered method is to invoke [StorageFlags::from_access()],
///[MapAccess::from_access()], [MapRangeFlags::from_access()], etc, as this trait does not provide methods
///to construct the values in full. Instead, the full flag values consist of a set of Read, Write, and
///"extra" flags, each added in the above methods as dictated by the corresponding elements of this
///trait.
///
pub trait BufferAccess {

    type Read: Boolean;
    type Write: Boolean;

    #[inline] fn extra_storage_flags() -> StorageFlags {Default::default()}
    #[inline] fn extra_map_access() -> MapAccess {MapAccess::ReadOnly}
    #[inline] fn extra_map_range_flags() -> MapRangeFlags {Default::default()}
    #[inline] fn default_usage() -> BufferUsage {Default::default()}
}

///Any [BufferAccess] allowing readable mappings of Buffer contents
pub trait ReadAccess: BufferAccess<Read=True> {}
impl<A:BufferAccess<Read=True>> ReadAccess for A {}

///Any [BufferAccess] allowing client-side writes of Buffer contents
pub trait WriteAccess: BufferAccess<Write=True> {}
impl<A:BufferAccess<Write=True>> WriteAccess for A {}

///A [BufferAccess] allowing no client-side operations
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct CopyOnly;
impl BufferAccess for CopyOnly { type Read = False; type Write = False; }

///A [BufferAccess] allowing only client-side reads
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Read;
impl BufferAccess for Read { type Read = True; type Write = False; }

///A [BufferAccess] allowing only client-side writes
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Write;
impl BufferAccess for Write { type Read = False; type Write = True; }

///A [BufferAccess] allowing both client-side reads and writes
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct ReadWrite;
impl BufferAccess for ReadWrite { type Read = True; type Write = True; }

//TODO, fix this doc comment

///
///Any type that can be cloned within a [buffer](super::Buf) by simple byte-wise copies of its data.
///
pub unsafe trait GPUCopy {}
unsafe impl<T:Copy> GPUCopy for T {}
unsafe impl<T:Copy> GPUCopy for [T] {}

macro_rules! impl_tuple_gpucopy {
    ({$($T:ident:$t:ident)*} $Last:ident:$l:ident) => {
        unsafe impl<$($T),*, $Last: Copy> GPUCopy for ($($T,)* [$Last]) where ($($T),*):GPUCopy {}
    };
}
impl_tuple!(impl_tuple_gpucopy @with_last);
