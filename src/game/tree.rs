use crate::sys::{Components, UID, delete::Delete};

#[derive(Debug, Default)]
pub struct Tree {
    nodes: Components<Node>,
    roots: Vec<UID>,
}

#[derive(Debug)]
pub struct Node {
    parent: Option<UID>,
    children: Vec<UID>,
}

impl Tree {
    pub fn parent(&mut self, child: UID, parent: Option<UID>, index: Option<usize>) {
        if let Some(child_node) = self.nodes.get_mut(&child) {
            let prev_parent = child_node.parent;
            child_node.parent = parent;
            let prev_siblings = self.get_children_mut(prev_parent.as_ref()).unwrap();
            let index = prev_siblings
                .iter()
                .position(|sibling| *sibling == child)
                .unwrap();
            prev_siblings.remove(index);
        } else {
            self.nodes.insert(
                child,
                Node {
                    parent,
                    children: Vec::new(),
                },
            );
        }
        let siblings =
            if let Some(parent_node) = parent.and_then(|parent| self.nodes.get_mut(&parent)) {
                &mut parent_node.children
            } else {
                &mut self.roots
            };
        if let Some(index) = index {
            siblings.insert(index, child);
        } else {
            siblings.push(child);
        }
    }
    pub fn get_children(&self, parent: Option<&UID>) -> Option<&Vec<UID>> {
        if let Some(parent) = parent {
            self.nodes.get(parent).map(|parent| &parent.children)
        } else {
            Some(&self.roots)
        }
    }
    fn get_children_mut(&mut self, parent: Option<&UID>) -> Option<&mut Vec<UID>> {
        if let Some(parent) = parent {
            self.nodes
                .get_mut(parent)
                .map(|parent| &mut parent.children)
        } else {
            Some(&mut self.roots)
        }
    }
    pub fn get_child(&self, parent: Option<&UID>, index: usize) -> Option<&UID> {
        self.get_children(parent)
            .and_then(|children| children.get(index))
    }
    pub fn nodes(&self) -> &Components<Node> {
        &self.nodes
    }
    pub fn roots(&self) -> &Vec<UID> {
        &self.roots
    }
    pub fn dfs_post(&self) -> DFSPost {
        DFSPost::new(self)
    }
    pub fn dfs_pre(&self) -> DFSPre {
        DFSPre::new(self)
    }
}

impl Delete for Tree {
    fn delete(&mut self, uid: &UID) {
        let Some(node) = self.nodes.remove(uid) else {
            return;
        };
        if let Some(siblings) = self.get_children_mut(node.parent.as_ref()) {
            let index = siblings.iter().position(|sibling| sibling == uid).unwrap();
            siblings.remove(index);
        }
        for child in node.children {
            self.parent(child, None, None);
        }
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
            let Some(child) = self.tree.get_child(parent.as_ref(), *i).copied() else {
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
            let Some(child) = self.tree.get_child(parent.as_ref(), *i) else {
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
            stack: vec![(None, 0)],
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
            let Some(child) = self.tree.get_child(parent.as_ref(), *i) else {
                self.stack.pop();
                continue;
            };
            *i += 1;
            self.stack.push((Some(*child), 0));
            break child;
        })
    }
}
