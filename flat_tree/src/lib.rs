use std::ops::Deref;

pub trait TreeNode {
    type Item;

    fn get_tree(&self) -> &FlatTree<Self::Item>;
    fn get_index(&self) -> usize;

    fn get_parent(&self) -> Option<Node<Self::Item>> {
        self.get_tree().parent_of(self.get_index())
    }
    fn get_item(&self) -> &Self::Item {
        self.get_tree().buf.get(self.get_index()).unwrap()
    }
}


#[derive(Debug, PartialEq)]
pub struct Node<'a, T> {
    tree: &'a FlatTree<T>,
    id: usize,
}
impl<T> TreeNode for Node<'_, T> {
    type Item = T;

    fn get_tree(&self) -> &FlatTree<T> {
        self.tree
    }

    fn get_index(&self) -> usize {
        self.id
    }
}
impl<'a, T> From<NodeMut<'a, T>> for Node<'a, T> {
    fn from(value: NodeMut<'a, T>) -> Node<'a, T> {
        let NodeMut { tree, id } = value;
        Node { tree, id }
    }
}
impl<T> Deref for Node<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.get_item()
    }
}

#[derive(Debug, PartialEq)]
pub struct NodeMut<'a, T> {
    tree: &'a mut FlatTree<T>,
    id: usize,
}
impl<T> TreeNode for NodeMut<'_, T> {
    type Item = T;

    fn get_tree(&self) -> &FlatTree<Self::Item> {
        self.tree
    }

    fn get_index(&self) -> usize {
        self.id
    }
}
impl<T> NodeMut<'_, T> {
    fn set_parent(self, parent: usize) -> Self {
        self.tree.set_parent(self.id, parent);
        self
    }
}


const SPARE_CAPACITY: usize = 10;
#[derive(Debug, PartialEq, Clone)]
pub struct FlatTree<T> {
    buf: Vec<T>,
    parents: Vec<Option<usize>>,
    len: usize
}
impl<T> FlatTree<T> {
    pub fn new() -> FlatTree<T> {
        FlatTree::with_capacity(SPARE_CAPACITY * 2)
    }

    pub fn with_capacity(capacity: usize) -> FlatTree<T> {
        let buf = Vec::with_capacity(capacity);
        let parents = Vec::with_capacity(capacity);
        FlatTree { buf, parents, len: capacity }
    }

    pub fn add(&mut self, value: T) -> NodeMut<'_, T> {
        self.buf.push(value);
        self.parents.push(None);
        self.len += 1;
        let id = self.len - 1;
        NodeMut { tree: self, id }
    }

    pub fn set_parent(&mut self, index: usize, parent: usize) {
        assert!(
            self.len >= index,
            "Cannot access values that haven't been set"
        );
        unsafe { *self.parents.get_unchecked_mut(index) = Some(parent) }
    }

    pub fn get(&self, index: usize) -> Node<T> {
        assert!(
            self.len >= index,
            "Cannot access values that haven't been set"
        );
        Node { tree: self, id: index }
    }

    fn parent_of(&self, index: usize) -> Option<Node<T>> {
        assert!(
            self.len >= index,
            "Cannot access values that haven't been set"
        );
        unsafe { self.parents.get_unchecked(index).map(|index| Node { tree: self, id: index }) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut tree = FlatTree::new();
        tree.add(2);

        let node = tree.get(0);
        assert_eq!(2, *node);

        assert_eq!(None, node.get_parent())
    }

    #[test]
    fn resizes() {
        let mut tree = FlatTree::with_capacity(10);
        for n in 1..20 {
            tree.add(n);
        }
    }

    #[test]
    #[should_panic(expected = "Cannot access values that haven't been set")]
    fn stop_illegal_access() {
        let tree: FlatTree<i16> = FlatTree::new();
        tree.get(10);
    }

    #[test]
    fn parent() {
        let mut tree = FlatTree::new();
        tree.add("Mother");
        let child = {
            let child = tree.add("Child");
            child.set_parent(0);
            child.into()
        }
        let parent = child.get_parent().unwrap();
        assert_eq!(parent, tree.get(0))
    }
}
