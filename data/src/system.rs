use crate::group::{Data, Group};

pub struct System<Key: slotmap::Key> {
	keys: Vec<Key>,
}
impl<Key: slotmap::Key> System<Key> {
	pub fn run(&mut self, data: &mut Data<Key>) {
		let mut remove = Vec::new();
		for (i, key) in self.keys.iter().copied().enumerate() {
			let Some(group) = data.get_mut(key) else {
				remove.push(i);
				continue;
			};
			let group = group.get_mut();
		}
		remove.reverse();
		for i in remove {
			self.keys.remove(i);
		}
	}
	pub fn add(&mut self, key: Key, data: &mut Data<Key>) {
		self.keys.push(key);
	}
}

struct SystemMembers<Key: slotmap::Key, S> {
	keys: Vec<Key>,
	system: S,
}

struct SystemMember<S> {
	member: Group,
	system: S,
}

pub enum SystemError {
	MissingComponent(u64),
}

trait SystemTrait<Key> {
	fn add(group: &mut Group);
}
trait SystemTraitRef<'a> {
	fn get(group: &'a Group) -> Result<Self, SystemError>
	where
		Self: 'a + Sized;
}
impl<'a, S: SystemTraitRef<'a>> SystemTraitMut<'a> for S {
	fn get_mut(group: &'a mut Group) -> Result<Self, SystemError>
	where
		Self: 'a + Sized,
	{
		Self::get(group)
	}
}
trait SystemTraitMut<'a> {
	fn get_mut(group: &'a mut Group) -> Result<Self, SystemError>
	where
		Self: 'a + Sized;
}

struct ExampleSystem<'a> {
	indicies: &'a mut Vec<usize>,
	other: &'a [usize],
}
impl<'a> SystemTraitMut<'a> for ExampleSystem<'a> {
	fn get_mut(group: &mut Group) -> Result<Self, SystemError> {
		Ok(Self {
			indicies: todo!(),
			other: todo!(),
		})
	}
}
