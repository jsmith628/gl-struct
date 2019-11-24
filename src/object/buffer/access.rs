use super::*;

pub unsafe trait BufferAccess {
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

macro_rules! access {
    ($( $(#[$attr:meta])* $ty:ident = [$($flag:ident = $bit:ident),*];)*) => {
        $(
            $(#[$attr])*
            #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
            pub struct $ty;

            unsafe impl BufferAccess for $ty {
                $(type $flag = $bit;)*
            }
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
