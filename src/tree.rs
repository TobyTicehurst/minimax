use std::sync::atomic::{AtomicUsize, Ordering};

const MAX_TREE_SIZE: usize = 50;

pub struct Tree<const N: usize> {
    data: [Node<N>; MAX_TREE_SIZE],
    head: AtomicUsize,
}

impl<const N: usize> Tree<N> {
    pub fn new() -> Tree<N> {
        Tree {
            data: [Node {evaluation: 0, children: [None; N]}; MAX_TREE_SIZE],
            head: AtomicUsize::new(0),
        }
    }

    pub fn create_root(&mut self) -> usize {
        return self.head.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_child(&self, index: Option<usize>, child_index: usize) -> Option<usize> {
        match index {
            Some(x) => return self.data[x].children[child_index],
            None => return None,
        }
    }

    fn add(&mut self, parent: &mut Node<N>, child_index: usize) {
        // reserve a space in memory
        let memory_index = self.head.fetch_add(1, Ordering::SeqCst);
        parent.children[child_index] = Some(memory_index);
    }

    fn get(&mut self, index: usize) -> &mut Node<N> {
        return &mut self.data[index];
    }
}

// a node with N children
#[derive(Debug, Clone, Copy)]
pub struct Node<const N: usize> {
    evaluation: i32,
    children: [Option<usize>; N],
}

