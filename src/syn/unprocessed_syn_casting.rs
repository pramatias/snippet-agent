//unprocessed_syn_casting.rs
use crate::json_selection::unprocessed_elements::*;
use crate::syn::syn_elements::*;
use crate::syn::syn_method::{extract_function_signature, extract_impl_signature};
use std::collections::HashMap;

///from_unprocessed
impl AllSynElements {
    pub fn from_unprocessed(
        u: AllUnprocessedElements,
        file_contents: &HashMap<String, String>,
    ) -> Self {
        Self {
            syn_attributes: build_syn_attributes(u.unprocessed_attributes, file_contents),

            // Pass-through collections (SynX = UnprocessedX type aliases)
            syn_tests_mods: u.unprocessed_tests_mods,
            syn_functions: u.unprocessed_functions,
            syn_impls: u.unprocessed_impls,
            syn_structs: u.unprocessed_structs,
            syn_traits: u.unprocessed_traits,
            syn_trait_method_sigs: u.unprocessed_trait_method_sigs,
            syn_trait_method_defs: u.unprocessed_trait_method_defs,
            syn_type_aliases: u.unprocessed_type_aliases,
            syn_enums: u.unprocessed_enums,
            syn_unions: u.unprocessed_unions,

            // Enriched collections
            syn_methods: u
                .unprocessed_methods
                .into_iter()
                .map(SynMethod::from_unprocessed)
                .collect(),

            syn_mods: u
                .unprocessed_mods
                .into_iter()
                .map(SynMod::from_unprocessed)
                .collect(),

            syn_expression_statements: u
                .unprocessed_expression_statements
                .into_iter()
                .map(SynExpressionStatement::from_unprocessed)
                .collect(),

            syn_use_declarations: u
                .unprocessed_use_declarations
                .into_iter()
                .map(SynUseDeclaration::from_unprocessed)
                .collect(),

            syn_macro_definitions: u
                .unprocessed_macro_definitions
                .into_iter()
                .map(SynMacroDefinition::from_unprocessed)
                .collect(),

            syn_macro_invocations: u
                .unprocessed_macro_invocations
                .into_iter()
                .map(SynMacroInvocation::from_unprocessed)
                .collect(),
        }
    }
}

macro_rules! impl_unprocessed_to_syn {
    ($from:ty => $to:ty [$($field:ident),+ $(,)?]) => {
        impl $to {
            pub fn from_unprocessed(u: $from) -> Self {
                Self {
                    file: u.file,
                    $($field: u.$field,)+
                }
            }
        }
    };
}

impl_unprocessed_to_syn!(UnprocessedMod                 => SynMod                 [mod_name, mod_body]);
impl_unprocessed_to_syn!(UnprocessedExpressionStatement => SynExpressionStatement [expression_body]);
impl_unprocessed_to_syn!(UnprocessedUseDeclaration      => SynUseDeclaration      [use_body]);
impl_unprocessed_to_syn!(UnprocessedMacroInvocation     => SynMacroInvocation     [invocation_body]);

///from unprocessed (field rename (macro_name → macro_definition_name))
impl SynMacroDefinition {
    pub fn from_unprocessed(u: UnprocessedMacroDefinition) -> Self {
        Self {
            file: u.file,
            macro_definition_name: u.macro_name,
            macro_definition_body: u.macro_body,
        }
    }
}

///from_unprocessed
impl SynMethod {
    pub fn from_unprocessed(u: UnprocessedMethod) -> Self {
        let impl_sig = extract_impl_signature(&u.impl_body);
        let function_sig = extract_function_signature(&u.method_body);
        let (type_ids, ds_name) = TypeIdentifiers::from_impl_signature(&impl_sig);
        let ds = ds_name.unwrap_or_default();

        Self {
            file: u.file,
            impl_body: u.impl_body,
            method_body: u.method_body,
            method_name: u.method_name,
            impl_signature: impl_sig,
            function_signature: function_sig,
            ds_structure: ds,
            type_identifiers: type_ids,
        }
    }
}

///from_unprocessed
impl SynAttribute {
    pub fn from_unprocessed(u: UnprocessedAttribute, context: &str) -> Self {
        Self {
            file: u.file,
            attribute_body: u.attribute_body,
            context_lines: context.to_string(),
        }
    }
}

// ── Attribute merging helper ──────────────────────────────────────────────────
//
// Consecutive attributes that are separated only by whitespace are merged into
// a single SynAttribute so that multi-line attribute blocks (e.g. a `#[derive]`
// followed by `#[serde(...)]`) are treated as one unit downstream.
fn build_syn_attributes(
    raw: Vec<UnprocessedAttribute>,
    file_contents: &HashMap<String, String>,
) -> SynAttributes {
    let mut accumulated: SynAttributes = Vec::new();
    let mut pending: Option<SynAttribute> = None;

    for unprocessed in raw {
        let content = file_contents
            .get(&unprocessed.file)
            .map(String::as_str)
            .unwrap_or("");

        let current = SynAttribute::from_unprocessed(unprocessed, content);

        match pending.take() {
            None => {
                pending = Some(current);
            }
            Some(prev_owned) => {
                if prev_owned
                    .attribute_body
                    .only_whitespace_between(&current.attribute_body, content)
                {
                    // Adjacent — keep accumulating into a merged attribute.
                    pending = Some(prev_owned.merge_with(&current, content));
                } else {
                    // Gap found — flush the previous group, start a new one.
                    accumulated.push(prev_owned);
                    pending = Some(current);
                }
            }
        }
    }

    if let Some(last) = pending {
        accumulated.push(last);
    }

    accumulated
}
