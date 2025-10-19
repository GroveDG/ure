use std::{
	any::Any,
	collections::HashMap,
	marker::PhantomData,
	ops::{Deref, DerefMut, Range},
};

pub use bitvec::{slice::BitSlice, vec::BitVec};
use indexmap::IndexMap;
pub use indexmap::IndexSet;
use multimap::MultiMap;
pub use one_or_many::OneOrMany;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(u64);
impl ComponentId {
	const fn new(ident: &str) -> Self {
		Self(const_fnv1a_hash::fnv1a_hash_str_64(ident))
	}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MethodId(u64);
impl MethodId {
	const fn new(ident: &str) -> Self {
		Self(const_fnv1a_hash::fnv1a_hash_str_64(ident))
	}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalId(u64);
impl SignalId {
	const fn new(ident: &str) -> Self {
		Self(const_fnv1a_hash::fnv1a_hash_str_64(ident))
	}
}

pub struct Signal<Args> {
	id: SignalId,
	_marker: PhantomData<Args>,
}
impl<Args> Signal<Args> {
	pub const fn new(ident: &'static str) -> Self {
		Self {
			id: SignalId::new(ident),
			_marker: PhantomData,
		}
	}
}

pub const NEW: Signal<Range<usize>> = Signal::new("New");
pub const DELETE: Signal<Range<usize>> = Signal::new("Delete");

pub trait Container: Any {
	type Ref<'a>;
	type Mut<'a>;
	type AsRef<'a>;
	type AsMut<'a>;

	fn new() -> Self;
	fn container_ref(&self) -> Self::Ref<'_>;
	fn container_mut(&mut self) -> Self::Mut<'_>;
}

pub trait Component: 'static {
	const ID: ComponentId = ComponentId::new(Self::IDENT);
	const IDENT: &'static str;

	type Container: Container;
}

#[macro_export]
macro_rules! component {
	($name:ident : $container:ty) => {
		struct $name;
		impl $crate::Component for $name {
			const IDENT: &'static str = stringify!($name);
			type Container = $container;
		}
	};
}

pub type Method<Args = (), Return = ()> = fn(&mut Components, Args) -> Return;
pub type Get<'a, C> = <<C as ComponentRetrieve>::Containers as Container>::Mut<'a>;
pub type Extract<'a, C> = <<C as ComponentRetrieve>::Containers as Container>::AsMut<'a>;

#[macro_export]
macro_rules! method {
	(CAPTURE, $i:ident, $t:ty) => {
		$i
	};
	($func:ident $(, extract $extract:ty)? $(, get $get:ty)? $(, args $args:ty)? $(,)?) => {
		|components, args| {
			$( let mut extract = <$extract as $crate::ComponentRetrieve>::extract(components).unwrap(); )?;
			$( let get = <$get as $crate::ComponentRetrieve>::retrieve_mut(components).unwrap(); )?
			let ret = ($func)(
				$(&mut $crate::method!(CAPTURE, extract, $extract), )?
				$($crate::method!(CAPTURE, get, $get), )?
				$($crate::method!(CAPTURE, args, $args), )?
			);
			$( <$extract as $crate::ComponentRetrieve>::insert(components, extract); )?
			ret
		}
	};
}

#[macro_export]
macro_rules! mut_components {
	(
		$fn_name:ident $components:ident
		($($component_name:ident : $component_type:ty,)*)
		($($arg_name:ident : $arg_type:ty),*)
	) => {
		let ($($component_name),*) = <($($component_type,)*)>::retrieve_mut($components).unwrap();
		($fn_name)($($component_name,)* $arg_name)
	};
}

#[derive(Debug)]
struct MethodFn<Args> {
	func: Method<Args>,
	id: MethodId,
	dependencies: &'static [ComponentId],
}
impl<Args> MethodFn<Args> {
	pub const fn new(
		func: Method<Args>,
		ident: &'static str,
		dependencies: &'static [ComponentId],
	) -> Self {
		MethodFn {
			func,
			id: MethodId::new(ident),
			dependencies,
		}
	}
}
impl<Args> Clone for MethodFn<Args> {
	fn clone(&self) -> Self {
		Self {
			func: self.func,
			id: self.id,
			dependencies: self.dependencies,
		}
	}
}
impl<Args> Copy for MethodFn<Args> {}

struct SignalBoard<Args: Clone> {
	methods: IndexMap<MethodId, Method<Args>>,
}
impl<Args: Clone> SignalBoard<Args> {
	fn insert(&mut self, method: MethodFn<Args>) {
		self.methods.insert(method.id, method.func);
	}
	fn call(&self, components: &mut Components, args: Args) -> Result<(), ()> {
		for method in self.methods.values().copied() {
			(method)(components, args.clone());
		}
		Ok(())
	}
}
trait Signaling {
	fn remove(&mut self, id: &MethodId);
}
impl<Args: Clone> Signaling for SignalBoard<Args> {
	fn remove(&mut self, id: &MethodId) {
		self.methods.shift_remove(id);
	}
}

impl dyn Signaling {
	unsafe fn downcast_signal<Args: Clone>(&self) -> &SignalBoard<Args> {
		unsafe {
			std::mem::transmute::<_, (*const SignalBoard<Args>, *const ())>(std::ptr::from_ref(
				self,
			))
			.0
			.as_ref()
			.unwrap()
		}
	}
	unsafe fn downcast_signal_mut<Args: Clone>(&mut self) -> &mut SignalBoard<Args> {
		unsafe {
			std::mem::transmute::<_, (*mut SignalBoard<Args>, *const ())>(std::ptr::from_mut(self))
				.0
				.as_mut()
				.unwrap()
		}
	}
}

type ComponentBox = Box<dyn Any>;
type SignalBox = Box<dyn Signaling>;

pub type Components = HashMap<ComponentId, ComponentBox>;
type Signals = HashMap<SignalId, SignalBox>;
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
	pub fn add_component<C: Component>(
		&mut self,
		new: MethodFn<Range<usize>>,
		delete: MethodFn<Range<usize>>,
	) -> Result<(), ()> {
		self.components.insert(C::ID, Box::new(C::Container::new()));
		if self.connect(NEW, new).is_ok() {
			if self.connect(DELETE, delete).is_ok() {
				(new.func)(&mut self.components, 0..self.len);
				return Ok(());
			};
			self.disconnect(&NEW.id, &new.id).unwrap();
		};
		self.components.remove(&C::ID);
		Err(())
	}
	fn connect<Args: Clone>(
		&mut self,
		signal: Signal<Args>,
		method: MethodFn<Args>,
	) -> Result<(), ()> {
		for component in method.dependencies {
			if !self.components.contains_key(component) {
				return Err(());
			}
		}
		let Some(signalboard) = self.get_signal_mut(&signal) else {
			return Err(());
		};
		signalboard.insert(method);
		for component in method.dependencies.iter().copied() {
			self.dependency.insert(component, method.id);
		}
		self.connection.insert(method.id, signal.id);
		Ok(())
	}
	fn disconnect(&mut self, signal: &SignalId, method: &MethodId) -> Result<(), ()> {
		let Some(signal) = self.signals.get_mut(signal) else {
			return Err(());
		};
		signal.remove(method);
		Ok(())
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
	fn get_signal_mut<Args: Clone>(
		&mut self,
		signal: &Signal<Args>,
	) -> Option<&mut SignalBoard<Args>> {
		let signal = self.signals.get_mut(&signal.id)?;
		let signal: &mut SignalBoard<Args> = unsafe { signal.downcast_signal_mut::<Args>() };
		Some(signal)
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
				self.disconnect(&signal, &method).unwrap();
			}
		}
		self.components.remove(id);
		Ok(())
	}
	pub fn signal<Args: Clone + 'static>(
		&mut self,
		signal: &Signal<Args>,
		args: Args,
	) -> Result<(), ()> {
		let Some(signal) = self.signals.get_mut(&signal.id) else {
			return Err(());
		};
		let signal: &mut SignalBoard<Args> = unsafe { signal.downcast_signal_mut::<Args>() };
		signal.call(&mut self.components, args.clone()).unwrap();
		Ok(())
	}
	pub fn new(&mut self, num: usize) {
		self.signal(&NEW, self.len..self.len + num).unwrap();
		self.len += num;
	}
	pub fn delete(&mut self, range: Range<usize>) {
		self.signal(&DELETE, range.clone()).unwrap();
		self.len -= range.len();
	}
}

pub trait ComponentRetrieve {
	type Containers: Container;
	type BoxedContainers;
	const IDS: &'static [ComponentId];

	fn retrieve(components: &Components) -> Option<<Self::Containers as Container>::Ref<'_>>;
	fn retrieve_mut(
		components: &mut Components,
	) -> Option<<Self::Containers as Container>::Mut<'_>>;
	fn extract(components: &mut Components) -> Option<Self::BoxedContainers>;
	fn insert(components: &mut Components, containers: Self::BoxedContainers);
}

impl ComponentRetrieve for () {
	type Containers = ();
	type BoxedContainers = ();
	const IDS: &'static [ComponentId] = &[];
	fn retrieve(_: &Components) -> Option<<Self::Containers as Container>::Ref<'_>> {
		Some(())
	}
	fn retrieve_mut(_: &mut Components) -> Option<<Self::Containers as Container>::Mut<'_>> {
		Some(())
	}
	fn extract(_: &mut Components) -> Option<Self::BoxedContainers> {
		Some(())
	}
	fn insert(_: &mut Components, _: Self::BoxedContainers) {}
}
impl<C: Component> ComponentRetrieve for C {
	type Containers = C::Container;
	type BoxedContainers = Box<Self::Containers>;
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
	fn extract(components: &mut Components) -> Option<Box<Self::Containers>> {
		components.remove(&C::ID)?.downcast().ok()
	}
	
	fn insert(components: &mut Components, containers: Box<Self::Containers>) {
		components.insert(C::ID, containers);
	}
}

impl Container for () {
	type Ref<'a> = ();
	type Mut<'a> = ();
	type AsRef<'a> = ();
	type AsMut<'a> = ();

	fn new() -> Self {}
	fn container_ref(&self) -> Self::Ref<'_> {}
	fn container_mut(&mut self) -> Self::Mut<'_> {}
}
pub struct One<T: 'static>(pub Option<T>);
impl<T: 'static> Container for One<T> {
	type Ref<'a> = &'a T;
	type Mut<'a> = &'a mut T;
	type AsRef<'a> = &'a One<T>;
	type AsMut<'a> = &'a mut One<T>;

	fn new() -> Self {
		One(None)
	}
	fn container_ref(&self) -> Self::Ref<'_> {
		self.0.as_ref().unwrap()
	}
	fn container_mut(&mut self) -> Self::Mut<'_> {
		self.0.as_mut().unwrap()
	}
}
impl<T: 'static> Container for Option<T> {
	type Ref<'a> = Option<&'a T>;
	type Mut<'a> = Option<&'a mut T>;
	type AsRef<'a> = &'a Option<T>;
	type AsMut<'a> = &'a mut Option<T>;

	fn new() -> Self {
		None
	}
	fn container_ref(&self) -> Self::Ref<'_> {
		self.as_ref()
	}
	fn container_mut(&mut self) -> Self::Mut<'_> {
		self.as_mut()
	}
}
impl<T: 'static> Container for Vec<T> {
	type Ref<'a> = &'a [T];
	type Mut<'a> = &'a mut [T];
	type AsRef<'a> = &'a Vec<T>;
	type AsMut<'a> = &'a mut Vec<T>;

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
	type AsRef<'a> = &'a IndexSet<T>;
	type AsMut<'a> = &'a mut IndexSet<T>;

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
	type AsRef<'a> = &'a OneOrMany<T>;
	type AsMut<'a> = &'a mut OneOrMany<T>;

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
	type AsRef<'a> = &'a BitVec;
	type AsMut<'a> = &'a mut BitVec;

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
	type AsRef<'a> = ($($T::AsRef<'a>),*);
	type AsMut<'a> = ($($T::AsMut<'a>),*);

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
	type BoxedContainers = ($(Box<$T::Container>),*);
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
	fn extract(
		components: &mut Components,
	) -> Option<Self::BoxedContainers> {
		$( let $T = <$T>::extract(components); )*
		if $(($T).is_some())&&* {
			Some( ($(($T).unwrap()),*) )
		} else {
			$(
			if let Some($T) = $T {
				<$T>::insert(components, $T);
			}
			)*
			None
		}
	}
	fn insert(
		components: &mut Components,
		containers: Self::BoxedContainers
	) {
		let ($($T),*) = containers;
		$(
		<$T as ComponentRetrieve>::insert(components, $T);
		)*
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
