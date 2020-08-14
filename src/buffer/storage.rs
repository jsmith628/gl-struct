use super::*;

pub unsafe trait BufferStorage {
    type MapRead: Bit;
    type MapWrite: Bit;
    type DynamicStorage: Bit;
    type MapPersistent: Bit;
}

#[marker] pub unsafe trait DowngradesTo<B:BufferStorage>: BufferStorage {}

unsafe impl<A:NonPersistent, B:NonPersistent> DowngradesTo<B> for A where
    <A as BufferStorage>::MapRead:        BitMasks<<B as BufferStorage>::MapRead>,
    <A as BufferStorage>::MapWrite:       BitMasks<<B as BufferStorage>::MapWrite>,
    <A as BufferStorage>::DynamicStorage: BitMasks<<B as BufferStorage>::DynamicStorage>
{}

unsafe impl<A:BufferStorage> DowngradesTo<ReadOnly> for A {}
unsafe impl<A:Dynamic> DowngradesTo<DynWrite> for A {}
unsafe impl DowngradesTo<PersistMapRead> for PersistMapReadWrite {}
unsafe impl DowngradesTo<PersistMapWrite> for PersistMapReadWrite {}
unsafe impl DowngradesTo<PersistMapRead> for PersistReadDynWrite {}
unsafe impl DowngradesTo<PersistMapRead> for PersistReadWrite {}
unsafe impl DowngradesTo<PersistMapWrite> for PersistReadWrite {}
unsafe impl DowngradesTo<PersistWrite> for PersistReadWrite {}

pub trait ReadMappable = BufferStorage<MapRead=High>;
pub trait WriteMappable = BufferStorage<MapWrite=High>;
pub trait ReadWriteMappable = ReadMappable + WriteMappable;
pub trait Dynamic = BufferStorage<DynamicStorage=High>;
pub trait Persistent = BufferStorage<MapPersistent=High>;
pub trait NonPersistent = BufferStorage<MapPersistent=Low>;

macro_rules! access {
    ($( $(#[$attr:meta])* $ty:ident = [$($flag:ident = $bit:ident),*];)*) => {
        $(
            $(#[$attr])*
            pub struct $ty(!);

            unsafe impl BufferStorage for $ty { $(type $flag = $bit;)* }
        )*
    }
}

access! {
    ReadOnly            = [MapRead=Low,  MapWrite=Low,  DynamicStorage=Low,  MapPersistent=Low ];
    MapRead             = [MapRead=High, MapWrite=Low,  DynamicStorage=Low,  MapPersistent=Low ];
    MapWrite            = [MapRead=Low,  MapWrite=High, DynamicStorage=Low,  MapPersistent=Low ];
    MapReadWrite        = [MapRead=High, MapWrite=High, DynamicStorage=Low,  MapPersistent=Low ];

    DynWrite            = [MapRead=Low,  MapWrite=Low,  DynamicStorage=High, MapPersistent=Low ];
    MapReadDynWrite     = [MapRead=High, MapWrite=Low,  DynamicStorage=High, MapPersistent=Low ];
    Write               = [MapRead=Low,  MapWrite=High, DynamicStorage=High, MapPersistent=Low ];
    ReadWrite           = [MapRead=High, MapWrite=High, DynamicStorage=High, MapPersistent=Low ];

//  PersistMap          = [MapRead=Low,  MapWrite=Low,  DynamicStorage=Low,  MapPersistent=High];
    PersistMapRead      = [MapRead=High, MapWrite=Low,  DynamicStorage=Low,  MapPersistent=High];
    PersistMapWrite     = [MapRead=Low,  MapWrite=High, DynamicStorage=Low,  MapPersistent=High];
    PersistMapReadWrite = [MapRead=High, MapWrite=High, DynamicStorage=Low,  MapPersistent=High];

//  PersistDynWrite     = [MapRead=Low,  MapWrite=Low,  DynamicStorage=High, MapPersistent=High];
    PersistReadDynWrite = [MapRead=High, MapWrite=Low,  DynamicStorage=High, MapPersistent=High];
    PersistWrite        = [MapRead=Low,  MapWrite=High, DynamicStorage=High, MapPersistent=High];
    PersistReadWrite    = [MapRead=High, MapWrite=High, DynamicStorage=High, MapPersistent=High];
}
