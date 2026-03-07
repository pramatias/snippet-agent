use crate::json_selection::unprocessed_elements::*;
use crate::syn::syn_elements::*;
use std::collections::HashMap;
use syntax_queries::HasByteRange;

///from unprocessed
impl AllSynElements {
    /// Convert an `AllUnprocessedElements` into `AllSynElements`.
    /// This conversion is shallow: it copies/aliases the unchanged collections and converts
    /// `UnprocessedImpl`s and `UnprocessedMethod`s into `SynImpl`/`SynMethod` with default
    /// processed fields. You can extend this to parse signatures/attributes and to attach
    /// methods to their impls by matching ranges/locations.
    pub fn from_unprocessed(u: AllUnprocessedElements) -> Self {
        // convert methods and impls using From implementations above
        let syn_methods: SynMethods = u
            .unprocessed_methods
            .into_iter()
            .map(SynMethod::from)
            .collect();

        AllSynElements {
            syn_attributes: u.unprocessed_attributes,
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

///print all
// impl AllSynElements {
//     pub fn print_all(&self) {
//         fn print_syn_element(label: &str, el: &SynElement, indent: usize) {
//             let pad = "  ".repeat(indent);
//             println!("{pad}{label}:");
//             let lines: Vec<&str> = el.text.lines().take(2).collect();
//             let preview = lines.join("\n");
//             println!("{pad}  preview: {:?}", preview);
//         }

//         // ── impls ─────────────────────────────────────────────────────────────
// println!("\n=== syn_impls ({}) ===", self.syn_impls.len());
// for (i, imp) in self.syn_impls.iter().enumerate() {
//     println!("  [{}] file: {}", i, imp.file);
//     println!("  impl_body: {}", imp.impl_body.text);
// }

        // // ── attributes ────────────────────────────────────────────────────────
        // println!("=== syn_attributes ({}) ===", self.syn_attributes.len());
        // for (i, attr) in self.syn_attributes.iter().enumerate() {
        //     println!("  [{}] file: {}", i, attr.file);
        //     print_syn_element("attribute_body", &attr.attribute_body, 2);
        // }

        // // ── test mods ─────────────────────────────────────────────────────────
        // println!("\n=== syn_tests_mods ({}) ===", self.syn_tests_mods.len());
        // for (i, tm) in self.syn_tests_mods.iter().enumerate() {
        //     println!("  [{}] file: {}", i, tm.file);
        //     print_syn_element("tests_mod_body", &tm.tests_mod_body, 2);
        // }

        // // ── functions ─────────────────────────────────────────────────────────
        // println!("\n=== syn_functions ({}) ===", self.syn_functions.len());
        // for (i, f) in self.syn_functions.iter().enumerate() {
        //     println!("  [{}] file: {}", i, f.file);
        //     print_syn_element("function_name", &f.function_name, 2);
        //     print_syn_element("function_body", &f.function_body, 2);
        // }

        // // ── methods ───────────────────────────────────────────────────────────
        // println!("\n=== syn_methods ({}) ===", self.syn_methods.len());
        // for (i, m) in self.syn_methods.iter().enumerate() {
        //     println!("  [{}] file: {}", i, m.file);
        //     println!("    {GREEN}impl_signature:{RESET}   {}", m.impl_signature);
        //     // type_identifiers
        //     println!("    {GREEN}type_identifiers:{RESET}");
        //     let ti = &m.type_identifiers;
        //     println!("      concrete_types: {YELLOW}{:?}{RESET}", ti.concrete_types);
        //     match &ti.type_variables {
        //         None => println!("      type_variables: None"),
        //         Some(tvars) => {
        //             println!("      type_variables:");
        //             let mut keys: Vec<_> = tvars.keys().collect();
        //             keys.sort();
        //             for k in keys {
        //                 println!("        {k}: {:?}", tvars[k]);
        //             }
        //         }
        //     }

        //     println!("    function_signature: {}", m.function_signature);
        //     println!("    {BLUE}ds_structure:{RESET}     {}", m.ds_structure);
        //     print_syn_element("method_name", &m.method_name, 2);
        //     print_syn_element("impl_body", &m.impl_body, 2);
        //     print_syn_element("method_body", &m.method_body, 2);
        // }
        // ── methods ───────────────────────────────────────────────────────────
        // println!("\n=== syn_methods ({}) ===", self.syn_methods.len());
        // for (i, m) in self.syn_methods.iter().enumerate() {
        //     println!("  [{}] file: {}", i, m.file);
        //     println!("    impl_signature:   {}", m.impl_signature);
        //     println!("    ds_structure:     {}\n", m.ds_structure);
        //     // type_identifiers
        //     println!("    type_identifiers:");
        //     let ti = &m.type_identifiers;
        //     match &ti.concrete_types {
        //         None => println!("      concrete_types: None"),
        //         Some(ct) => println!("      concrete_types: {:?}", ct),
        //     }
        //     match &ti.type_variables {
        //         None => println!("      type_variables: None"),
        //         Some(tvars) => {
        //             println!("      type_variables:");
        //             let mut keys: Vec<_> = tvars.keys().collect();
        //             keys.sort();
        //             for k in keys {
        //                 println!("        {k}: {:?}", tvars[k]);
        //             }
        //         }
        //     }
        // }

        // // ── structs ───────────────────────────────────────────────────────────
        // println!("\n=== syn_structs ({}) ===", self.syn_structs.len());
        // for (i, s) in self.syn_structs.iter().enumerate() {
        //     println!("  [{}] file: {}", i, s.file);
        //     print_syn_element("struct_name", &s.struct_name, 2);
        //     print_syn_element("struct_body", &s.struct_body, 2);
        // }

        // // ── traits ────────────────────────────────────────────────────────────
        // println!("\n=== syn_traits ({}) ===", self.syn_traits.len());
        // for (i, t) in self.syn_traits.iter().enumerate() {
        //     println!("  [{}] file: {}", i, t.file);
        //     print_syn_element("trait_name", &t.trait_name, 2);
        //     print_syn_element("trait_body", &t.trait_body, 2);
        // }

        // // ── trait method signatures ────────────────────────────────────────────
        // println!(
        //     "\n=== syn_trait_method_sigs ({}) ===",
        //     self.syn_trait_method_sigs.len()
        // );
        // for (i, sig) in self.syn_trait_method_sigs.iter().enumerate() {
        //     println!("  [{}] file: {}", i, sig.file);
        //     print_syn_element("trait_name", &sig.trait_name, 2);
        //     print_syn_element("trait_body", &sig.trait_body, 2);
        //     print_syn_element("method_signature_name", &sig.method_signature_name, 2);
        //     print_syn_element("trait_method_signature", &sig.trait_method_signature, 2);
        // }

        // // ── trait method definitions ───────────────────────────────────────────
        // println!(
        //     "\n=== syn_trait_method_defs ({}) ===",
        //     self.syn_trait_method_defs.len()
        // );
        // for (i, def) in self.syn_trait_method_defs.iter().enumerate() {
        //     println!("  [{}] file: {}", i, def.file);
        //     print_syn_element("trait_name", &def.trait_name, 2);
        //     print_syn_element("trait_body", &def.trait_body, 2);
        //     print_syn_element("method_name", &def.method_name, 2);
        //     print_syn_element("trait_method_body", &def.trait_method_body, 2);
        // }

        // // ── type aliases ──────────────────────────────────────────────────────
        // println!(
        //     "\n=== syn_type_aliases ({}) ===",
        //     self.syn_type_aliases.len()
        // );
        // for (i, ta) in self.syn_type_aliases.iter().enumerate() {
        //     println!("  [{}] file: {}", i, ta.file);
        //     print_syn_element("type_name", &ta.type_name, 2);
        //     print_syn_element("type_body", &ta.type_body, 2);
        // }

        // // ── enums ─────────────────────────────────────────────────────────────
        // println!("\n=== syn_enums ({}) ===", self.syn_enums.len());
        // for (i, e) in self.syn_enums.iter().enumerate() {
        //     println!("  [{}] file: {}", i, e.file);
        //     print_syn_element("enum_name", &e.enum_name, 2);
        //     print_syn_element("enum_body", &e.enum_body, 2);
        // }

        // // ── unions ────────────────────────────────────────────────────────────
        // println!("\n=== syn_unions ({}) ===", self.syn_unions.len());
        // for (i, u) in self.syn_unions.iter().enumerate() {
        //     println!("  [{}] file: {}", i, u.file);
        //     print_syn_element("union_name", &u.union_name, 2);
        //     print_syn_element("union_body", &u.union_body, 2);
        // }
//     }
// }

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

// ─────────────────────────────────────────────────────────────────────────────
// AllSynElements::from_unprocessed  (updated to build SynAttributes)
// ─────────────────────────────────────────────────────────────────────────────
//
// Replace the existing `from_unprocessed` with this version, which accepts
// file contents so it can build `SynAttribute`s properly.  Call
// `merge_adjacent_attributes` afterwards if you want merging.

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

        let syn_attributes: SynAttributes = u
            .unprocessed_attributes
            .into_iter()
            .map(|raw| {
                let content = file_contents
                    .get(&raw.file)
                    .map(String::as_str)
                    .unwrap_or("");
                SynAttribute::from_unprocessed(raw, content)
            })
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
