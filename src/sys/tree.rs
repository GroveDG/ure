use serde::{Deserialize, Serialize};

use super::{Components, UID};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Tree {
    roots: Vec<UID>,
    nodes: Components<Node>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    parent: Option<UID>,
    children: Vec<UID>,
}

impl Tree {
    /// Add an entity as a node.
    pub fn insert(&mut self, uid: UID, parent: Option<UID>) -> Result<(), ()> {
        if let Some(parent) = parent {
            self.nodes.get_mut(&parent).ok_or(())?.children.push(uid);
        } else {
            self.roots.push(uid);
        }
        self.nodes.insert(
            uid,
            Node {
                parent,
                children: Vec::new(),
            },
        );
        Ok(())
    }
    pub fn delete(&mut self, uid: &UID) {
        if let Some(i) = self.roots.iter().position(|r| r == uid) {
            self.roots.remove(i);
        }
        self.nodes.remove(uid);
    }
    pub fn get(&self, uid: &UID) -> Result<&Node, ()> {
        self.nodes.get(uid).ok_or(())
    }
    pub fn children(&self, uid: &UID) -> Result<std::slice::Iter<'_, UID>, ()> {
        self.get(uid).map(|node| node.children.iter())
    }
    /// Depth-fist post order
    /// 
    /// This is UI layout order, 2D render order, and script execution order.
    /// 
    /// https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN
    pub fn lrn(&self, root: UID) -> LRN<'_>
    where
        Self: Sized,
    {
        LRN::new(self, root)
    }
    pub fn ancestors(&self, node: UID) -> Ancestors<'_> {
        Ancestors::new(self, node)
    }
    pub fn global<C: Compose>(&self, node: UID, c: &Components<C>) -> Option<C> {
        self.ancestors(node)
            .map_while(|n| c.get(&n).cloned())
            .reduce(C::compose)
    }
    pub fn inherited<'a, C>(&self, node: UID, c: &'a Components<C>) -> Option<&'a C> {
        self.ancestors(node).find_map(|n| c.get(&n))
    }
}

pub trait Compose: Clone {
    fn compose(self, parent: Self) -> Self;
}

#[derive(Debug, Clone, Copy)]
pub struct Ancestors<'a> {
    tree: &'a Tree,
    node: Option<UID>,
}
impl<'a> Iterator for Ancestors<'a> {
    type Item = UID;

    fn next(&mut self) -> Option<Self::Item> {
        let this = self.node?;
        self.node = self.tree.get(&this).map_or(None, |n| n.parent);
        Some(this)
    }
}
impl<'a> Ancestors<'a> {
    pub fn new(tree: &'a Tree, node: UID) -> Self {
        Self {
            tree,
            node: Some(node),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LRN<'a> {
    tree: &'a Tree,
    parent: UID,
    path: Vec<usize>,
}
impl<'a> Iterator for LRN<'a> {
    type Item = Result<UID, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.path.last_mut()?;
        let parent = match self.tree.get(&self.parent) {
            Ok(p) => p,
            Err(e) => return Some(Err(e)),
        };
        let this = *parent.children.get(*index).unwrap();

        *index += 1;
        if *index >= parent.children.len() {
            self.path.pop();
            self.parent = parent.parent?;
        }

        Some(Ok(this))
    }
}
impl<'a> LRN<'a> {
    pub fn new(tree: &'a Tree, root: UID) -> Self {
        let mut dfs = Self {
            tree,
            parent: root,
            path: Vec::new(),
        };
        dfs.down();
        dfs
    }
    // Traverse down to the first descendant without children.
    fn down(&mut self) -> Result<(), ()> {
        loop {
            let Some(node) = self.tree.nodes.get(&self.parent) else {
                return Err(());
            };
            if let Some(child) = node.children.first() {
                self.parent = *child;
                self.path.push(0);
            } else {
                return Ok(());
            }
        }
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// struct Branch(UID, Vec<Branch>);

// impl Serialize for Tree {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         fn branch(t: &Tree, node: &UID) -> Branch {
//             Branch(
//                 *node,
//                 t.nodes[node]
//                     .children
//                     .iter()
//                     .map(|n| branch(t, n))
//                     .collect(),
//             )
//         }
//         self.roots
//             .iter()
//             .map(|root| branch(self, root))
//             .collect::<Vec<_>>()
//             .serialize(serializer)
//     }
// }
