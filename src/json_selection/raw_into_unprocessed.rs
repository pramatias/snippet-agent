use crate::ast_grep::ast_grep_everything_selection::extract_selections_from_ast_grep_json;
use crate::json_selection::unprocessed_elements::*;

/// Converts a raw extracted value (Option<Vec<_>>) into a typed Vec
/// by unwrapping, iterating, and mapping Into::into.
///
/// Usage:
///   collect_unprocessed!(source_var => TargetType)
macro_rules! collect_unprocessed {
    ($source:expr => $target_type:ty) => {
        $source
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect::<$target_type>()
    };
}

///from_json
impl AllUnprocessedElements {
    pub fn from_raw_json(json: &str) -> Result<Self, serde_json::Error> {
        let (
            raw_attributes,
            raw_tests_mods,
            raw_functions,
            raw_methods,
            raw_impls,
            raw_structs,
            raw_traits,
            raw_trait_method_sigs,
            raw_trait_method_defs,
            raw_type_aliases,
            raw_enums,
            raw_unions,
        ) = extract_selections_from_ast_grep_json(json)?;

        Ok(AllUnprocessedElements {
            unprocessed_attributes:        collect_unprocessed!(raw_attributes        => UnprocessedAttributes),
            unprocessed_tests_mods:        collect_unprocessed!(raw_tests_mods        => UnprocessedTestsMods),
            unprocessed_functions:         collect_unprocessed!(raw_functions         => UnprocessedFunctions),
            unprocessed_methods:           collect_unprocessed!(raw_methods           => UnprocessedMethods),
            unprocessed_impls:             collect_unprocessed!(raw_impls             => UnprocessedImpls),
            unprocessed_structs:           collect_unprocessed!(raw_structs           => UnprocessedStructs),
            unprocessed_traits:            collect_unprocessed!(raw_traits            => UnprocessedTraits),
            unprocessed_trait_method_sigs: collect_unprocessed!(raw_trait_method_sigs => UnprocessedTraitMethodSigs),
            unprocessed_trait_method_defs: collect_unprocessed!(raw_trait_method_defs => UnprocessedTraitMethodDefs),
            unprocessed_type_aliases:      collect_unprocessed!(raw_type_aliases      => UnprocessedTypeAliases),
            unprocessed_enums:             collect_unprocessed!(raw_enums             => UnprocessedEnums),
            unprocessed_unions:            collect_unprocessed!(raw_unions            => UnprocessedUnions),
        })
    }
}
