use crate::json_selection::unprocessed_elements::*;
use crate::syn::syn_elements::*;
use std::collections::HashMap;
use syntax_queries::HasByteRange;
// ─── from_unprocessed ────────────────────────────────────────────────────────
impl AllSynElements {
    pub fn from_unprocessed(u: AllUnprocessedElements) -> Self {
        let syn_methods: SynMethods = u
            .unprocessed_methods
            .into_iter()
            .map(SynMethod::from)
            .collect();

        let syn_attributes: SynAttributes = u
            .unprocessed_attributes
            .into_iter()
            .map(|raw| SynAttribute::from_unprocessed(raw, ""))
            .collect();

        AllSynElements {
            syn_attributes,
            syn_tests_mods: u.unprocessed_tests_mods,
            syn_functions: u.unprocessed_functions,
            syn_methods,
            syn_impls: u.unprocessed_impls,
            syn_structs: u.unprocessed_structs,
            syn_traits: u.unprocessed_traits,
            syn_trait_method_sigs: u.unprocessed_trait_method_sigs,
            syn_trait_method_defs: u.unprocessed_trait_method_defs,
            syn_type_aliases: u.unprocessed_type_aliases,
            syn_enums: u.unprocessed_enums,
            syn_unions: u.unprocessed_unions,
        }
    }
}

// ─── from_unprocessed_with_files ─────────────────────────────────────────────
impl AllSynElements {
    pub fn from_unprocessed_with_files(
        u: AllUnprocessedElements,
        file_contents: &HashMap<FilePath, String>,
    ) -> Self {
        let syn_methods: SynMethods = u
            .unprocessed_methods
            .into_iter()
            .map(SynMethod::from)
            .collect();

        // unprocessed_attributes is already Vec<SynAttribute> — assign directly
        let syn_attributes: SynAttributes = u.unprocessed_attributes;

        AllSynElements {
            syn_attributes,
            syn_tests_mods: u.unprocessed_tests_mods,
            syn_functions: u.unprocessed_functions,
            syn_methods,
            syn_impls: u.unprocessed_impls,
            syn_structs: u.unprocessed_structs,
            syn_traits: u.unprocessed_traits,
            syn_trait_method_sigs: u.unprocessed_trait_method_sigs,
            syn_trait_method_defs: u.unprocessed_trait_method_defs,
            syn_type_aliases: u.unprocessed_type_aliases,
            syn_enums: u.unprocessed_enums,
            syn_unions: u.unprocessed_unions,
        }
    }
}

impl AllSynElements {
    /// Convert `UnprocessedAttribute`s into `SynAttribute`s, using
    /// `file_contents` (a map from file path → raw source) to determine
    /// context lines and whitespace gaps.
    ///
    /// Attributes in the same file that have *only whitespace* between them
    /// are merged into a single `SynAttribute`.  Merging is performed
    /// iteratively: each attribute looks at the one immediately before it
    /// (using `HasByteRange::immediate_before`); if they are adjacent
    /// (whitespace-only gap) they are merged.
    ///
    /// The method replaces `self.syn_attributes` in place and returns a
    /// reference to the updated collection.
    pub fn merge_adjacent_attributes(
        &mut self,
        file_contents: &HashMap<FilePath, String>,
    ) {
        // ── 1. Group the raw UnprocessedAttributes by file ────────────────────
        // We work with SynAttribute from the start so we can call merge_with.
        let mut by_file: HashMap<FilePath, Vec<SynAttribute>> = HashMap::new();

        for raw in std::mem::take(&mut self.syn_attributes) {
            let content = file_contents
                .get(&raw.file)
                .map(String::as_str)
                .unwrap_or("");
            let syn_attr = SynAttribute::from_unprocessed(raw, content);
            by_file
                .entry(syn_attr.file.clone())
                .or_default()
                .push(syn_attr);
        }

        // ── 2. Within each file: sort by start byte, then merge adjacent ──────
        let mut merged_all: SynAttributes = Vec::new();

        for (file, mut attrs) in by_file {
            let content = file_contents.get(&file).map(String::as_str).unwrap_or("");

            // Sort by start byte so we can use a simple left-to-right sweep.
            attrs.sort_by_key(|a| a.attribute_body.range.byte_range.start);

            // We use a stack-based merge: pop the last accumulated attribute,
            // check if the current one is immediately adjacent (whitespace only),
            // and either merge or push both.
            let mut accumulated: Vec<SynAttribute> = Vec::with_capacity(attrs.len());

            for current in attrs {
                if let Some(prev) = accumulated.last() {
                    // HasByteRange::before / only_whitespace_between live on SynElement
                    // We access them through attribute_body.
                    let prev_body  = &prev.attribute_body;
                    let curr_body  = &current.attribute_body;

                    let is_before     = prev_body.before(curr_body);
                    let only_ws       = prev_body.only_whitespace_between(curr_body, content);

                    if is_before && only_ws {
                        // Merge current into prev (pop, merge, push back).
                        let prev_owned = accumulated.pop().unwrap();
                        let merged = prev_owned.merge_with(&current, content);
                        accumulated.push(merged);
                        continue;
                    }
                }
                accumulated.push(current);
            }

            merged_all.extend(accumulated);
        }

        // ── 3. Store result ───────────────────────────────────────────────────
        self.syn_attributes = merged_all;
    }
}
