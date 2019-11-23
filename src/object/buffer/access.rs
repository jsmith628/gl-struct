use super::*;

///
///Trait-level control over buffer creation and mapping access flags
///
///This particular system exists in lieu of a runtime solution in order to provide proper
///restriction of (Buffer)[super::Buffer] features at compile-time. This gives a way to make sure that
///all available functions for a given buffer satisfy the OpenGL API restrictions on use.
///
pub trait BufferAccess {

    type Read: Bit;
    type Write: Bit;
    type Persistent: Bit;

}

///Any [BufferAccess] allowing readable mappings of Buffer contents
pub trait ReadAccess = BufferAccess<Read=High>;

///Any [BufferAccess] allowing client-side writes of Buffer contents
pub trait WriteAccess = BufferAccess<Write=High>;

///Any [BufferAccess] allowing persistent mapping
pub trait PersistentAccess = BufferAccess<Persistent=High>;

///Any [BufferAccess] that doesn't persistently map buffers
pub trait NonPersistentAccess = BufferAccess<Persistent=Low>;

///A [BufferAccess] allowing no client-side access
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct CopyOnly;
impl BufferAccess for CopyOnly { type Read=Low; type Write=Low; type Persistent=Low; }

///A [BufferAccess] allowing readonly client-side access
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Read;
impl BufferAccess for Read { type Read = High; type Write = Low; type Persistent = Low; }

///A [BufferAccess] allowing readonly client-side access and persistent mapping
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct PersistentRead;
impl BufferAccess for PersistentRead { type Read=High; type Write=Low; type Persistent=High; }

///A [BufferAccess] allowing writeonly client-side access
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Write;
impl BufferAccess for Write { type Read=Low; type Write=High; type Persistent=Low; }

///A [BufferAccess] allowing writeonly client-side access and persistent mapping
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct PersistentWrite;
impl BufferAccess for PersistentWrite { type Read=Low; type Write=High; type Persistent=High; }

///A [BufferAccess] allowing both client-side reads and writes
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct ReadWrite;
impl BufferAccess for ReadWrite { type Read=High; type Write=High; type Persistent=Low; }

///A [BufferAccess] allowing persistent mapping and both client-side reads and writes
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct PersistentReadWrite;
impl BufferAccess for PersistentReadWrite { type Read=High; type Write=High; type Persistent=High; }
