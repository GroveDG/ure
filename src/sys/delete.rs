use std::collections::HashSet;

use super::UID;

/// Starting at the beginning of the previous frame
/// we delete all UIDs queued. This prevents skipping
/// deletes when systems are reordered.
///
/// Don't forget, you can (and should!) immediately
/// delete an entity's components in any system you
/// currently have mutable access to when queueing
/// a delete.
///
/// Visualization
/// -------------
///
/// `|====================|====================|`
///
/// `(====================|==========)=========|`
///
/// versus
///
/// `|==========(=========|==========)=========|`
///
#[derive(Debug, Default)]
pub struct DeleteQueue {
    queue: [HashSet<UID>; 2],
}

impl DeleteQueue {
    fn queue(&mut self, uid: UID) {
        self.queue[1].insert(uid);
    }
    pub fn delete(&mut self, system: &mut dyn Delete, uid: UID) {
        system.delete(&uid);
        self.queue(uid);
    }
    pub fn start_frame(&mut self) {
        self.queue[0].clear();
        self.queue.swap(0, 1);
    }
    pub fn apply(&self, system: &mut dyn Delete) {
        for uid in self.iter() {
            system.delete(uid);
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &UID> {
        self.queue[0].iter().chain(self.queue[1].iter())
    }
}

pub trait Delete {
    fn delete(&mut self, uid: &UID);
}
