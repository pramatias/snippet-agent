// file_syn_elements_tree.rs
use crate::syn::file_syn_elements::{AnyFileSynElement, FileSynElements};
use std::collections::HashSet;
use syntax_queries::byte_range_ordering::HasByteRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(usize);

#[derive(Debug, Clone)]
pub struct ArenaNode<T> {
    pub data: T,
    parent: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
    next_sibling: Option<NodeId>,
    prev_sibling: Option<NodeId>,
    /// Tombstone flag – set by `remove_subtree`.
    removed: bool,
}

impl<T> ArenaNode<T> {
    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn first_child(&self) -> Option<NodeId> {
        self.first_child
    }
    pub fn last_child(&self) -> Option<NodeId> {
        self.last_child
    }
    pub fn next_sibling(&self) -> Option<NodeId> {
        self.next_sibling
    }
}

#[derive(Debug, Clone)]
pub struct Arena<T> {
    nodes: Vec<ArenaNode<T>>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena { nodes: Vec::new() }
    }

    pub fn new_node(&mut self, data: T) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(ArenaNode {
            data,
            parent: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
            removed: false,
        });
        id
    }

    /// `None` if the node has been removed (tombstoned) or never existed.
    pub fn get(&self, id: NodeId) -> Option<&ArenaNode<T>> {
        self.nodes.get(id.0).filter(|n| !n.removed)
    }
}

impl<T> std::ops::Index<NodeId> for Arena<T> {
    type Output = ArenaNode<T>;
    fn index(&self, id: NodeId) -> &ArenaNode<T> {
        &self.nodes[id.0]
    }
}

impl<T> std::ops::IndexMut<NodeId> for Arena<T> {
    fn index_mut(&mut self, id: NodeId) -> &mut ArenaNode<T> {
        &mut self.nodes[id.0]
    }
}

impl NodeId {
    /// Append `child` as the last child of `self`.
    pub fn append<T>(self, child: NodeId, arena: &mut Arena<T>) {
        // Detach child from any previous parent first (not needed here, but
        // keeps the API safe).
        debug_assert!(arena[child].parent.is_none(), "child already has a parent");

        arena[child].parent = Some(self);

        match arena[self].last_child {
            None => {
                // First child.
                arena[self].first_child = Some(child);
                arena[self].last_child = Some(child);
            }
            Some(prev_last) => {
                arena[prev_last].next_sibling = Some(child);
                arena[child].prev_sibling = Some(prev_last);
                arena[self].last_child = Some(child);
            }
        }
    }

    /// Remove `self` *and its entire subtree* by tombstoning every node in it.
    /// Also unlinks `self` from its parent's child list so that sibling
    /// iteration skips the removed node.
    pub fn remove_subtree<T>(self, arena: &mut Arena<T>) {
        // Unlink self from parent's child list.
        self.unlink_from_parent(arena);

        // Tombstone self and all descendants via an explicit stack (avoids
        // recursion-depth problems on large subtrees).
        let mut stack = vec![self];
        while let Some(id) = stack.pop() {
            // Collect children before tombstoning.
            let mut child_opt = arena[id].first_child;
            while let Some(c) = child_opt {
                stack.push(c);
                child_opt = arena[c].next_sibling;
            }
            arena[id].removed = true;
        }
    }

    /// Iterate `self` followed by all descendants in pre-order (parent before
    /// children, children in insertion order).  Removed nodes are skipped.
    pub fn descendants<T>(self, arena: &Arena<T>) -> Descendants<T> {
        Descendants {
            arena,
            // We use an explicit stack; start with self.
            stack: vec![self],
        }
    }

    // ── private helpers ──────────────────────────────────────────────────────

    fn unlink_from_parent<T>(self, arena: &mut Arena<T>) {
        let parent = match arena[self].parent {
            Some(p) => p,
            None => return,
        };

        let prev = arena[self].prev_sibling;
        let next = arena[self].next_sibling;

        // Patch previous sibling.
        if let Some(p) = prev {
            arena[p].next_sibling = next;
        } else {
            // self was first child.
            arena[parent].first_child = next;
        }

        // Patch next sibling.
        if let Some(n) = next {
            arena[n].prev_sibling = prev;
        } else {
            // self was last child.
            arena[parent].last_child = prev;
        }

        arena[self].parent = None;
    }
}

/// Pre-order iterator over a subtree.  Yields `NodeId`s.
pub struct Descendants<'a, T> {
    arena: &'a Arena<T>,
    stack: Vec<NodeId>,
}

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        // Skip tombstoned nodes; they can appear if `remove_subtree` was
        // called after this iterator was created (not the case here, but
        // defensive).
        while let Some(id) = self.stack.pop() {
            if self.arena[id].removed {
                continue;
            }
            // Push children in *reverse* order so the first child is popped first.
            let mut children: Vec<NodeId> = Vec::new();
            let mut child_opt = self.arena[id].first_child;
            while let Some(c) = child_opt {
                if !self.arena[c].removed {
                    children.push(c);
                }
                child_opt = self.arena[c].next_sibling;
            }
            for c in children.into_iter().rev() {
                self.stack.push(c);
            }
            return Some(id);
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct DepthNode {
    pub element_idx: Option<usize>,
    pub depth: usize,
    pub is_attribute: bool,
}

#[derive(Debug)]
pub struct FileSynElementTree {
    pub arena: Arena<DepthNode>,
    pub root: NodeId,
}

// ── from_file_syn_elements ────────────────────────────────────────────────────

impl FileSynElementTree {
    pub fn from_file_syn_elements(fse: &FileSynElements) -> Self {
        let mut arena: Arena<DepthNode> = Arena::new();
        let root = arena.new_node(DepthNode {
            element_idx: None,
            depth: 0,
            is_attribute: false,
        });
        let mut stack: Vec<(NodeId, u64)> = vec![(root, u64::MAX)];

        for (idx, el) in fse.elements.iter().enumerate() {
            let el_start = el.byte_range().start;
            let el_end = el.byte_range().end;

            while stack.len() > 1 {
                let (_, ancestor_end) = *stack.last().unwrap();
                if ancestor_end <= el_start {
                    stack.pop();
                } else {
                    break;
                }
            }

            let (parent_id, _) = *stack.last().unwrap();
            let depth = stack.len();
            let is_attribute = el.is_attribute();

            let node_id = arena.new_node(DepthNode {
                element_idx: Some(idx),
                depth,
                is_attribute,
            });
            parent_id.append(node_id, &mut arena);
            stack.push((node_id, el_end));
        }

        FileSynElementTree { arena, root }
    }
}

// ── remove_deeper_than ───────────────────────────────────────────────────────

impl FileSynElementTree {
    pub fn remove_deeper_than(&mut self, max_depth: usize) {
        let to_remove: Vec<NodeId> = self
            .root
            .descendants(&self.arena)
            .skip(1)
            .filter(|&id| self.arena[id].get().depth == max_depth + 1)
            .collect();

        for id in to_remove {
            id.remove_subtree(&mut self.arena);
        }
    }
}

// ── remove_excess_attributes ─────────────────────────────────────────────────

impl FileSynElementTree {
    pub fn remove_excess_attributes(&mut self, max_attrs: usize) {
        let parents: Vec<NodeId> = std::iter::once(self.root)
            .chain(
                self.root
                    .descendants(&self.arena)
                    .skip(1)
                    .filter(|&id| self.arena[id].last_child().is_some()),
            )
            .collect();

        for parent in parents {
            if self.arena.get(parent).is_none() {
                continue;
            }
            self.trim_attr_run_under(parent, max_attrs);
        }
    }
}

// ── trim_attr_run_under ───────────────────────────────────────────────────────

impl FileSynElementTree {
    fn trim_attr_run_under(&mut self, parent: NodeId, max_attrs: usize) {
        let mut run: Vec<NodeId> = Vec::new();
        let mut to_remove: Vec<NodeId> = Vec::new();
        let mut child_opt = self.arena[parent].first_child();

        while let Some(child_id) = child_opt {
            child_opt = self.arena[child_id].next_sibling();

            if self.arena[child_id].get().is_attribute {
                run.push(child_id);
            } else {
                if run.len() > max_attrs {
                    to_remove.extend(run.drain(max_attrs..));
                }
                run.clear();
            }
        }
        if run.len() > max_attrs {
            to_remove.extend(run.drain(max_attrs..));
        }

        for id in to_remove {
            id.remove_subtree(&mut self.arena);
        }
    }
}

// ── surviving_indices ─────────────────────────────────────────────────────────

impl FileSynElementTree {
    pub fn surviving_indices(&self) -> Vec<usize> {
        self.root
            .descendants(&self.arena)
            .skip(1)
            .filter_map(|id| self.arena[id].get().element_idx)
            .collect()
    }
}

// ── Public helpers ────────────────────────────────────────────────────────────

pub fn filter_elements(fse: &mut FileSynElements, max_depth: usize, max_attrs: usize) {
    fse.elements.sort_by(|a, b| {
        let ra = a.byte_range();
        let rb = b.byte_range();
        ra.start.cmp(&rb.start).then(rb.end.cmp(&ra.end))
    });

    let mut tree = FileSynElementTree::from_file_syn_elements(fse);
    tree.remove_deeper_than(max_depth);
    tree.remove_excess_attributes(max_attrs);

    let survivors: HashSet<usize> = tree.surviving_indices().into_iter().collect();

    let old = std::mem::take(&mut fse.elements);
    fse.elements = old
        .into_iter()
        .enumerate()
        .filter(|(i, _)| survivors.contains(i))
        .map(|(_, el)| el)
        .collect();
}

pub fn indices_to_invalidate(
    fse: &FileSynElements,
    max_depth: usize,
    max_attrs: usize,
) -> HashSet<usize> {
    let mut tree = FileSynElementTree::from_file_syn_elements(fse);
    tree.remove_deeper_than(max_depth);
    tree.remove_excess_attributes(max_attrs);

    let survivors: HashSet<usize> = tree.surviving_indices().into_iter().collect();
    (0..fse.elements.len())
        .filter(|i| !survivors.contains(i))
        .collect()
}
