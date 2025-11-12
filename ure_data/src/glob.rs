use std::{
	cell::{Ref, RefMut},
	collections::HashMap,
	convert::Infallible,
	error::Error,
	fmt::Display,
	hash::Hash,
	marker::PhantomData,
};

use indexmap::IndexSet;

use crate::{
	components::{Component, ComponentDependency, ComponentGroup, ComponentId, MissingDependency},
	group::{Data, Group},
	method::{MethodTrait, TryFromGlob},
};

pub struct Glob<GroupKey: slotmap::Key, ItemKey, C: Component<Container = IndexSet<ItemKey>>> {
	items: HashMap<GroupKey, Option<Vec<ItemKey>>>,
	_marker: PhantomData<C>,
}
pub struct Globule<GroupKey: slotmap::Key, ItemKey> {
	group: GroupKey,
	indices: Option<Vec<ItemKey>>,
}
pub struct GlobuleIndexed<'a> {
	group: Ref<'a, Group>,
	indices: Option<Vec<usize>>,
}
impl<'a> GlobuleIndexed<'a> {
	pub fn as_ref<'b>(&'a self) -> GlobuleRef<'a, 'b>
	where
		'a: 'b,
	{
		GlobuleRef {
			group: &self.group,
			indices: self.indices.as_ref().map(|i| i.as_slice()),
		}
	}
	pub fn group(&'a self) -> Ref<'a, Group> {
		Ref::clone(&self.group)
	}
}
pub struct GlobuleIndexedMut<'a> {
	group: RefMut<'a, Group>,
	indices: Option<Vec<usize>>,
}
impl<'a> GlobuleIndexedMut<'a> {
	pub fn as_ref<'b>(&'a self) -> GlobuleRef<'a, 'b>
	where
		'a: 'b,
	{
		GlobuleRef {
			group: &self.group,
			indices: self.indices.as_ref().map(|i| i.as_slice()),
		}
	}
	pub fn as_mut<'b, 'c>(&'a mut self) -> GlobuleMut<'a, 'b, 'c>
	where
		'a: 'b,
	{
		GlobuleMut {
			group: &mut self.group,
			indices: self.indices.as_ref().map(|i| i.as_slice()),
		}
	}
	pub fn group(&'a mut self) -> &'a mut RefMut<'a, Group> {
		&mut self.group
	}
}
#[derive(Clone, Copy)]
pub struct GlobuleRef<'a, 'b> {
	group: &'a Group,
	indices: Option<&'b [usize]>,
}
impl<'a, 'b> GlobuleRef<'a, 'b> {
	pub fn from_group(group: &'a Group) -> Self {
		Self {
			group,
			indices: None,
		}
	}
	pub fn call_method<T: TryFromGlob<'a, 'b>, Args, Return>(
		self,
		method: impl MethodTrait<T, Args, Return>,
		args: Args,
	) -> Result<Return, Box<dyn Error>> {
		(method).call_method(self, args)
	}
	pub fn group(&self) -> &'a Group {
		self.group
	}
}
pub struct GlobuleMut<'a, 'b, 'c> {
	group: &'c mut RefMut<'a, Group>,
	indices: Option<&'b [usize]>,
}
impl<'a, 'b, 'c> GlobuleMut<'a, 'b, 'c> {
	pub fn from_group(group: &'c mut RefMut<'a, Group>) -> Self {
		Self {
			group,
			indices: None,
		}
	}
	pub fn as_ref(&'a self) -> GlobuleRef<'a, 'b> {
		GlobuleRef {
			group: &self.group,
			indices: self.indices,
		}
	}
	pub fn call_method<T: TryFromGlob<'a, 'b>, Args, Return>(
		&'a self,
		method: impl MethodTrait<T, Args, Return>,
		args: Args,
	) -> Result<Return, Box<dyn Error>> {
		self.as_ref().call_method(method, args)
	}
	pub fn group(&mut self) -> &mut RefMut<'a, Group> {
		&mut self.group
	}
}

impl<GroupKey: slotmap::Key, ItemKey: Hash + Eq + 'static, C> Glob<GroupKey, ItemKey, C>
where
	C: Component<Container = IndexSet<ItemKey>>,
{
	pub fn new() -> Self {
		Self {
			items: Default::default(),
			_marker: PhantomData,
		}
	}
	pub fn index(&self, group: &Group, group_key: &GroupKey) -> Option<Option<Vec<usize>>> {
		let Some(item) = self.items.get(group_key)?.as_ref() else {
			return Some(None);
		};
		let component = group.borrow_component::<C>()?;
		Some(item.iter().map(|key| component.get_index_of(key)).collect())
	}
	pub fn get<'a>(
		&self,
		data: &'a Data<GroupKey>,
		group_key: GroupKey,
	) -> Option<GlobuleIndexed<'a>> {
		let group = data.get(group_key)?.borrow();
		Some(GlobuleIndexed {
			indices: self.index(&group, &group_key)?,
			group,
		})
	}
	pub fn get_mut<'a>(
		&self,
		data: &'a Data<GroupKey>,
		group_key: GroupKey,
	) -> Option<GlobuleIndexedMut<'a>> {
		let group = data.get(group_key)?.borrow_mut();
		Some(GlobuleIndexedMut {
			indices: self.index(&group, &group_key)?,
			group,
		})
	}
	pub fn add_group(&mut self, group_key: GroupKey) {
		self.items.insert(group_key, None);
	}
	pub fn iter<'a, 'b>(
		&'a self,
		data: &'b Data<GroupKey>,
	) -> GlobIter<'a, 'b, GroupKey, ItemKey, C> {
		GlobIter {
			glob: self.items.iter(),
			data,
			_marker: PhantomData,
		}
	}
	pub fn iter_mut<'a, 'b>(
		&'a self,
		data: &'b Data<GroupKey>,
	) -> GlobIterMut<'a, 'b, GroupKey, ItemKey, C> {
		GlobIterMut {
			glob: self.items.iter(),
			data,
			_marker: PhantomData,
		}
	}
}

pub struct GlobIter<'a, 'b, GroupKey: slotmap::Key, ItemKey, C> {
	glob: std::collections::hash_map::Iter<'a, GroupKey, Option<Vec<ItemKey>>>,
	data: &'b Data<GroupKey>,
	_marker: PhantomData<C>,
}
impl<'a, 'b: 'a, GroupKey: slotmap::Key, ItemKey: Hash + Eq + 'static, C: Component> Iterator
	for GlobIter<'a, 'b, GroupKey, ItemKey, C>
where
	C: Component<Container = IndexSet<ItemKey>>,
{
	type Item = GlobuleIndexed<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		let (key, value) = self.glob.next()?;
		let group = self.data.get(*key).unwrap().borrow();
		let Some(value) = value.as_ref() else {
			return Some(GlobuleIndexed {
				group,
				indices: None,
			});
		};
		let indices = {
			let component = group.borrow_component::<C>().unwrap();
			value
				.iter()
				.map(|key| component.get_index_of(key))
				.collect()
		};
		Some(GlobuleIndexed { group, indices })
	}
}

pub struct GlobIterMut<'a, 'b, GroupKey: slotmap::Key, ItemKey, C> {
	glob: std::collections::hash_map::Iter<'a, GroupKey, Option<Vec<ItemKey>>>,
	data: &'b Data<GroupKey>,
	_marker: PhantomData<C>,
}
impl<'a, 'b: 'a, GroupKey: slotmap::Key, ItemKey: Hash + Eq + 'static, C: Component> Iterator
	for GlobIterMut<'a, 'b, GroupKey, ItemKey, C>
where
	C: Component<Container = IndexSet<ItemKey>>,
{
	type Item = GlobuleIndexedMut<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		let (key, value) = self.glob.next()?;
		let group = self.data.get(*key).unwrap().borrow_mut();
		let Some(value) = value.as_ref() else {
			return Some(GlobuleIndexedMut {
				group,
				indices: None,
			});
		};
		let indices = {
			let component = group.borrow_component::<C>().unwrap();
			value
				.iter()
				.map(|key| component.get_index_of(key))
				.collect()
		};
		Some(GlobuleIndexedMut { group, indices })
	}
}

pub struct Len(pub usize);
impl ComponentDependency for Len {
	fn dependencies() -> Vec<ComponentId> {
		Vec::new()
	}
}
impl TryFrom<GlobuleRef<'_, '_>> for Len {
	type Error = Infallible;

	fn try_from(value: GlobuleRef<'_, '_>) -> Result<Self, Self::Error> {
		Ok(Self(value.group.len()))
	}
}

#[derive(Debug, Clone, Copy)]
pub struct MissingIndices;
impl Display for MissingIndices {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Glob item does not contain indices.")
	}
}
impl Error for MissingIndices {}
pub struct Indices<'b>(pub &'b [usize]);
impl<'a> ComponentDependency for Indices<'a> {
	fn dependencies() -> Vec<ComponentId> {
		Vec::new()
	}
}
impl<'a, 'b> TryFrom<GlobuleRef<'a, 'b>> for Indices<'b> {
	type Error = MissingIndices;

	fn try_from(value: GlobuleRef<'a, 'b>) -> Result<Self, Self::Error> {
		Ok(Self(value.indices.ok_or(MissingIndices)?))
	}
}

pub struct ContRef<'a, C: ComponentGroup>(pub C::ContainersRef<'a>);
impl<'a, C: ComponentGroup> ComponentDependency for ContRef<'a, C> {
	fn dependencies() -> Vec<ComponentId> {
		C::IDS.to_vec()
	}
}
impl<'a, C: ComponentGroup> TryFrom<GlobuleRef<'a, '_>> for ContRef<'a, C> {
	type Error = MissingDependency;

	fn try_from(value: GlobuleRef<'a, '_>) -> Result<Self, Self::Error> {
		C::borrow_containers(value.group).map(|c| Self(c))
	}
}

pub struct ContMut<'a, C: ComponentGroup>(pub C::ContainersRefMut<'a>);
impl<'a, C: ComponentGroup> ComponentDependency for ContMut<'a, C> {
	fn dependencies() -> Vec<ComponentId> {
		C::IDS.to_vec()
	}
}
impl<'a, C: ComponentGroup> TryFrom<GlobuleRef<'a, '_>> for ContMut<'a, C> {
	type Error = MissingDependency;

	fn try_from(value: GlobuleRef<'a, '_>) -> Result<Self, Self::Error> {
		C::borrow_containers_mut(value.group).map(|c| Self(c))
	}
}

pub struct CompRef<'a, C: ComponentGroup>(pub C::ComponentsRef<'a>);
impl<'a, C: ComponentGroup> ComponentDependency for CompRef<'a, C> {
	fn dependencies() -> Vec<ComponentId> {
		C::IDS.to_vec()
	}
}
impl<'a, C: ComponentGroup> TryFrom<GlobuleRef<'a, '_>> for CompRef<'a, C> {
	type Error = MissingDependency;

	fn try_from(value: GlobuleRef<'a, '_>) -> Result<Self, Self::Error> {
		C::borrow_components(value.group).map(|c| Self(c))
	}
}

pub struct CompMut<'a, C: ComponentGroup>(pub C::ComponentsRefMut<'a>);
impl<'a, C: ComponentGroup> ComponentDependency for CompMut<'a, C> {
	fn dependencies() -> Vec<ComponentId> {
		C::IDS.to_vec()
	}
}
impl<'a, C: ComponentGroup> TryFrom<GlobuleRef<'a, '_>> for CompMut<'a, C> {
	type Error = MissingDependency;

	fn try_from(value: GlobuleRef<'a, '_>) -> Result<Self, Self::Error> {
		C::borrow_components_mut(value.group).map(|c| Self(c))
	}
}
