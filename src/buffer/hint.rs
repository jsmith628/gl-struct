use super::*;

pub type DataHint = Option<BufferUsage>;
pub type StorageHint = Option<StorageFlags>;
pub type CreationHint = Option<BufferCreationFlags>;

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

bitflags! {

    ///Access flags for [glBufferStorage](gl::BufferStorage())
    #[derive(Default)]
    pub struct StorageFlags: GLbitfield {
        const MAP_READ_BIT = gl::MAP_READ_BIT;
        const MAP_WRITE_BIT = gl::MAP_WRITE_BIT;
        const MAP_PERSISTENT_BIT = gl::MAP_PERSISTENT_BIT;
        const MAP_COHERENT_BIT = gl::MAP_COHERENT_BIT;
        const DYNAMIC_STORAGE_BIT = gl::DYNAMIC_STORAGE_BIT;
        const CLIENT_STORAGE_BIT = gl::CLIENT_STORAGE_BIT;
    }

}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BufferCreationFlags(pub BufferUsage, pub StorageFlags);

impl BufferCreationFlags {
    #[inline] pub fn client_storage() -> Self { Self(Default::default(), StorageFlags::CLIENT_STORAGE_BIT) }
}

impl From<BufferUsage> for BufferCreationFlags {
    #[inline] fn from(usage:BufferUsage) -> Self { Self(usage,Default::default()) }
}

impl Default for BufferUsage { #[inline] fn default() -> Self {BufferUsage::StaticDraw} }
impl Default for BufferCreationFlags { #[inline] fn default() -> Self {Self(Default::default(),Default::default())} }
