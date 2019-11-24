use super::*;

pub trait BufferAccess {
    type MapRead: Bit;
    type MapWrite: Bit;
    type DynamicStorage: Bit;
    type MapPersistent: Bit;
}

pub trait DowngradesTo<A:BufferAccess> = BufferAccess where
    <Self as BufferAccess>::MapRead:        BitMasks<<A as BufferAccess>::MapRead>,
    <Self as BufferAccess>::MapWrite:       BitMasks<<A as BufferAccess>::MapWrite>,
    <Self as BufferAccess>::DynamicStorage: BitMasks<<A as BufferAccess>::DynamicStorage>,
    <Self as BufferAccess>::MapPersistent:  BitMasks<<A as BufferAccess>::MapPersistent>;

pub trait MapReadAccess = BufferAccess<MapRead=High>;
pub trait MapWriteAccess = BufferAccess<MapWrite=High>;
pub trait DynamicAccess = BufferAccess<DynamicStorage=High>;
pub trait PersistentAccess = BufferAccess<MapPersistent=High>;
pub trait NonPersistentAccess = BufferAccess<MapPersistent=Low>;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct ReadOnly;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct MapRead;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct Write;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct DynWrite;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct MapWrite;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct MapReadDynWrite;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct MapReadWrite;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct ReadWrite;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct PersistRead;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct PersistWrite;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct PersistReadDynWrite;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct PersistReadMapWrite;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)] pub struct PersistReadWrite;

impl BufferAccess for ReadOnly { type MapRead=Low; type DynamicStorage=Low; type MapWrite=Low; type MapPersistent=Low; }
impl BufferAccess for MapRead { type MapRead = High; type DynamicStorage=Low; type MapWrite=Low; type MapPersistent=Low; }

impl BufferAccess for Write { type MapRead=Low; type DynamicStorage=High; type MapWrite=High; type MapPersistent=Low; }
impl BufferAccess for DynWrite { type MapRead=Low; type DynamicStorage=High; type MapWrite=Low; type MapPersistent=Low; }
impl BufferAccess for MapWrite { type MapRead=Low; type DynamicStorage=Low; type MapWrite=High; type MapPersistent=Low; }

impl BufferAccess for MapReadDynWrite { type MapRead=High; type DynamicStorage=High; type MapWrite=Low; type MapPersistent=Low; }
impl BufferAccess for MapReadWrite { type MapRead=High; type DynamicStorage=Low; type MapWrite=High; type MapPersistent=Low; }
impl BufferAccess for ReadWrite { type MapRead=High; type DynamicStorage=High; type MapWrite=High; type MapPersistent=Low; }

impl BufferAccess for PersistRead { type MapRead=High; type DynamicStorage=Low; type MapWrite=Low; type MapPersistent=High; }
impl BufferAccess for PersistWrite { type MapRead=Low; type DynamicStorage=Low; type MapWrite=High; type MapPersistent=High; }
impl BufferAccess for PersistReadDynWrite { type MapRead=High; type DynamicStorage=High; type MapWrite=Low; type MapPersistent=High; }
impl BufferAccess for PersistReadMapWrite { type MapRead=High; type DynamicStorage=High; type MapWrite=Low; type MapPersistent=High; }
impl BufferAccess for PersistReadWrite { type MapRead=High; type DynamicStorage=High; type MapWrite=High; type MapPersistent=High; }
