use serde::{
    Deserialize, Serialize,
    ser::{SerializeSeq, SerializeTupleStruct},
};

use super::{Components, UID};

#[derive(Debug, Default, Deserialize)]
pub struct Tree {
    map: Components<Node>,
    roots: Vec<UID>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    parent: Option<UID>,
    children: Vec<UID>,
}

impl Tree {
    pub fn insert(&mut self, uid: UID, parent: Option<UID>) {
        self.add_child(uid, parent.as_ref());
        self.map.insert(
            uid,
            Node {
                parent,
                children: Vec::new(),
            },
        );
    }

    /// Returns an Err if parent does not exist.
    fn add_child(&mut self, uid: UID, parent: Option<&UID>) {
        if let Some(parent) = parent {
            self.get_mut(parent).unwrap().children.push(uid);
        } else {
            self.roots.push(uid);
        }
    }

    pub fn get(&self, uid: &UID) -> Option<&Node> {
        self.map.get(uid)
    }

    fn get_mut(&mut self, uid: &UID) -> Option<&mut Node> {
        self.map.get_mut(uid)
    }

    pub fn children(&self, parent: Option<&UID>) -> Option<&Vec<UID>> {
        if let Some(parent) = parent {
            self.map.get(parent).map(|node| &node.children)
        } else {
            Some(&self.roots)
        }
    }

    pub fn delete(&mut self, uid: &UID) {
        let Some(node) = self.map.remove(uid) else {
            return;
        };
        if let Some(parent) = node.parent {
            self.map
                .get_mut(&parent)
                .unwrap()
                .children
                .retain(|child| child != uid);
        } else {
            self.roots.retain(|root| root != uid);
        }
    }

    pub fn child(&self, parent: Option<&UID>, i: usize) -> Option<&UID> {
        if let Some(parent) = parent {
            &self.get(parent).unwrap().children
        } else {
            &self.roots
        }
        .get(i)
    }

    pub fn dfs_post(&self) -> DFSPost {
        DFSPost::new(self)
    }

    pub fn dfs_pre(&self) -> DFSPre {
        DFSPre::new(self)
    }
}

pub struct DFSPost<'a> {
    tree: &'a Tree,
    stack: Vec<(Option<UID>, usize)>,
}
impl<'a> DFSPost<'a> {
    fn new(tree: &'a Tree) -> Self {
        let mut dfs = Self {
            tree,
            stack: vec![(None, 0)],
        };
        dfs.descend();
        dfs
    }

    fn descend(&mut self) {
        loop {
            let (parent, i) = self.stack.last().unwrap();
            let Some(child) = self.tree.child(parent.as_ref(), *i).copied() else {
                break;
            };
            self.stack.push((Some(child), 0));
        }
    }
}
impl<'a> Iterator for DFSPost<'a> {
    type Item = &'a UID;

    fn next(&mut self) -> Option<Self::Item> {
        Some(loop {
            let Some((parent, i)) = self.stack.last_mut() else {
                return None;
            };
            let Some(child) = self.tree.child(parent.as_ref(), *i) else {
                self.stack.pop();
                continue;
            };
            *i += 1;
            self.descend();
            break child;
        })
    }
}

pub struct DFSPre<'a> {
    tree: &'a Tree,
    stack: Vec<(Option<UID>, usize)>,
}
impl<'a> DFSPre<'a> {
    fn new(tree: &'a Tree) -> Self {
        Self {
            tree,
            stack: vec![(tree.child(None, 0).copied(), 0)],
        }
    }
}
impl<'a> Iterator for DFSPre<'a> {
    type Item = &'a UID;

    fn next(&mut self) -> Option<Self::Item> {
        Some(loop {
            let Some((parent, i)) = self.stack.last_mut() else {
                return None;
            };
            let Some(child) = self.tree.child(parent.as_ref(), *i) else {
                self.stack.pop();
                continue;
            };
            *i += 1;
            self.stack.push((Some(*child), 0));
            break child;
        })
    }
}

impl Serialize for Tree {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        struct ChildNodes<'a> {
            tree: &'a Tree,
            parent: Option<&'a UID>,
        }
        impl<'a> Serialize for ChildNodes<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let children = self.tree.children(self.parent).unwrap();
                let mut serializer = serializer.serialize_seq(Some(children.len()))?;
                for child in children {
                    if self.tree.children(Some(child)).unwrap().is_empty() {
                        serializer.serialize_element(child)?;
                    } else {
                        serializer.serialize_element(&(
                            child,
                            Self {
                                tree: self.tree,
                                parent: Some(child),
                            },
                        ))?;
                    }
                }
                serializer.end()
            }
        }
        let mut serializer = serializer.serialize_tuple_struct("Tree", 1)?;
        serializer.serialize_field(&ChildNodes {
            tree: self,
            parent: None,
        })?;
        serializer.end()
    }
}
