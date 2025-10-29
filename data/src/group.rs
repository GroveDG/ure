use std::{cell::UnsafeCell, collections::HashMap, hash::Hash, marker::PhantomData, ops::Range};

use indexmap::IndexMap;
use multimap::MultiMap;

use crate::{ComponentContainer, ComponentId, ComponentIdInner, Components, container::Container};

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

#[derive(Debug)]
pub struct Method<Args = (), Return = ()> {
    id: MethodId<Args, Return>,
    func: MethodFn<Args, Return>,
    dependencies: &'static [ComponentIdInner],
}
impl<Args, Return> Clone for Method<Args, Return> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            func: self.func,
            dependencies: self.dependencies,
        }
    }
}
impl<Args, Return> Copy for Method<Args, Return> {}
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
        ( $($extract_name:ident : $extract:expr),* $(,)? ),
        ( $($get_name:ident : $get:expr),* $(,)? ),
        $components:ident,
        $args_name:ident,
        $f:block $(,)?
    ) => {
        $crate::Method::new(
            stringify!($name),
            |$components, $args_name| {
                unsafe{
                $(
                let $extract_name = $extract.id.get_container_mut($components).unwrap();
                )*
                $(
                let $get_name = $get.id.get_mut($components).unwrap();
                )*
                $f
                }
            },
            &[
                $( $extract.id.inner(), )*
                $( $get.id.inner(), )*
            ]
        )
    };
}
#[macro_export]
macro_rules! method {
    (
        $v:vis $name:ident
        ( $( $get_name:ident : $get:expr ),* $(,)? ),
        $( $args:ty, )?
        $func:block $(,)?
    ) => {
        $v const $name: $crate::Method<($($args)?)> = $crate::method_custom!(
            (),
            ( $( $get_name : $get),* ), components, args, {
                $func
            }
        );
    };
    (
        $v:vis $name:ident
        ( $i:ident, $( $get_name:ident : $get:expr ),* $(,)? ),
        $( $args:ty, )?
        $func:block $(,)?
    ) => {
        $v const $name: $crate::Method<($($args)?)> = $crate::method_custom!(
            (),
            ( $( $get_name : $get),* ), components, args, {
                for $ident in 0..components.len() {
                    $func
                }
            }
        );
    };
    (
        $v:vis $name:ident
        ( _, $( $get_name:ident : $get:expr ),* $(,)? ),
        $( $args:ty, )?
        $func:block $(,)?
    ) => {
        $v const $name: $crate::Method<($($args)?)> = $crate::method_custom!(
            (),
            ( $( $get_name : $get),* ), components, args, {
                for i in 0..components.len() {
                    $(
                    let $get_name = &mut $get_name[i];
                    )*
                    $func
                }
            }
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

type SignalBox = Box<dyn Signaling>;

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
    pub fn add_component<C: Container>(
        &mut self,
        component: ComponentContainer<C>,
    ) -> Result<(), ()> {
        let ComponentContainer::<C> {
            id,
            container,
            len,
            new,
            delete,
        } = component;

        let (new_id, delete_id) = (new.id, delete.id);
        let new_func = new.func;

        self.components
            .insert(id.inner(), UnsafeCell::new(Box::new(container)));
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
        self.components.remove(&id.inner());
        Err(())
    }
    pub fn remove_component<C: Container>(&mut self, id: &ComponentId<C>) {
        self.components.remove(&id.inner());

        for method in self.dependency.remove(&id.inner()).unwrap() {
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
    pub fn call_method<Args, Return>(
        &mut self,
        method: Method<Args, Return>,
        args: Args,
    ) -> Return {
        (method.func)(&mut self.components, args)
    }
    pub fn new(&mut self, num: usize) {
        self.call_signal(&NEW, self.len..self.len + num).unwrap();
        self.len += num;
    }
    pub fn delete(&mut self, num: usize) {
        self.call_signal(&DELETE, self.len..self.len + num).unwrap();
        self.len -= num;
    }
    pub unsafe fn get_component<C: Container>(
        &self,
        component: ComponentId<C>,
    ) -> Option<C::Ref<'_>> {
        Some(unsafe {
            self.components
                .get(&component.inner())?
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
                .get(&component.inner())?
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
                .get(&component.inner())?
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
                .get(&component.inner())?
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
