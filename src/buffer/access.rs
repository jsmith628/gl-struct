use super::*;

///
///Trait-level control over buffer creation and mapping access flags
///
///This particular system exists in lieu of a runtime solution in order to provide proper
///restriction of (Buffer)[super::Buffer] features at compile-time. This gives a way to make sure that
///all available functions for a given buffer satisfy the OpenGL API restrictions on use.
///
pub trait BufferAccess {

    type Read: Boolean;
    type Write: Boolean;
    type Persistent: Boolean;

}

///Any [BufferAccess] allowing readable mappings of Buffer contents
pub trait ReadAccess: BufferAccess<Read=True> {}
impl<A:BufferAccess<Read=True>> ReadAccess for A {}

///Any [BufferAccess] allowing client-side writes of Buffer contents
pub trait WriteAccess: BufferAccess<Write=True> {}
impl<A:BufferAccess<Write=True>> WriteAccess for A {}

///Any [BufferAccess] allowing persistent mapping
pub trait PersistentAccess: BufferAccess<Persistent=True> {}
impl<A:BufferAccess<Persistent=True>> PersistentAccess for A {}

///Any [BufferAccess] that doesn't persistently map buffers
pub trait NonPersistentAccess: BufferAccess<Persistent=False> {}
impl<A:BufferAccess<Persistent=False>> NonPersistentAccess for A {}

///A [BufferAccess] allowing no client-side access
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct CopyOnly;
impl BufferAccess for CopyOnly { type Read=False; type Write=False; type Persistent=False; }

///A [BufferAccess] allowing readonly client-side access
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Read;
impl BufferAccess for Read { type Read = True; type Write = False; type Persistent = False; }

///A [BufferAccess] allowing readonly client-side access and persistent mapping
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct PersistentRead;
impl BufferAccess for PersistentRead { type Read=True; type Write=False; type Persistent=True; }

///A [BufferAccess] allowing writeonly client-side access
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Write;
impl BufferAccess for Write { type Read=False; type Write=True; type Persistent=False; }

///A [BufferAccess] allowing both client-side reads and writes
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct ReadWrite;
impl BufferAccess for ReadWrite { type Read=True; type Write=True; type Persistent=False; }

///A [BufferAccess] allowing persistent mapping and both client-side reads and writes
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct PersistentReadWrite;
impl BufferAccess for PersistentReadWrite { type Read=True; type Write=True; type Persistent=True; }
