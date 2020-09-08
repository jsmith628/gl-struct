use super::*;

pub unsafe trait BufferStorage {
    type MapRead: Bit;
    type MapWrite: Bit;
    type DynamicStorage: Bit;
    type MapPersistent: Bit;
}

#[marker] pub unsafe trait DowngradesTo<B:BufferStorage>: BufferStorage {}

//
//Downgrade to a specific variety
//

//anything can be downgraded to ReadOnly
unsafe impl<A:BufferStorage> DowngradesTo<ReadOnly> for A {}

//we don't have to worry about the persistent bit since we can't map DynWrite buffers
unsafe impl<A:Dynamic> DowngradesTo<DynWrite> for A {}

unsafe impl<A:NonPersistent+ReadMappable>              DowngradesTo<MapRead>         for A {}
unsafe impl<A:NonPersistent+WriteMappable>             DowngradesTo<MapWrite>        for A {}
unsafe impl<A:NonPersistent+ReadWriteMappable>         DowngradesTo<MapReadWrite>    for A {}
unsafe impl<A:NonPersistent+Dynamic+ReadMappable>      DowngradesTo<MapReadDynWrite> for A {}
unsafe impl<A:NonPersistent+Dynamic+WriteMappable>     DowngradesTo<Write>           for A {}
unsafe impl<A:NonPersistent+Dynamic+ReadWriteMappable> DowngradesTo<ReadWrite>       for A {}

//we have to keep the persistent and non-persistent types separate
unsafe impl<A:Persistent+ReadMappable>              DowngradesTo<PersistMapRead>      for A {}
unsafe impl<A:Persistent+WriteMappable>             DowngradesTo<PersistMapWrite>     for A {}
unsafe impl<A:Persistent+ReadWriteMappable>         DowngradesTo<PersistMapReadWrite> for A {}
unsafe impl<A:Persistent+Dynamic+ReadMappable>      DowngradesTo<PersistReadDynWrite> for A {}
unsafe impl<A:Persistent+Dynamic+WriteMappable>     DowngradesTo<PersistWrite> for A {}
unsafe impl<A:Persistent+Dynamic+ReadWriteMappable> DowngradesTo<PersistReadWrite> for A {}

//
//Downgrade from a specific type
//

unsafe impl<A:NonPersistent+NonDynamic+NonReadMappable+NonWriteMappable> DowngradesTo<A> for ReadOnly {}

unsafe impl<A:NonPersistent>                             DowngradesTo<A> for ReadWrite {}
unsafe impl<A:NonPersistent+NonReadMappable>             DowngradesTo<A> for Write {}
unsafe impl<A:NonPersistent+NonWriteMappable>            DowngradesTo<A> for MapReadDynWrite {}
unsafe impl<A:NonPersistent+NonDynamic>                  DowngradesTo<A> for MapReadWrite {}
unsafe impl<A:NonPersistent+NonDynamic+NonReadMappable>  DowngradesTo<A> for MapWrite {}
unsafe impl<A:NonPersistent+NonDynamic+NonWriteMappable> DowngradesTo<A> for MapRead {}

unsafe impl<A:Persistent>                             DowngradesTo<A> for PersistReadWrite {}
unsafe impl<A:Persistent+NonReadMappable>             DowngradesTo<A> for PersistWrite {}
unsafe impl<A:Persistent+NonWriteMappable>            DowngradesTo<A> for PersistReadDynWrite {}
unsafe impl<A:Persistent+NonDynamic>                  DowngradesTo<A> for PersistMapReadWrite {}
unsafe impl<A:Persistent+NonDynamic+NonReadMappable>  DowngradesTo<A> for PersistMapWrite {}
unsafe impl<A:Persistent+NonDynamic+NonWriteMappable> DowngradesTo<A> for PersistMapRead {}

pub trait ReadMappable = BufferStorage<MapRead=High>;
pub trait WriteMappable = BufferStorage<MapWrite=High>;
pub trait ReadWriteMappable = ReadMappable + WriteMappable;
pub trait Dynamic = BufferStorage<DynamicStorage=High>;
pub trait Persistent = BufferStorage<MapPersistent=High>;

pub trait NonReadMappable = BufferStorage<MapRead=Low>;
pub trait NonWriteMappable = BufferStorage<MapWrite=Low>;
pub trait NonDynamic = BufferStorage<DynamicStorage=Low>;
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
