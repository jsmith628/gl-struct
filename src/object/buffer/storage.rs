use super::*;

pub unsafe trait BufferStorage {
    type MapRead: Bit;
    type MapWrite: Bit;
    type DynamicStorage: Bit;
    type MapPersistent: Bit;
}

#[marker] pub unsafe trait Initialized: BufferStorage {}

pub trait ReadMappable = Initialized + BufferStorage<MapRead=High>;
pub trait WriteMappable = Initialized + BufferStorage<MapWrite=High>;
pub trait ReadWriteMappable = Initialized + ReadMappable + WriteMappable;
pub trait Dynamic = Initialized + BufferStorage<DynamicStorage=High>;
pub trait Persistent = Initialized + BufferStorage<MapPersistent=High>;
pub trait NonPersistent = Initialized + BufferStorage<MapPersistent=Low>;

macro_rules! access {
    ($( $(#[$attr:meta])* $ty:ident = [$($flag:ident = $bit:ident),*];)*) => {
        $(
            $(#[$attr])*
            #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
            pub struct $ty;

            unsafe impl Initialized for $ty {}
            unsafe impl BufferStorage for $ty { $(type $flag = $bit;)* }
        )*
    }
}

unsafe impl BufferStorage for ! {
    type MapRead=Low;
    type MapWrite=Low;
    type DynamicStorage=Low;
    type MapPersistent=Low;
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
