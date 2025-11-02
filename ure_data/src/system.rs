use crate::group::Data;

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
