use std::{collections::HashMap, marker::PhantomData, ptr::NonNull};

use super::{Components, UID};

#[derive(Debug, Default)]
pub struct Tree {
    map: HashMap<UID, NonNull<Node>>,
    roots: Vec<NonNull<Node>>,
    first: Option<NonNull<Node>>,
    _data: PhantomData<Box<Node>>,
}
impl Drop for Tree {
    fn drop(&mut self) {
        // See [LinkedList::pop_front_node]
        while let Some(first) = self.first {
            unsafe {
                let mut first: Box<Node> = Box::from_raw(first.as_ptr());
                self.first = first.next.take();
                if let Some(first) = self.first {
                    (*first.as_ptr()).prev = None;
                }
                drop(first);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    uid: UID,
    parent: Option<NonNull<Node>>,
    children: Vec<NonNull<Node>>,
    next: Option<NonNull<Node>>,
    prev: Option<NonNull<Node>>,
}
/*
6
-3
 -1
 -2
-5
 -4
*/
impl Tree {
    pub fn insert(&mut self, uid: UID, parent: Option<&UID>) -> Result<(), ()> {
        let parent = parent.and_then(|p| self.map.get(p).copied());

        let prev = if let Some(parent) = parent {
            unsafe { (*parent.as_ptr()).prev }
        } else {
            self.roots.last().copied()
        };

        let node = Box::leak(Box::new(Node {
            uid,
            parent,
            children: Vec::new(),
            next: parent,
            prev,
        })).into();

        self.map.insert(uid, node);

        unsafe {
            if let Some(parent) = parent {
                (*parent.as_ptr()).children.push(node);
                (*parent.as_ptr()).prev = Some(node);
            }
            if let Some(prev) = prev {
                (*prev.as_ptr()).next = Some(node);
            }
        }

        Ok(())
    }

    pub fn delete(&mut self, node: &UID) {
        let Some(node_ptr) = self.map.remove(&node) else {
            return;
        };
        let mut node: Box<Node> = unsafe { Box::from_raw(node_ptr.as_ptr()) };
        unsafe {
            if let Some(parent) = node.parent.take() {
                (*parent.as_ptr()).children.retain(|n| *n != node_ptr);
            }
            for child in node.children.drain(..) {
                (*child.as_ptr()).parent = None;
            }
            let next = node.next.take();
            let prev = node.prev.take();
            if let Some(next) = next {
                (*next.as_ptr()).prev = prev;
            }
            if let Some(prev) = prev {
                (*prev.as_ptr()).next = next;
            }
        }
        drop(node);
    }
}
