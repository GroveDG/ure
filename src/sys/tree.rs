use super::{Components, UID};

#[derive(Debug, Clone)]
pub struct Tree {
    pub root: UID,
    nodes: Components<Node>,
}
#[derive(Debug, Clone)]
pub struct Node {
    parent: UID,
    children: Vec<UID>,
}

impl Tree {
    pub fn new(root: UID) -> Self {
        Self {
            root,
            nodes: Default::default(),
        }
    }
    fn get(&self, uid: &UID) -> Result<&Node, ()> {
        self.nodes.get(uid).ok_or(())
    }
    pub fn children(&self, uid: &UID) -> Result<std::slice::Iter<'_, UID>, ()> {
        self.get(uid).map(|node| node.children.iter())
    }
    pub fn dfs(&self) -> DFS<'_>
    where
        Self: Sized,
    {
        DFS::new(self)
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
    node: UID,
}
impl<'a> Iterator for Ancestors<'a> {
    type Item = UID;

    fn next(&mut self) -> Option<Self::Item> {
        let branch = self.tree.get(&self.node).ok()?;
        let this = self.node;
        self.node = branch.parent;
        Some(this)
    }
}
impl<'a> Ancestors<'a> {
    pub fn new(tree: &'a Tree, node: UID) -> Self {
        Self { tree, node }
    }
}

#[derive(Debug, Clone)]
pub struct DFS<'a> {
    tree: &'a Tree,
    parent: UID,
    path: Vec<usize>,
}
impl<'a> Iterator for DFS<'a> {
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
            self.parent = parent.parent;
        }

        Some(Ok(this))
    }
}
impl<'a> DFS<'a> {
    pub fn new(tree: &'a Tree) -> Self {
        let mut dfs = Self {
            tree: tree,
            parent: tree.root,
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
