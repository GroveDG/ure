use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    ops::Range,
};

pub use bitvec::{slice::BitSlice, vec::BitVec};
pub use indexmap::IndexSet;
use multimap::MultiMap;
pub use one_or_many::OneOrMany;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MethodId(*const ());
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalId(u64);
impl SignalId {
    pub const fn new(ident: &str) -> Self {
        Self(const_fnv1a_hash::fnv1a_hash_str_64(ident))
    }
}

pub const NEW: SignalId = SignalId::new("New");
pub const DELETE: SignalId = SignalId::new("Delete");

type ComponentBox = Box<dyn Any>;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct AnyFn(*const (), TypeId);
impl AnyFn {
    fn is<T: Any>(&self) -> bool {
        TypeId::of::<T>() == self.1
    }
}
type MethodFn<C, Args> =
    fn(<<C as ComponentRetrieve>::Containers as ContainerRetrieve>::Mut<'_>, Args);
type MethodCall<Args> = fn(&Method, &mut Components, Args);

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Method {
    method: AnyFn,
    receiver: AnyFn,
    components: Vec<ComponentId>,
}
impl Method {
    pub fn new<C: ComponentRetrieve + 'static, Args: 'static>(f: MethodFn<C, Args>, components: C) -> Self {
        Self {
            method: AnyFn(f as *const (), TypeId::of::<MethodFn<C, Args>>()),
            receiver: AnyFn(Self::call::<C, Args> as *const (), TypeId::of::<Args>()),
            components: components.ids(),
        }
    }
    pub fn call<C: ComponentRetrieve + 'static, Args: 'static>(
        &self,
        components: &mut Components,
        args: Args,
    ) -> Result<(), ()> {
        if self.method.is::<MethodFn<C, Args>>() {
            return Err(());
        }
        let method: MethodFn<C, Args> = unsafe { std::mem::transmute(self.method.0) };
        let Some(components) = C::retrieve_mut(components) else {
            return Err(());
        };
        (method)(components, args);
        Ok(())
    }
    pub fn recv<Args: 'static>(&self, components: &mut Components, args: Args) -> Result<(), ()> {
        if self.method.is::<Args>() {
            return Err(());
        }
        let receiver: MethodCall<Args> = unsafe { std::mem::transmute(self.receiver.0) };
        (receiver)(self, components, args);
        Ok(())
    }
    pub fn id(&self) -> MethodId {
        MethodId(self.method.0)
    }
}

type Components = HashMap<ComponentId, ComponentBox>;
type Signals = MultiMap<SignalId, Method>;
type Connection = MultiMap<MethodId, SignalId>;
type Dependency = MultiMap<ComponentId, MethodId>;

#[derive(Default)]
pub struct Group {
    len: usize,
    components: Components,
    signals: Signals,
    connection: Connection,
    dependency: Dependency,
}

impl Group {
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn add_component<C: Container>(&mut self, component: Component<C>, new: Method, delete: Method) {
        self.components.insert(component.id, Box::new(C::new()));
        new.recv(&mut self.components, 0..self.len);
        self.connect(new, NEW);
        self.connect(delete, DELETE);
    }
    fn connect(&mut self, method: Method, signal: SignalId) -> Result<(), ()> {
        for component in method.components {
            if !self.components.contains_key(component) {
                return Err(());
            }
        }
        for component in method.components.iter().copied() {
            self.dependency.insert(component, method.id());
        }
        self.signals.insert(signal, method);
        Ok(())
    }
    fn disconnect(&mut self, signal: &SignalId, method: &MethodId) -> Result<(), ()> {
        let Some(signal) = self.signals.get_vec_mut(signal) else {
            return Err(());
        };
        for _ in signal.extract_if(.., |m| &m.id() == method) {}
        Ok(())
    }
    pub fn get_components<C: ComponentRetrieve>(
        &self,
        components: &C,
    ) -> Option<<C::Containers as ContainerRetrieve>::Ref<'_>> {
        components.retrieve(&self.components)
    }
    pub fn get_components_mut<C: ComponentRetrieve>(
        &mut self,
        components: &C,
    ) -> Option<<C::Containers as ContainerRetrieve>::Mut<'_>> {
        components.retrieve_mut(&mut self.components)
    }
    pub fn remove_component(&mut self, id: &ComponentId) -> Result<(), ()> {
        let Some(dependents) = self.dependency.remove(id) else {
            return Err(());
        };
        for method in dependents {
            let Some(signals) = self.connection.remove(&method) else {
                continue;
            };
            for signal in signals {
                self.disconnect(&signal, &method);
            }
        }
        self.components.remove(id);
        Ok(())
    }
    pub fn signal<Args: Clone + 'static>(&mut self, signal: &SignalId, args: Args) {
        let Some(signal) = self.signals.get_vec(signal) else {
            return;
        };
        for method in signal {
            method.recv(&mut self.components, args.clone());
        }
    }
    pub fn new(&mut self, num: usize) {
        self.signal(&NEW, self.len..self.len + num);
        self.len += num;
    }
    pub fn delete(&mut self, range: Range<usize>) {
        self.signal(&DELETE, range.clone());
        self.len -= range.len();
    }
}

pub trait ContainerRetrieve: 'static {
    type Ref<'a>;
    type Mut<'a>;
}

pub trait Container: Any + ContainerRetrieve {
    fn new() -> Self;
    fn container_ref(&self) -> Self::Ref<'_>;
    fn container_mut(&mut self) -> Self::Mut<'_>;
}

impl ContainerRetrieve for () {
    type Ref<'a> = ();
    type Mut<'a> = ();
}
impl Container for () {
    fn new() -> Self {}
    fn container_ref(&self) -> Self::Ref<'_> {}
    fn container_mut(&mut self) -> Self::Mut<'_> {}
}

impl<T: 'static> ContainerRetrieve for Vec<T> {
    type Ref<'a> = &'a [T];
    type Mut<'a> = &'a mut [T];
}
impl<T: 'static> Container for Vec<T> {
    fn new() -> Self {
        Vec::new()
    }
    fn container_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn container_mut(&mut self) -> Self::Mut<'_> {
        self
    }
}

impl<T: 'static> ContainerRetrieve for IndexSet<T> {
    type Ref<'a> = &'a IndexSet<T>; // TODO: prevent structural modifications
    type Mut<'a> = &'a mut IndexSet<T>;
}
impl<T: 'static> Container for IndexSet<T> {
    fn new() -> Self {
        IndexSet::new()
    }
    fn container_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn container_mut(&mut self) -> Self::Mut<'_> {
        self
    }
}

impl<T: 'static> ContainerRetrieve for OneOrMany<T> {
    type Ref<'a> = &'a [T];
    type Mut<'a> = &'a mut [T];
}
impl<T: 'static> Container for OneOrMany<T> {
    fn new() -> Self {
        OneOrMany::None
    }
    fn container_ref(&self) -> Self::Ref<'_> {
        self.as_slice()
    }
    fn container_mut(&mut self) -> Self::Mut<'_> {
        self.as_mut_slice()
    }
}

impl ContainerRetrieve for BitVec {
    type Ref<'a> = &'a BitSlice;
    type Mut<'a> = &'a mut BitSlice;
}
impl Container for BitVec {
    fn new() -> Self {
        BitVec::new()
    }
    fn container_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn container_mut(&mut self) -> Self::Mut<'_> {
        self
    }
}
// GPU Buffer container defined in the URE GPU crate.

pub struct Component<C: Container> {
    id: ComponentId,
    _marker: PhantomData<C>,
}

pub trait ComponentRetrieve: 'static {
    type Containers: ContainerRetrieve;

    fn retrieve<'a>(
        &self,
        components: &'a Components,
    ) -> Option<<Self::Containers as ContainerRetrieve>::Ref<'a>>;
    fn retrieve_mut<'a>(
        &self,
        components: &'a mut Components,
    ) -> Option<<Self::Containers as ContainerRetrieve>::Mut<'a>>;
    fn ids(&self) -> Vec<ComponentId>;
}

// impl ComponentRetrieve for Component<()> {
//     type Containers = ();
//     fn retrieve(_: &Components) -> Option<<Self::Containers as Container>::Ref<'_>> {
//         Some(())
//     }
//     fn retrieve_mut(_: &mut Components) -> Option<<Self::Containers as Container>::Mut<'_>> {
//         Some(())
//     }
// }
impl<C: Container> ComponentRetrieve for Component<C> {
    type Containers = C;

    fn retrieve<'a>(
        &self,
        components: &'a Components,
    ) -> Option<<Self::Containers as ContainerRetrieve>::Ref<'a>> {
        Some(
            components
                .get(&self.id)?
                .downcast_ref::<C>()?
                .container_ref(),
        )
    }
    fn retrieve_mut<'a>(
        &self,
        components: &'a mut Components,
    ) -> Option<<Self::Containers as ContainerRetrieve>::Mut<'a>> {
        Some(
            components
                .get_mut(&self.id)?
                .downcast_mut::<C>()?
                .container_mut(),
        )
    }
    fn ids(&self) -> Vec<ComponentId> {
        vec![self.id]
    }
}

macro_rules! container_tuples {
	($($T:ident),*) => {
#[allow(non_snake_case)]
impl<$($T: Container),*> ContainerRetrieve for ($($T),*) {
	type Ref<'a> = ($($T::Ref<'a>),*);
	type Mut<'a> = ($($T::Mut<'a>),*);
}
#[allow(non_snake_case)]
impl<$($T: Container),*> ComponentRetrieve for ($(Component<$T>),*) {
	type Containers = ($($T),*);

	fn retrieve<'a>(&self, components: &'a Components) -> Option<<Self::Containers as ContainerRetrieve>::Ref<'a>> {
        let ($($T),*) = self;
		$(
			let $T = components
			.get(&$T.id)?
			.downcast_ref::<$T>()?
			.container_ref();
		)*
		Some(($($T),*))
	}
	fn retrieve_mut<'a>(
        &self,
		components: &'a mut Components,
	) -> Option<<Self::Containers as ContainerRetrieve>::Mut<'a>> {
        let ($($T),*) = self;
		let [$($T),*] = components.get_disjoint_mut([
            $(&$T.id),*
        ]);
		$(
			let $T = $T?.downcast_mut::<$T>()?.container_mut();
		)*
		Some(($($T),*))
	}
    fn ids(&self) -> Vec<ComponentId> {
        let ($($T),*) = self;
        vec![
            $($T.id),*
        ]
    }
}
	};
}
container_tuples!(A, B);
container_tuples!(A, B, C);
container_tuples!(A, B, C, D);
container_tuples!(A, B, C, D, E);
container_tuples!(A, B, C, D, E, F);
container_tuples!(A, B, C, D, E, F, G);
container_tuples!(A, B, C, D, E, F, G, H);
container_tuples!(A, B, C, D, E, F, G, H, I);
container_tuples!(A, B, C, D, E, F, G, H, I, J);
