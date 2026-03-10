// file_syn_elements_tree.rs
use crate::AllSynElements;
use crate::syn::FilePath;
use crate::syn::file_syn_elements::{AnyFileSynElement, FileSynElements};
use indextree::{Arena, NodeId};
use std::collections::BTreeMap;
use syntax_queries::byte_range_ordering::HasByteRange;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct DepthNode {
    /// Index into the originating FileSynElements::elements.
    /// None for the sentinel root.
    pub element_idx: Option<usize>,
    pub depth: usize,
    pub is_attribute: bool,
}

#[derive(Debug)]
pub struct FileSynElementTree {
    pub arena: Arena<DepthNode>,
    /// Invisible root; all top-level elements are its direct children.
    pub root: NodeId,
}

///from_file_syn_elements
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
            let is_attribute = matches!(el, AnyFileSynElement::Attribute(_));

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

///remove_deeper_than
impl FileSynElementTree {
    pub fn remove_deeper_than(&mut self, max_depth: usize) {
        let to_remove: Vec<NodeId> = self
            .root
            .descendants(&self.arena)
            .skip(1)
            .filter(|&id| self.arena[id].get().depth > max_depth)
            .collect();
        for id in to_remove {
            id.remove_subtree(&mut self.arena);
        }
    }
}

///remove_excess_attributes
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
            self.trim_attr_run_under(parent, max_attrs);
        }
    }
}

///trim_attr_run_under
impl FileSynElementTree {
    fn trim_attr_run_under(&mut self, parent: NodeId, max_attrs: usize) {
        let children: Vec<NodeId> = parent.children(&self.arena).collect();
        let mut run: Vec<NodeId> = Vec::new();

        let flush = |run: &mut Vec<NodeId>, arena: &mut Arena<DepthNode>| {
            if run.len() > max_attrs {
                for &id in run.iter().skip(max_attrs) {
                    id.remove_subtree(arena);
                }
            }
            run.clear();
        };

        for child_id in children {
            if self.arena.get(child_id).is_none() {
                flush(&mut run, &mut self.arena);
                continue;
            }
            if self.arena[child_id].get().is_attribute {
                run.push(child_id);
            } else {
                flush(&mut run, &mut self.arena);
            }
        }
        flush(&mut run, &mut self.arena);
    }
}

///surviving_indices
impl FileSynElementTree {
    pub fn surviving_indices(&self) -> Vec<usize> {
        self.root
            .descendants(&self.arena)
            .skip(1)
            .filter_map(|id| self.arena[id].get().element_idx)
            .collect()
    }
}

///indices_to_invalidate
impl FileSynElementTree {
    /// Returns the set of element indices (into FileSynElements::elements)
    /// that the tree filter would remove. Does NOT mutate the arena.
    pub fn indices_to_invalidate(
        &self,
        max_depth: usize,
        max_attrs: usize,
    ) -> HashSet<usize> {
        let mut to_invalidate = HashSet::new();

        // ── 1. Elements deeper than max_depth ─────────────────────────────
        for id in self.root.descendants(&self.arena).skip(1) {
            let node = self.arena[id].get();
            if node.depth > max_depth {
                if let Some(idx) = node.element_idx {
                    to_invalidate.insert(idx);
                }
            }
        }

        // ── 2. Excess attribute runs under every parent ────────────────────
        let parents: Vec<NodeId> = std::iter::once(self.root)
            .chain(
                self.root
                    .descendants(&self.arena)
                    .skip(1)
                    .filter(|&id| self.arena[id].last_child().is_some()),
            )
            .collect();

        for parent in parents {
            let mut attr_run: Vec<usize> = Vec::new();

            for child_id in parent.children(&self.arena) {
                let node = self.arena[child_id].get();
                if node.is_attribute {
                    if let Some(idx) = node.element_idx {
                        attr_run.push(idx);
                    }
                } else {
                    if attr_run.len() > max_attrs {
                        for &idx in attr_run.iter().skip(max_attrs) {
                            to_invalidate.insert(idx);
                        }
                    }
                    attr_run.clear();
                }
            }

            // flush trailing run
            if attr_run.len() > max_attrs {
                for &idx in attr_run.iter().skip(max_attrs) {
                    to_invalidate.insert(idx);
                }
            }
        }

        to_invalidate
    }
}

//Sentinel variant
impl AnyFileSynElement {
    fn sentinel() -> Self {
        AnyFileSynElement::Sentinel
    }
}
