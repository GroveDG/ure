use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    ops::Range,
};

pub use bitvec::{slice::BitSlice, vec::BitVec};
pub use indexmap::IndexSet;
pub use one_or_many::OneOrMany;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalId(u64);
impl SignalId {
    pub const fn new(ident: &str) -> Self {
        Self(const_fnv1a_hash::fnv1a_hash_str_64(ident))
    }
}

pub const NEW: SignalId = SignalId::new("New");
pub const DELETE: SignalId = SignalId::new("Delete");

pub trait Container: Any {
    type Ref<'a>;
    type Mut<'a>;

    fn new() -> Self;
    fn container_ref(&self) -> Self::Ref<'_>;
    fn container_mut(&mut self) -> Self::Mut<'_>;
}

pub trait Component: 'static {
    const ID: ComponentId = ComponentId(const_fnv1a_hash::fnv1a_hash_str_64(Self::IDENT));
    const IDENT: &'static str;

    type Container: Container;
}

pub struct Method<Components: ComponentRetrieve, Args> {
    id: FunctionId,
    f: Box<dyn FnMut(<Components::Containers as Container>::Mut<'_>, Args)>,
    _marker: PhantomData<Components>,
}

impl<C: ComponentRetrieve + 'static, Args: 'static> Method<C, Args> {
    fn call(method: &mut dyn Any, components: &mut Components, args: Args) {
        let Some(method) = method.downcast_mut::<Method<C, Args>>() else {
            return;
        };
        let Some(components) = C::retrieve_mut(components) else {
            return;
        };
        (method.f)(components, args)
    }
}

type ComponentBox = Box<dyn Any>;

struct MethodBox {
    any: Box<dyn Any>,
    id: FunctionId,
    components: &'static [ComponentId],
    signals: Vec<SignalId>,
}
impl MethodBox {
    pub fn new<C: ComponentRetrieve + 'static, A: 'static>(method: Method<C, A>) -> Self {
        Self {
            id: method.id,
            any: Box::new(method),
            components: C::IDS,
            signals: Vec::new(),
        }
    }
}

type MethodCall<Args> = fn(&mut dyn Any, &mut Components, Args);

pub struct Connection<Args> {
    id: FunctionId,
    f: MethodCall<Args>,
}

struct SignalBox<Args> {
    connections: Vec<Connection<Args>>,
}
impl<Args> SignalBox<Args> {
    fn connect(&mut self, method: FunctionId, call: MethodCall<Args>) {
        self.connections.push(Connection { id: method, f: call });
    }
    fn call(&self, components: &mut Components, methods: &mut Methods, args: Args) {
        for connection in self.connections.iter() {
            let Some(method) = methods.get_mut(&connection.id) else {
                continue;
            };
            (connection.f)(&mut method.any, components, args)
        }
    }
}
trait Signaling: Any {
    fn disconnect(&mut self, method: &FunctionId) -> Result<(), ()>;
}
impl<Args: Any> Signaling for SignalBox<Args> {
    fn disconnect(&mut self, method: &FunctionId) -> Result<(), ()> {
        let Some(i) = self.connections.iter().position(|c| c.id == *method) else {
            return Err(());
        };
        self.connections.remove(i);
        Ok(())
    }
}

type Components = HashMap<ComponentId, ComponentBox>;
type Methods = HashMap<FunctionId, MethodBox>;
#[derive(Default)]
pub struct Signals {
    inner: HashMap<SignalId, Box<dyn Signaling>>,
}
impl Signals {
    fn connect<Args: 'static>(&mut self, signal: &SignalId, method: FunctionId, call: MethodCall<Args> ) -> Result<(), ()> {
        let Some(signal) = self.inner.get_mut(signal) else {
            return Err(());
        };
        let Some(signal) = (signal.as_mut() as &mut dyn Any).downcast_mut::<SignalBox<Args>>()
        else {
            return Err(());
        };
        signal.connect(method, call);
        Ok(())
    }
    fn disconnect(&mut self, signal: &SignalId, method: &FunctionId) -> Result<(), ()> {
        let Some(signal) = self.inner.get_mut(signal) else {
            return Err(());
        };
        signal.disconnect(method);
        Ok(())
    }
    fn call<Args: 'static>(&mut self, signal: &SignalId, components: &mut Components, methods: &mut Methods, args: Args) -> Result<(), ()> {
        let Some(signal) = self.inner.get(signal) else {
            return Err(());
        };
        let Some(signal) = (signal.as_mut() as &mut dyn Any).downcast_mut::<SignalBox<Args>>()
        else {
            return Err(());
        };
        signal.call(components, methods, args);
    }
}

#[derive(Default)]
pub struct Group {
    len: usize,
    components: Components,
    methods: Methods,
    signals: Signals,
}

impl Group {
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn add_component<C: Component, N: ComponentRetrieve, D: ComponentRetrieve>(
        &mut self,
        new: Method<N, Range<usize>>,
        delete: Method<D, Range<usize>>,
    ) {
        let new_id = new.id;
        self.connect(new, &NEW);
        self.connect(delete, &DELETE);
        self.components.insert(C::ID, Box::new(C::Container::new()));
        self.call::<N, Range<usize>>(&new_id, 0..self.len);
    }
    fn connect<C: ComponentRetrieve + 'static, Args: Clone + 'static>(
        &mut self,
        method: Method<C, Args>,
        signal: &SignalId,
    ) -> Result<(), ()> {
        for comp in C::IDS {
            if !self.components.contains_key(comp) {
                return Err(());
            }
        }
        self.signals.connect(signal, method.id, Method::<C, Args>::call);
        self.methods.insert(method.id, MethodBox::new(method));
        Ok(())
    }
    fn disconnect(&mut self, signal: &SignalId, method: &FunctionId) {
        self.signals.disconnect(signal, method);
    }
    pub fn get_components<C: ComponentRetrieve>(
        &self,
    ) -> Option<<C::Containers as Container>::Ref<'_>> {
        C::retrieve(&self.components)
    }
    pub fn get_components_mut<C: ComponentRetrieve>(
        &mut self,
    ) -> Option<<C::Containers as Container>::Mut<'_>> {
        C::retrieve_mut(&mut self.components)
    }
    pub fn remove_component(&mut self, id: &ComponentId) {
        for (_, method) in self.methods.extract_if(|_, m| m.components.contains(id)) {
            for signal in method.signals.iter() {
                self.signals.disconnect(signal, &method.id);
            }
            drop(method);
        }
        self.components.remove(id);
    }
    pub fn call<C: ComponentRetrieve + 'static, Args: 'static>(
        &mut self,
        id: &FunctionId,
        args: Args,
    ) {
        let Some(method) = self.methods.get_mut(id) else {
            return;
        };
        let Some(method) = method.any.downcast_mut::<Method<C, Args>>() else {
            return;
        };
        let Some(components) = C::retrieve_mut(&mut self.components) else {
            return;
        };
        (method.f)(components, args)
    }
    pub fn signal<Args: Clone + 'static>(&mut self, signal: &SignalId, args: Args) {
        let Some(signal) = self.signals.get(signal) else {
            return;
        };
        let Some(signal) = signal.downcast_ref::<SignalBox<Args>>() else {
            return;
        };
        for connection in signal.connections.iter() {
            let Some(method) = self.methods.get_mut(&connection.id) else {
                continue;
            };
            (connection.f)(method, &mut self.components, args.clone())
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

pub trait ComponentRetrieve: 'static {
    type Containers: Container;
    const IDS: &'static [ComponentId];

    fn retrieve(components: &Components) -> Option<<Self::Containers as Container>::Ref<'_>>;
    fn retrieve_mut(
        components: &mut Components,
    ) -> Option<<Self::Containers as Container>::Mut<'_>>;
}

impl ComponentRetrieve for () {
    type Containers = ();
    const IDS: &'static [ComponentId] = &[];
    fn retrieve(_: &Components) -> Option<<Self::Containers as Container>::Ref<'_>> {
        Some(())
    }
    fn retrieve_mut(_: &mut Components) -> Option<<Self::Containers as Container>::Mut<'_>> {
        Some(())
    }
}
impl<C: Component> ComponentRetrieve for C {
    type Containers = C::Container;
    const IDS: &'static [ComponentId] = &[C::ID];

    fn retrieve(components: &Components) -> Option<<Self::Containers as Container>::Ref<'_>> {
        Some(
            components
                .get(&C::ID)?
                .downcast_ref::<C::Container>()?
                .container_ref(),
        )
    }
    fn retrieve_mut(
        components: &mut Components,
    ) -> Option<<Self::Containers as Container>::Mut<'_>> {
        Some(
            components
                .get_mut(&C::ID)?
                .downcast_mut::<C::Container>()?
                .container_mut(),
        )
    }
}

impl Container for () {
    type Ref<'a> = ();
    type Mut<'a> = ();

    fn new() -> Self {}
    fn container_ref(&self) -> Self::Ref<'_> {}
    fn container_mut(&mut self) -> Self::Mut<'_> {}
}
impl<T: 'static> Container for Vec<T> {
    type Ref<'a> = &'a [T];
    type Mut<'a> = &'a mut [T];

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
impl<T: 'static> Container for IndexSet<T> {
    type Ref<'a> = &'a IndexSet<T>; // TODO: prevent structural modifications
    type Mut<'a> = &'a mut IndexSet<T>;

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
impl<T: 'static> Container for OneOrMany<T> {
    type Ref<'a> = &'a [T];
    type Mut<'a> = &'a mut [T];

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
impl Container for BitVec {
    type Ref<'a> = &'a BitSlice;
    type Mut<'a> = &'a mut BitSlice;

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

macro_rules! container_tuples {
	($($T:ident),*) => {
#[allow(non_snake_case)]
impl<$($T: Container),*> Container for ($($T),*) {
	type Ref<'a> = ($($T::Ref<'a>),*);
	type Mut<'a> = ($($T::Mut<'a>),*);

	fn new() -> Self {
		panic!("Component tuples are invalid containers.")
	}
	fn container_ref(&self) -> Self::Ref<'_> {
		let ($($T),*) = self;
		($($T.container_ref()),*)
	}
	fn container_mut(&mut self) -> Self::Mut<'_> {
		let ($($T),*) = self;
		($($T.container_mut()),*)
	}
}
#[allow(non_snake_case)]
impl<$($T: Component),*> ComponentRetrieve for ($($T),*) {
	type Containers = ($($T::Container),*);
	const IDS: &'static [ComponentId] = &[$($T::ID),*];

	fn retrieve(components: &Components) -> Option<<Self::Containers as Container>::Ref<'_>> {
		$(
			let $T = components
			.get(&$T::ID)?
			.downcast_ref::<$T::Container>()?
			.container_ref();
		)*
		Some(($($T),*))
	}
	fn retrieve_mut(
		components: &mut Components,
	) -> Option<<Self::Containers as Container>::Mut<'_>> {
		let [$($T),*] = components.get_disjoint_mut([$(&$T::ID),*]);
		$(
			let $T = $T?.downcast_mut::<$T::Container>()?.container_mut();
		)*
		Some(($($T),*))
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
