use std::{collections::HashMap, error::Error, marker::PhantomData};

use nohash_hasher::BuildNoHashHasher;
use slotmap::{SlotMap, new_key_type};

use crate::{glob::GlobuleRef, method::Method};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SignalId<Args>(u64, PhantomData<Args>);
impl<Args> Clone for SignalId<Args> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), PhantomData)
	}
}
impl<Args> Copy for SignalId<Args> {}
impl<Args> nohash_hasher::IsEnabled for SignalId<Args> {}
impl<Args> SignalId<Args> {
	pub const fn new(name: &str) -> Self {
		Self(const_fnv1a_hash::fnv1a_hash_str_64(name), PhantomData)
	}
}

new_key_type! {
	pub struct ConnectionId;
}

#[derive(Default)]
pub struct Signal {
	methods: SlotMap<ConnectionId, Box<Method<()>>>,
}
impl Signal {
	pub unsafe fn call<Args>(&self, glob: GlobuleRef<'_, '_>, mut args: Args) {
		for method in self.methods.values() {
			let f = unsafe { std::mem::transmute::<&Method<()>, &Method<Args>>(method) };
			(f)(glob.clone(), &mut args);
		}
	}
	pub unsafe fn connect(&mut self, method: Box<Method<()>>) -> ConnectionId {
		self.methods.insert(method)
	}
}

#[macro_export]
macro_rules! signal {
	($v:vis $name:ident: $args:ty) => {
$v const $name: $crate::signals::SignalId<$args> = $crate::signals::SignalId::new(stringify!($name));
	};
}

#[derive(Default)]
pub struct Signals {
	inner: HashMap<u64, Signal, BuildNoHashHasher<u64>>,
}
impl Signals {
	pub fn connect<
		Args,
		M: for<'a, 'b, 'c> Fn(GlobuleRef<'a, 'b>, &'c mut Args) -> Result<(), Box<dyn Error>> + 'static,
	>(
		&mut self,
		signal_id: &SignalId<Args>,
		method: M,
	) {
		let method: Box<Method<Args>> = Box::new(method);
		let method: Box<Method<()>> = unsafe { std::mem::transmute(method) };
		let Some(signal) = self.inner.get_mut(&signal_id.0) else {
			let mut signal = Signal::default();
			unsafe { signal.connect(method) };
			self.inner.insert(signal_id.0, signal);
			return;
		};
		unsafe { signal.connect(method) };
	}
	pub fn call<Args>(&self, signal_id: &SignalId<Args>, glob: GlobuleRef<'_, '_>, args: Args) {
		let Some(signal) = self.inner.get(&signal_id.0) else {
			return;
		};
		unsafe { signal.call(glob, args) };
	}
}
