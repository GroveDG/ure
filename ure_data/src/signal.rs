use std::collections::HashMap;

use indexmap::IndexMap;
use nohash_hasher::BuildNoHashHasher;

use crate::{group::Group, method::Method};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(u64);
impl nohash_hasher::IsEnabled for ConnectionId {}
impl ConnectionId {
	pub const fn new(name: &str) -> Self {
		Self(const_fnv1a_hash::fnv1a_hash_str_64(name))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalId(u64);
impl nohash_hasher::IsEnabled for SignalId {}
impl SignalId {
	pub const fn new(name: &str) -> Self {
		Self(const_fnv1a_hash::fnv1a_hash_str_64(name))
	}
}

#[derive(Debug, Default)]
pub struct Signal<Args: Clone> {
	methods: IndexMap<ConnectionId, Method<Args>>,
}
impl<Args: Clone> Signal<Args> {
	pub fn call(&self, group: &Group, args: Args) {
		for method in self.methods.values() {
			method.call(group, args.clone());
		}
	}
}

trait SignalTrait {
	fn disconnect(&mut self, connection_id: ConnectionId);
}

#[derive(Default)]
pub struct Signals {
	inner: HashMap<SignalId, Box<dyn SignalTrait>, BuildNoHashHasher<u64>>,
}