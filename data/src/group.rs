use std::{any::Any, cell::UnsafeCell, collections::HashMap, marker::PhantomData, ops::Range};

pub use bitvec::{slice::BitSlice, vec::BitVec};
use indexmap::IndexMap;
pub use indexmap::IndexSet;
use multimap::MultiMap;
pub use one_or_many::OneOrMany;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentIdInner(u64);
impl ComponentIdInner {
    const fn new(ident: &str) -> Self {
        Self(const_fnv1a_hash::fnv1a_hash_str_64(ident))
    }
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ComponentId<C: Container> {
    inner: ComponentIdInner,
    _marker: PhantomData<C>,
}
impl<C: Container> ComponentId<C> {
    pub const fn new(ident: &str) -> Self {
        Self {
            inner: ComponentIdInner::new(ident),
            _marker: PhantomData,
        }
    }
    pub const fn inner(&self) -> ComponentIdInner {
        self.inner
    }
    pub unsafe fn get_container(&self, components: &Components) -> Option<&C> {
        unsafe { components.get(&self.inner)?.get().as_mut() }?.downcast_ref::<C>()
    }
    pub unsafe fn get_container_mut(&self, components: &Components) -> Option<&mut C> {
        unsafe { components.get(&self.inner)?.get().as_mut() }?.downcast_mut::<C>()
    }
    pub unsafe fn get(&self, components: &Components) -> Option<C::Ref<'_>> {
        Some(
            unsafe { components.get(&self.inner)?.get().as_mut() }?
                .downcast_ref::<C>()?
                .as_ref(),
        )
    }
    pub unsafe fn get_mut(&self, components: &Components) -> Option<C::Mut<'_>> {
        Some(
            unsafe { components.get(&self.inner)?.get().as_mut() }?
                .downcast_mut::<C>()?
                .as_mut(),
        )
    }
}
impl<C: Container> Clone for ComponentId<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: self._marker.clone(),
        }
    }
}
impl<C: Container> Copy for ComponentId<C> {}
#[macro_export]
macro_rules! component {
    ($v:vis $name:ident : $t:ty) => {
$v const $name : $crate::ComponentId<$t> = $crate::ComponentId::new(stringify!($name));
    };
}

type MethodFn<Args = (), Return = ()> = fn(&mut Components, Args) -> Return;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MethodIdInner(u64);
impl MethodIdInner {
    const fn new(ident: &str) -> Self {
        Self(const_fnv1a_hash::fnv1a_hash_str_64(ident))
    }
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct MethodId<Args = (), Return = ()> {
    inner: MethodIdInner,
    _marker: PhantomData<MethodFn<Args, Return>>,
}
impl<Args, Return> MethodId<Args, Return> {
    pub const fn new(ident: &str) -> Self {
        Self {
            inner: MethodIdInner::new(ident),
            _marker: PhantomData,
        }
    }
    pub const fn inner(&self) -> MethodIdInner {
        self.inner
    }
}
impl<Args, Return> Clone for MethodId<Args, Return> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: self._marker.clone(),
        }
    }
}
impl<Args, Return> Copy for MethodId<Args, Return> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalIdInner(u64);
impl SignalIdInner {
    const fn new(ident: &str) -> Self {
        Self(const_fnv1a_hash::fnv1a_hash_str_64(ident))
    }
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SignalId<Args> {
    inner: SignalIdInner,
    _marker: PhantomData<MethodFn<Args>>,
}
impl<Args> SignalId<Args> {
    const fn new(ident: &str) -> Self {
        Self {
            inner: SignalIdInner::new(ident),
            _marker: PhantomData,
        }
    }
    pub const fn inner(&self) -> SignalIdInner {
        self.inner
    }
}
impl<Args> Clone for SignalId<Args> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: self._marker.clone(),
        }
    }
}
impl<Args> Copy for SignalId<Args> {}
#[macro_export]
macro_rules! signal {
    ($v:vis $name:ident : $t:ty) => {
$v const $name : $crate::SignalId<$t> = $crate::SignalId::new(stringify!($name));
    };
}

signal!(NEW: Range<usize>);
signal!(DELETE: Range<usize>);

pub struct ComponentStruct<C: Container> {
    pub id: ComponentId<C>,
    pub container: C,
    pub len: usize,
    pub new: Method<Range<usize>>,
    pub delete: Method<Range<usize>>,
}
impl<C: Container + Default> ComponentStruct<C> {
    pub fn new(
        id: ComponentId<C>,
        new: Method<Range<usize>>,
        delete: Method<Range<usize>>,
    ) -> Self {
        Self {
            id,
            container: Default::default(),
            len: 0,
            new,
            delete,
        }
    }
}
pub struct Method<Args = (), Return = ()> {
    id: MethodId<Args, Return>,
    func: MethodFn<Args, Return>,
    dependencies: &'static [ComponentIdInner],
}
impl<Args, Return> Method<Args, Return> {
    pub const fn new(
        ident: &str,
        func: MethodFn<Args, Return>,
        dependencies: &'static [ComponentIdInner],
    ) -> Self {
        Self {
            id: MethodId::new(ident),
            func,
            dependencies,
        }
    }
}
#[macro_export]
macro_rules! method_custom {
    (
        ( $($extract_name:ident : $extract:ident),* $(,)? )
        ( $($get_name:ident : $get:ident),* $(,)? )
        $args_name:ident
        $f:block
    ) => {
        $crate::Method::new(
            stringify!($name),
            |components, $args_name| {
                unsafe{
                $(
                let $extract_name = $extract.get_container_mut(components).unwrap();
                )*
                $(
                let $get_name = $get.get_mut(components).unwrap();
                )*
                $f
                }
            },
            &[
                $( $extract.inner(), )*
                $( $get.inner(), )*
            ]
        )
    };
}
#[macro_export]
macro_rules! method {
    ($v:vis $func:ident
        ( $($extract:ident),* $(,)? )
        ( $($get:ident),* $(,)? )
        $( $args:ty $(,)? )?
    ) => {
        $crate::mident::mident!{
        $v const #upcase $func: $crate::Method<($($args)?)> = $crate::method_custom!(
            ( $(#downcase $extract : $extract),* ) ( $(#downcase $get : $get),* ) args {
                ($func)(
                    $( #downcase $extract, )*
                    $( #downcase $get, )*
                    $( args as $args )?
                )
            }
        );
        }
    };
}
#[macro_export]
macro_rules! new {
    (
        $v:vis
        $func:ident
        $component:ident
        ( $($get:ident),* $(,)? )
    ) => {
        $crate::mident::mident!(
        $v const #upcase $func: $crate::Method<(std::ops::Range<usize>)> = $crate::method_custom!(
            ( #downcase $component : $component ) ( $(#downcase $get : $get),* ) range {
                for i in range {
                    ($func)(
                        #downcase $component,
                        $(#downcase $get, )*
                        i
                    )
                }
            }
        );
        );
    };
}

#[derive(Debug, Default)]
pub struct Signal<Args: Clone> {
    methods: IndexMap<MethodIdInner, MethodFn<Args>>,
}
impl<Args: Clone> Signal<Args> {
    pub fn insert(&mut self, id: MethodId<Args>, method: MethodFn<Args>) {
        self.methods.insert(id.inner, method);
    }
    pub fn call(&self, components: &mut Components, args: Args) {
        for method in self.methods.values().copied() {
            (method)(components, args.clone());
        }
    }
}
trait Signaling {
    fn disconnect(&mut self, id: &MethodIdInner);
}
impl<Args: Clone> Signaling for Signal<Args> {
    fn disconnect(&mut self, id: &MethodIdInner) {
        self.methods.shift_remove(id);
    }
}

type ComponentBox = Box<dyn Any>;
type SignalBox = Box<dyn Signaling>;

pub type Components = HashMap<ComponentIdInner, UnsafeCell<ComponentBox>>;
#[derive(Default)]
struct Signals {
    inner: HashMap<SignalIdInner, SignalBox>,
}
impl Signals {
    fn get(&self, id: &SignalIdInner) -> Option<&dyn Signaling> {
        Some(self.inner.get(id)?.as_ref())
    }
    fn get_mut(&mut self, id: &SignalIdInner) -> Option<&mut dyn Signaling> {
        Some(self.inner.get_mut(id)?.as_mut())
    }
    fn get_signal<Args: Clone>(&self, id: &SignalId<Args>) -> Option<&Signal<Args>> {
        let signal = self.inner.get(&id.inner)?.as_ref();
        unsafe { (signal as *const dyn Signaling as *const Signal<Args>).as_ref() }
    }
    fn get_signal_mut<Args: Clone>(&mut self, id: &SignalId<Args>) -> Option<&mut Signal<Args>> {
        let signal = self.inner.get_mut(&id.inner)?.as_mut();
        unsafe { (signal as *mut dyn Signaling as *mut Signal<Args>).as_mut() }
    }
}
type Connection = MultiMap<MethodIdInner, SignalIdInner>;
type Dependency = MultiMap<ComponentIdInner, MethodIdInner>;

#[derive(Default)]
pub struct Group {
    len: usize,
    components: Components,
    signals: Signals,
    connection: Connection,
    dependency: Dependency,
}

impl Group {
    pub fn add_component<C: Container>(&mut self, component: ComponentStruct<C>) -> Result<(), ()> {
        let ComponentStruct::<C> {
            id,
            container,
            len,
            new,
            delete,
        } = component;

        let (new_id, delete_id) = (new.id, delete.id);
        let new_func = new.func;

        self.components
            .insert(id.inner, UnsafeCell::new(Box::new(container)));
        let new_ok = self.connect(NEW, new).is_ok();
        let delete_ok = self.connect(DELETE, delete).is_ok();
        if new_ok && delete_ok {
            (new_func)(&mut self.components, len..self.len);
            return Ok(());
        }
        if !new_ok {
            self.disconnect(&NEW.inner, &new_id.inner).unwrap();
        }
        if !delete_ok {
            self.disconnect(&DELETE.inner, &delete_id.inner).unwrap();
        }
        self.components.remove(&id.inner);
        Err(())
    }
    pub fn remove_component<C: Container>(&mut self, id: &ComponentId<C>) {
        self.components.remove(&id.inner);

        for method in self.dependency.remove(&id.inner).unwrap() {
            for signal in self.connection.remove(&method).unwrap() {
                self.signals.get_mut(&signal).unwrap().disconnect(&method);
            }
        }
    }
    fn connect<Args: Clone>(
        &mut self,
        signal_id: SignalId<Args>,
        method: Method<Args>,
    ) -> Result<(), ()> {
        let Method::<Args> {
            id,
            func,
            dependencies,
        } = method;

        for component in dependencies {
            if !self.components.contains_key(component) {
                return Err(());
            }
        }
        let Some(signal) = self.signals.get_signal_mut(&signal_id) else {
            return Err(());
        };
        signal.insert(id, func);

        for component in method.dependencies.iter().copied() {
            self.dependency.insert(component, id.inner);
        }
        self.connection.insert(id.inner, signal_id.inner);

        Ok(())
    }
    fn disconnect(&mut self, signal: &SignalIdInner, method: &MethodIdInner) -> Result<(), ()> {
        let Some(signal) = self.signals.get_mut(signal) else {
            return Err(());
        };
        signal.disconnect(method);
        Ok(())
    }
    pub fn call_signal<Args: Clone>(&mut self, id: &SignalId<Args>, args: Args) -> Result<(), ()> {
        let Some(signal) = self.signals.get_signal(id) else {
            return Err(());
        };
        signal.call(&mut self.components, args);
        Ok(())
    }
    pub unsafe fn get_component<C: Container>(
        &self,
        component: ComponentId<C>,
    ) -> Option<C::Ref<'_>> {
        Some(unsafe {
            self.components
                .get(&component.inner)?
                .get()
                .as_ref()?
                .downcast_ref::<C>()?
                .as_ref()
        })
    }
    pub unsafe fn get_component_mut<C: Container>(
        &self,
        component: ComponentId<C>,
    ) -> Option<C::Mut<'_>> {
        Some(unsafe {
            self.components
                .get(&component.inner)?
                .get()
                .as_mut()?
                .downcast_mut::<C>()?
                .as_mut()
        })
    }
    pub unsafe fn get_container<C: Container>(
        &self,
        component: ComponentId<C>,
    ) -> Option<C::Ref<'_>> {
        Some(unsafe {
            self.components
                .get(&component.inner)?
                .get()
                .as_ref()?
                .downcast_ref::<C>()?
                .as_ref()
        })
    }
    pub unsafe fn get_container_mut<C: Container>(
        &self,
        component: ComponentId<C>,
    ) -> Option<C::Mut<'_>> {
        Some(unsafe {
            self.components
                .get(&component.inner)?
                .get()
                .as_mut()?
                .downcast_mut::<C>()?
                .as_mut()
        })
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

pub trait Container: Any {
    type Ref<'a>;
    type Mut<'a>;

    fn as_ref(&self) -> Self::Ref<'_>;
    fn as_mut(&mut self) -> Self::Mut<'_>;
    fn delete(&mut self, range: Range<usize>);
}
impl Container for () {
    type Ref<'a> = ();
    type Mut<'a> = ();

    fn as_ref(&self) -> Self::Ref<'_> {}
    fn as_mut(&mut self) -> Self::Mut<'_> {}
    fn delete(&mut self, _: Range<usize>) {}
}
#[derive(Debug, Default)]
pub struct One<T: 'static>(pub T);
impl<T: 'static> Container for One<T> {
    type Ref<'a> = &'a T;
    type Mut<'a> = &'a mut T;

    fn as_ref(&self) -> Self::Ref<'_> {
        &self.0
    }
    fn as_mut(&mut self) -> Self::Mut<'_> {
        &mut self.0
    }
    fn delete(&mut self, _: Range<usize>) {}
}
impl<T: 'static> Container for Option<T> {
    type Ref<'a> = Option<&'a T>;
    type Mut<'a> = Option<&'a mut T>;

    fn as_ref(&self) -> Self::Ref<'_> {
        self.as_ref()
    }
    fn as_mut(&mut self) -> Self::Mut<'_> {
        self.as_mut()
    }
    fn delete(&mut self, _: Range<usize>) {}
}
impl<T: 'static> Container for Vec<T> {
    type Ref<'a> = &'a [T];
    type Mut<'a> = &'a mut [T];

    fn as_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn as_mut(&mut self) -> Self::Mut<'_> {
        self
    }
    fn delete(&mut self, range: Range<usize>) {
        for i in range {
            self.swap_remove(i);
        }
    }
}
impl<T: 'static> Container for IndexSet<T> {
    type Ref<'a> = &'a IndexSet<T>; // TODO: prevent structural modifications
    type Mut<'a> = &'a mut IndexSet<T>;

    fn as_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn as_mut(&mut self) -> Self::Mut<'_> {
        self
    }
    fn delete(&mut self, range: Range<usize>) {
        for i in range {
            self.swap_remove_index(i);
        }
    }
}
impl<T: 'static> Container for OneOrMany<T> {
    type Ref<'a> = &'a [T];
    type Mut<'a> = &'a mut [T];

    fn as_ref(&self) -> Self::Ref<'_> {
        self.as_slice()
    }
    fn as_mut(&mut self) -> Self::Mut<'_> {
        self.as_mut_slice()
    }
    fn delete(&mut self, range: Range<usize>) {
        if let OneOrMany::Many(items) = self {
            for i in range {
                items.swap_remove(i);
            }
        }
    }
}
impl Container for BitVec {
    type Ref<'a> = &'a BitSlice;
    type Mut<'a> = &'a mut BitSlice;

    fn as_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn as_mut(&mut self) -> Self::Mut<'_> {
        self
    }
    fn delete(&mut self, range: Range<usize>) {
        for i in range {
            self.swap_remove(i);
        }
    }
}
// GPU Buffer container defined in the URE GPU crate.
