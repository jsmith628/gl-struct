use super::*;

macro_rules! gl_resource{

    (@fun gen=$gl:ident) => {
        #[inline]
        fn gen(_gl: &Self::GL) -> Self {
            unsafe {
                let mut obj = ::std::mem::uninitialized::<Self>();
                gl::$gl(1, &mut obj.0 as *mut gl::types::GLuint);
                obj
            }
        }

        #[inline]
        fn gen_resources(_gl: &Self::GL, count: gl::types::GLuint) -> Box<[Self]> {
            unsafe {
                let mut obj = Vec::<Self>::with_capacity(count as usize);
                obj.set_len(count as usize);
                gl::$gl(count as gl::types::GLsizei, &mut obj[0].0 as *mut gl::types::GLuint);
                obj.into_boxed_slice()
            }
        }
    };

    (@fun is=$gl:ident) => {
        #[inline] fn is(id: gl::types::GLuint) -> bool { unsafe { gl::$gl(id) != gl::FALSE } }
    };

    (@fun bind=$gl:ident) => {};

    (@fun gl=$GL:ident) => { type GL = $GL; };
    (@fun target=$Target:ident) => { type BindingTarget = $Target; };

    (@fun delete=$gl:ident) => {
        #[inline]
        fn delete(self) {
            unsafe { gl::$gl(1, &self.into_raw() as *const gl::types::GLuint); }
        }

        #[inline]
        fn delete_resources(resources: Box<[Self]>) {
            unsafe {
                //the transmutation makes sure that we don't double-free
                let ids = ::std::mem::transmute::<Box<[Self]>, Box<[gl::types::GLuint]>>(resources);
                gl::$gl(ids.len() as gl::types::GLsizei, &ids[0] as *const gl::types::GLuint);
            }
        }
    };

    (@bind $ty:ident {$($tt:tt)*} bind=$gl:ident $($rest:tt)*) => { gl_resource!(@bind $ty {$($tt)* bind=$gl} $($rest)*); };
    (@bind $ty:ident {$($tt:tt)*} target=$Target:ident $($rest:tt)*) => { gl_resource!(@bind $ty {target=$Target $($tt)*} $($rest)*); };
    (@bind $ty:ident {$($tt:tt)*} $param:ident=$gl:ident $($rest:tt)*) => { gl_resource!(@bind $ty {$($tt)*} $($rest)*); };
    (@bind $ty:ident {target=$Target:ident bind=$gl:ident}) => {
        unsafe impl Target for $Target {
            type Resource = $ty;
            #[inline] unsafe fn bind(self, id:GLuint) {gl::$gl(self as GLenum, id)}
        }
    };

    ({$($mod:tt)*} struct $name:ident {$($fun:ident=$gl:ident),*}) => {

        gl_resource!(@bind $name {} $($fun=$gl)*);

        #[repr(C)] $($mod)* struct $name(GLuint);

        unsafe impl $crate::Resource for $name {
            $(gl_resource!(@fun $fun=$gl);)*

            #[inline] fn id(&self) -> gl::types::GLuint { self.0 }

            #[inline]
            unsafe fn from_raw(id: gl::types::GLuint) -> Option<Self> {
                if Self::is(id) { Some($name(id)) } else {None}
            }

            #[inline]
            fn into_raw(self) -> gl::types::GLuint {
                let id = self.id();
                ::std::mem::forget(self);
                id
            }

        }

        impl Drop for $name {
            #[inline] fn drop(&mut self) { $crate::Resource::delete($name(self.0)); }
        }


    };

    ({$($mod:tt)*} #[$attr:meta] $($tt:tt)*) => {gl_resource!({$($mod)* #[$attr]} $($tt)*);};
    ({$($mod:tt)*} $kw:ident($($args:tt)*) $($tt:tt)*) => {gl_resource!({$($mod)* $kw($($args)*)} $($tt)*);};
    ({$($mod:tt)*} $kw:ident $($tt:tt)*) => {gl_resource!({$($mod)* $kw} $($tt)*);};

    ($kw:ident $($tt:tt)*) => {gl_resource!({} $kw $($tt)*);};
    (#[$attr:meta] $($tt:tt)*) => {gl_resource!({} #[$attr] $($tt)*);};
}


///
///An OpenGL resource object that follows the standard [glGen*](Resource::gen),
///[glIs*](Resource::is), and [glDelete*](Resource::delete) pattern
///
///# Unsafety
///
///It is up to the caller to implement [Drop] individually in order to call [delete](Self::delete)
///when leaving scope, and to guarrantee that the [GL](Resource::GL) object properly loads all necessary
///functions
///
pub unsafe trait Resource:Sized {

    ///The OpenGL version type that guarrantees that the functions required for initialization are loaded
    type GL: GLProvider;
    type BindingTarget: Target<Resource=Self>;

    ///
    ///The identification of the object used internally by OpenGL that is returned by the gen method.
    ///
    ///This identification number is unique to this object for as long as it exists, and thus,
    ///barring unsafe shenanigans, no other object will share it. In fact, the id is unique even
    ///outside its type and for all OpenGL resources of this particular form. Of course though, once
    ///the object has been deleted, the id can be reused for another object or even another type.
    ///
    ///Note that this is intended to be analogous to creating a raw pointer from a reference. As such,
    ///this method is actually _not_ unsafe, as it does not inherently make any unsafe memory operations
    ///or even leak data (much like pointer creation). Rather, any functions operating the returned a
    ///raw id should be marked unsafe themselves since they can make no guarrantees of memory safety.
    ///
    fn id(&self) -> GLuint;

    ///
    ///Consumes this object and leaks its id
    ///
    ///As such, it is up to the caller to delete the resource or remake the object with
    ///[from_raw](Resource::from_raw) to avoid a memroy leak. Despite this however, this method is not
    ///considered unsafe for much the same reason [Box::into_raw] is not
    fn into_raw(self) -> GLuint;

    ///
    ///Constructs an object from a raw GL object id or [Option::None] if the id is not a name of
    ///
    ///
    ///This id should be one from a previous call to [into_raw](Resource::into_raw) or an unowned object
    ///created manually otherwise there almost certainly will be a double-free. However, it is _not_
    ///unsafe to provide an invalid id or an id from a different type as OpenGL and the implementor
    ///should catch them and return a None
    ///
    ///# Unsafety
    ///
    ///Calling this on the same id twice will almost certainly cause a double-free and/or other memory
    ///issues from double-ownership
    ///
    unsafe fn from_raw(id:GLuint) -> Option<Self>;


    ///Creates a new OpenGL resource of this type
    fn gen(gl: &Self::GL) -> Self;

    ///Creates an array of new OpenGL resources
    fn gen_resources(gl: &Self::GL, count: GLuint) -> Box<[Self]>;

    ///Determines if a given id is the name of an OpenGL resource of this type
    fn is(GLuint) -> bool;

    ///Returns true if the two OpenGL objects are the same _object_ without checking value-equivalence
    #[inline] fn obj_eq<R:Resource+?Sized>(&self, rhs:&R) -> bool {self.id()==rhs.id()}

    ///
    ///Consumes this object and deletes its OpenGL resource.
    ///
    ///Do note though that if this object is queued for an operation (unless synchronization is
    ///overrided by various (unsafe) means) the object will continue to exist until those operations
    ///have completed. Furthermore, certain objects will continue to exist in memory if bound
    ///to certain targets similar to a reference counted smart-pointer.
    ///
    fn delete(self);

    ///
    ///Consumes an array of objects and deletes their OpenGL resources.
    ///
    ///Do note though that if this object is queued for an operation (unless synchronization is
    ///overrided by various (unsafe) means) the object will continue to exist until those operations
    ///have completed. Furthermore, certain objects will continue to exist in memory if bound
    ///to certain targets similar to a reference counted smart-pointer.
    ///
    fn delete_resources(resouces: Box<[Self]>);

}

///
///An OpenGL Enum that corresponds to target arguments in the glBind* functions
///
///# Unsafety
///
///It is up to the implementor to make sure that the possible enum values are valid arguments to
///whichever glBind* function is being called
///
pub unsafe trait Target: GLEnum {

    type Resource: Resource<BindingTarget=Self>;

    ///
    ///Binds the given resource id to this target
    ///
    ///# Unsafety
    ///
    ///The caller must make sure that the id corresponds to a valid resource id of a type valid for
    ///this particlular glBind* function. Furthermore, the caller must guarrantee that this target is
    ///valid for the configuration of the resource
    ///
    unsafe fn bind(self, id: GLuint);

    ///
    ///Constructs a new binding location with the given target
    ///
    ///# Unsafety
    ///It is up to the caller to guarrantee that this is the only location with the given
    ///[binding target](Target) at the given time
    #[inline]
    unsafe fn as_loc(self) -> BindingLocation<Self::Resource> {
        BindingLocation(self)
    }
}

///An object that owns a [Target] to a glBind* function for a resource `R`
#[derive(PartialEq, Eq, Hash)]
pub struct BindingLocation<R:Resource>(pub(crate) R::BindingTarget);

///An object that owns a binding of a [Resource] to a particular [BindingLocation] and unbinds it when leaving scope
pub struct Binding<'a,R:Resource>(pub(crate) &'a BindingLocation<R>, pub(crate) GLuint);

impl<'a,R:Resource> Binding<'a,R> {
    #[inline] pub fn target(&self) -> R::BindingTarget { self.0.target() }
    #[inline] pub fn target_id(&self) -> GLenum { self.0.target_id() }
    #[inline] pub fn resource_id(&self) -> GLuint { self.1 }
}

impl<'a,R:Resource> !Sync for Binding<'a,R> {}
impl<'a,R:Resource> !Send for Binding<'a,R> {}
impl<'a,R:Resource> Drop for Binding<'a,R> {
    #[inline] fn drop(&mut self) { unsafe { self.target().bind(0) } }
}

impl<R:Resource> !Sync for BindingLocation<R> {}
impl<R:Resource> !Send for BindingLocation<R> {}
impl<R:Resource> BindingLocation<R> {

    ///The [target](Target) of this location
    pub fn target(&self) -> R::BindingTarget { self.0 }

    ///The the [GLenum] corresponding to this location's target
    pub fn target_id(&self) -> GLenum { self.0.into() }

    ///
    ///Constructs a new binding location with the given target
    ///
    ///# Unsafety
    ///It is up to the caller to guarrantee that this is the only location with the given
    ///[binding target](Target) at the given time
    #[inline]
    pub unsafe fn new(target: R::BindingTarget) -> Self {BindingLocation(target)}

    ///
    ///A wrapper of glBind* for `R` using a raw resource id
    ///
    ///# Safety
    ///
    ///Do note that This method is actually _safe_. While it certainly appears as if it wouldn't be,
    ///all possible unsafe sources are already accounted for:
    /// * We already know that this is the only [BindingLocation] for its [Target] as its construction
    ///   is marked as unsafe
    /// * This does not violate memory safety as any object modification must happen from an unsafe
    ///   OpenGL call
    /// * Even if the `id` is not a valid resource name for `R`, we can easily check with
    ///   a glIs* function
    /// * While not really unsafe _per se_, the resource will never remain bound outside its lifetime
    ///   due to the implementation of [Drop] on [Binding]
    /// * The buffer cannot be deleted before being unbound without running unsafe fuctions
    ///
    ///# Errors
    ///
    ///A [GLError::InvalidOperation] is returned if `id` is not a name of a resource of type `R`
    ///
    #[inline]
    pub fn bind_raw<'a>(&'a mut self, id: GLuint) -> Result<Binding<'a,R>, GLError> {
        if R::is(id) {
            unsafe { self.target().bind(id); }
            Ok(Binding(self, id))
        } else {
            Err(GLError::InvalidOperation("Cannot bind resource to the given target".to_string()))
        }
    }

    #[inline]
    pub unsafe fn bind_unchecked<'a>(&'a mut self, id: GLuint) -> Binding<'a,R> {
        self.target().bind(id);
        Binding(self, id)
    }

    ///
    ///A wrapper of glBind* for `R` using an owned resource
    ///
    ///The fact that it is owned means that we can bind without checking validity of the resource
    ///id. Furthermore, for the same reasons as [bind_raw](BindingLocation::bind_raw), this method is actually safe
    ///
    #[inline]
    pub fn bind<'a>(&'a mut self, resource: &'a R) -> Binding<'a,R> {
        unsafe { self.target().bind(resource.id()); }
        Binding(self, resource.id())
    }

}
