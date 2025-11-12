use std::{convert::Infallible, error::Error, fmt::Display, marker::PhantomData};

use indexmap::IndexSet;

use crate::{
	components::{Component, ComponentDependency, ComponentGroup, ComponentId, MissingDependency},
	group::Group,
};

pub struct Glob<GroupKey: slotmap::Key, ItemKey, C: Component<Container = IndexSet<ItemKey>>> {
	items: Vec<GlobItem<GroupKey, ItemKey>>,
	_marker: PhantomData<C>,
}
pub struct GlobItem<GroupKey: slotmap::Key, ItemKey> {
	group: GroupKey,
	indices: Option<Vec<ItemKey>>,
}
#[derive(Clone, Copy)]
pub struct GlobItemRef<'a> {
	group: &'a Group,
	indices: Option<&'a [usize]>,
}
impl<'a> GlobItemRef<'a> {
	pub fn from_group(group: &'a Group) -> Self {
		Self {
			group,
			indices: None,
		}
	}
}

impl<GroupKey: slotmap::Key, ItemKey, C> Glob<GroupKey, ItemKey, C>
where
	C: Component<Container = IndexSet<ItemKey>>,
{
	pub fn new() -> Self {
		Self {
			items: Vec::new(),
			_marker: PhantomData,
		}
	}
	pub fn index<'a>(&self, group: &'a Group, i: usize) -> GlobItemRef<'a> {
		todo!()
	}
}

pub struct Len(pub usize);
impl ComponentDependency for Len {
	fn dependencies() -> Vec<ComponentId> {
		Vec::new()
	}
}
impl TryFrom<GlobItemRef<'_>> for Len {
	type Error = Infallible;

	fn try_from(value: GlobItemRef<'_>) -> Result<Self, Self::Error> {
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
pub struct Indices<'a>(pub &'a [usize]);
impl<'a> ComponentDependency for Indices<'a> {
	fn dependencies() -> Vec<ComponentId> {
		Vec::new()
	}
}
impl<'a> TryFrom<GlobItemRef<'a>> for Indices<'a> {
	type Error = MissingIndices;

	fn try_from(value: GlobItemRef<'a>) -> Result<Self, Self::Error> {
		Ok(Self(value.indices.ok_or(MissingIndices)?))
	}
}

pub struct ContRef<'a, C: ComponentGroup>(pub C::ContainersRef<'a>);
impl<'a, C: ComponentGroup> ComponentDependency for ContRef<'a, C> {
	fn dependencies() -> Vec<ComponentId> {
		C::IDS.to_vec()
	}
}
impl<'a, C: ComponentGroup> TryFrom<GlobItemRef<'a>> for ContRef<'a, C> {
	type Error = MissingDependency;

	fn try_from(value: GlobItemRef<'a>) -> Result<Self, Self::Error> {
		C::borrow_containers(value.group).map(|c| Self(c))
	}
}

pub struct ContMut<'a, C: ComponentGroup>(pub C::ContainersRefMut<'a>);
impl<'a, C: ComponentGroup> ComponentDependency for ContMut<'a, C> {
	fn dependencies() -> Vec<ComponentId> {
		C::IDS.to_vec()
	}
}
impl<'a, C: ComponentGroup> TryFrom<GlobItemRef<'a>> for ContMut<'a, C> {
	type Error = MissingDependency;

	fn try_from(value: GlobItemRef<'a>) -> Result<Self, Self::Error> {
		C::borrow_containers_mut(value.group).map(|c| Self(c))
	}
}

pub struct CompRef<'a, C: ComponentGroup>(pub C::ComponentsRef<'a>);
impl<'a, C: ComponentGroup> ComponentDependency for CompRef<'a, C> {
	fn dependencies() -> Vec<ComponentId> {
		C::IDS.to_vec()
	}
}
impl<'a, C: ComponentGroup> TryFrom<GlobItemRef<'a>> for CompRef<'a, C> {
	type Error = MissingDependency;

	fn try_from(value: GlobItemRef<'a>) -> Result<Self, Self::Error> {
		C::borrow_components(value.group).map(|c| Self(c))
	}
}

pub struct CompMut<'a, C: ComponentGroup>(pub C::ComponentsRefMut<'a>);
impl<'a, C: ComponentGroup> ComponentDependency for CompMut<'a, C> {
	fn dependencies() -> Vec<ComponentId> {
		C::IDS.to_vec()
	}
}
impl<'a, C: ComponentGroup> TryFrom<GlobItemRef<'a>> for CompMut<'a, C> {
	type Error = MissingDependency;

	fn try_from(value: GlobItemRef<'a>) -> Result<Self, Self::Error> {
		C::borrow_components_mut(value.group).map(|c| Self(c))
	}
}
