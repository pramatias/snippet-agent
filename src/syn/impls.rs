use crate::syn::syn_elements::*;
use std::collections::HashSet;

impl AllSynElements {
    /// Build a B+Tree keyed by (hash of filepath, hash of impl_body.text) from all SynMethods,
    /// then filter syn_impls to only those whose (file, impl_body.text) hash pair is NOT present.
pub fn pick_blanket_impls(&mut self) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_pair(a: &str, b: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        a.hash(&mut hasher);
        b.hash(&mut hasher);
        hasher.finish()
    }

    let method_index: HashSet<u64> = self
    .syn_methods
    .iter()
    .map(|m| hash_pair(&m.file, &m.impl_body.text))
    .collect();

self.syn_impls.retain(|impl_item| {
    !method_index.contains(&hash_pair(&impl_item.file, &impl_item.impl_body.text))
});
}
}
