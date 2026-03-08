//unprocessed_syn_casting.rs
use crate::json_selection::unprocessed_elements::*;
use crate::syn::syn_elements::*;
use std::collections::HashMap;
use syntax_queries::byte_range_ordering::HasByteRange;
use crate::syn::FilePath;

macro_rules! impl_unprocessed_to_syn {
    ($source:ty => $target:ty [$($field:ident),+ $(,)?]) => {
        impl From<$source> for $target {
            fn from(u: $source) -> Self {
                Self {
                    file: u.file,
                    $($field: u.$field,)+
                }
            }
        }
    };
}

impl_unprocessed_to_syn!(UnprocessedMod                 => SynMod                 [mod_name, mod_body]);
impl_unprocessed_to_syn!(UnprocessedExpressionStatement  => SynExpressionStatement  [expression_body]);
impl_unprocessed_to_syn!(UnprocessedUseDeclaration       => SynUseDeclaration       [use_body]);
impl_unprocessed_to_syn!(UnprocessedMacroDefinition      => SynMacroDefinition      [macro_name, macro_body]);
impl_unprocessed_to_syn!(UnprocessedMacroInvocation      => SynMacroInvocation      [invocation_body]);

// ─── from_unprocessed ────────────────────────────────────────────────────────
impl AllSynElements {
    pub fn from_unprocessed(u: AllUnprocessedElements) -> Self {
        AllSynElements {
            syn_attributes:            u.unprocessed_attributes
                                        .into_iter()
                                        .map(|raw| SynAttribute::from_unprocessed(raw, ""))
                                        .collect(),
            syn_methods:               u.unprocessed_methods.into_iter().map(SynMethod::from).collect(),
            syn_mods:                  u.unprocessed_mods.into_iter().map(SynMod::from).collect(),
            syn_expression_statements: u.unprocessed_expression_statements.into_iter().map(SynExpressionStatement::from).collect(),
            syn_use_declarations:      u.unprocessed_use_declarations.into_iter().map(SynUseDeclaration::from).collect(),
            syn_macro_definitions:     u.unprocessed_macro_definitions.into_iter().map(SynMacroDefinition::from).collect(),
            syn_macro_invocations:     u.unprocessed_macro_invocations.into_iter().map(SynMacroInvocation::from).collect(),
            syn_tests_mods:            u.unprocessed_tests_mods,
            syn_functions:             u.unprocessed_functions,
            syn_impls:                 u.unprocessed_impls,
            syn_structs:               u.unprocessed_structs,
            syn_traits:                u.unprocessed_traits,
            syn_trait_method_sigs:     u.unprocessed_trait_method_sigs,
            syn_trait_method_defs:     u.unprocessed_trait_method_defs,
            syn_type_aliases:          u.unprocessed_type_aliases,
            syn_enums:                 u.unprocessed_enums,
            syn_unions:                u.unprocessed_unions,
        }
    }
}

// ─── from_unprocessed_with_files ─────────────────────────────────────────────
impl AllSynElements {
    pub fn from_unprocessed_with_files(
        u: AllUnprocessedElements,
        file_contents: &HashMap<FilePath, String>,
    ) -> Self {
        let resolve = |file: &FilePath| file_contents.get(file).map(String::as_str).unwrap_or("");

        AllSynElements {
            syn_attributes:            u.unprocessed_attributes
                                        .into_iter()
                                        .map(|raw| {
                                            let content = resolve(&raw.file);
                                            SynAttribute::from_unprocessed(raw, content)
                                        })
                                        .collect(),
            syn_methods:               u.unprocessed_methods.into_iter().map(SynMethod::from).collect(),
            syn_mods:                  u.unprocessed_mods.into_iter().map(SynMod::from).collect(),
            syn_expression_statements: u.unprocessed_expression_statements.into_iter().map(SynExpressionStatement::from).collect(),
            syn_use_declarations:      u.unprocessed_use_declarations.into_iter().map(SynUseDeclaration::from).collect(),
            syn_macro_definitions:     u.unprocessed_macro_definitions.into_iter().map(SynMacroDefinition::from).collect(),
            syn_macro_invocations:     u.unprocessed_macro_invocations.into_iter().map(SynMacroInvocation::from).collect(),
            syn_tests_mods:            u.unprocessed_tests_mods,
            syn_functions:             u.unprocessed_functions,
            syn_impls:                 u.unprocessed_impls,
            syn_structs:               u.unprocessed_structs,
            syn_traits:                u.unprocessed_traits,
            syn_trait_method_sigs:     u.unprocessed_trait_method_sigs,
            syn_trait_method_defs:     u.unprocessed_trait_method_defs,
            syn_type_aliases:          u.unprocessed_type_aliases,
            syn_enums:                 u.unprocessed_enums,
            syn_unions:                u.unprocessed_unions,
        }
    }
}

// ─── merge_adjacent_attributes ───────────────────────────────────────────────
impl AllSynElements {
    pub fn merge_adjacent_attributes(
        &mut self,
        file_contents: &HashMap<FilePath, String>,
    ) {
        let mut by_file: HashMap<FilePath, Vec<SynAttribute>> = HashMap::new();

        for attr in std::mem::take(&mut self.syn_attributes) {
            let content = file_contents.get(&attr.file).map(String::as_str).unwrap_or("");
            let syn_attr = SynAttribute::from_unprocessed(attr, content);
            by_file.entry(syn_attr.file.clone()).or_default().push(syn_attr);
        }

        self.syn_attributes = by_file
            .into_iter()
            .flat_map(|(file, mut attrs)| {
                let content = file_contents.get(&file).map(String::as_str).unwrap_or("");
                attrs.sort_by_key(|a| a.attribute_body.range.byte_range.start);

                let mut accumulated: Vec<SynAttribute> = Vec::with_capacity(attrs.len());
                for current in attrs {
                    if let Some(prev) = accumulated.last() {
                        let is_before = prev.attribute_body.before(&current.attribute_body);
                        let only_ws   = prev.attribute_body.only_whitespace_between(&current.attribute_body, content);
                        if is_before && only_ws {
                            let prev_owned = accumulated.pop().unwrap();
                            accumulated.push(prev_owned.merge_with(&current, content));
                            continue;
                        }
                    }
                    accumulated.push(current);
                }
                accumulated
            })
            .collect();
    }
}
