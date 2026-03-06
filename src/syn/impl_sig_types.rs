use std::collections::{HashMap, HashSet};
use serde::Deserialize;
use syntax_queries::RustParser;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TypeIdentifiers {
    /// mapping: type variable -> set of concrete types (kept in sorted order & unique)
    pub type_variables: Option<HashMap<String, HashSet<String>>>,

    /// list of concrete types encountered
    pub concrete_types: HashSet<String>,
}

///remove_ds_structure
impl TypeIdentifiers {
    /// Remove `ds_structure` from `concrete_types` and from every HashSet in `type_variables`.
    ///
    /// Returns the total number of removals performed.
    ///
    /// This will also remove any entries from `type_variables` whose HashSet becomes empty
    /// as a result of the removal. If the resulting map is empty it sets `type_variables` to `None`.
    pub fn remove_ds_structure(&mut self, ds_structure: &str) -> usize {
        let mut removed_count = 0;

        // Remove from concrete_types
        if self.concrete_types.remove(ds_structure) {
            removed_count += 1;
        }

        // If there's a HashMap inside the Option, mutate it
        if let Some(map) = self.type_variables.as_mut() {
            // Remove from each type variable's set
            for set in map.values_mut() {
                if set.remove(ds_structure) {
                    removed_count += 1;
                }
            }

            // Remove any keys whose set is now empty
            map.retain(|_, set| !set.is_empty());

            // If the map is empty, clear the Option to None (optional)
            if map.is_empty() {
                self.type_variables = None;
            }
        }

        removed_count
    }
}

///from_impl_parts
impl TypeIdentifiers {
    /// Parse trait bounds and const params out of `impl_signature` while removing the
    /// matched occurrences from `impl_signature`. Returns a TypeIdentifiers instance.
    pub fn from_impl_parts(
        mut all_types: Vec<String>,
        trait_bounds: Vec<String>,
        const_params: Vec<String>,
        impl_signature: &str,
    ) -> Self {
        // Work on a mutable owned copy of the impl signature so helper parsers can
        // remove occurrences in-place and subsequent parsers see the updated string.
        let mut impl_sig = impl_signature.to_string();

        // Do not consume `const_params` here — iterate by reference so we can use them later.
        let const_param_idents: HashSet<String> = const_params
            .iter()
            .filter_map(|p| ident_before_colon(p))
            .collect();

        // First parse trait bounds: this returns type variable idents and a map
        // of type variable -> concrete types (from trait bounds). It will also
        // remove the matched trait bound occurrences from `impl_sig`.
        let (type_var_idents, mut type_variables) = Self::parse_trait_bounds_from_signature(
            &mut impl_sig,
            trait_bounds,
            &const_param_idents,
        );

        // Next parse const params: this will add const param keys and their
        // capitalized concrete tokens, and remove const param occurrences from
        // `impl_sig` when found.
        let const_type_variables =
            Self::parse_const_params_from_signature(&mut impl_sig, &const_params);

        // Merge const param entries into the type_variables map (merge sets).
        for (k, set) in const_type_variables.into_iter() {
            let entry = type_variables.entry(k).or_insert_with(HashSet::new);
            for v in set.into_iter() {
                entry.insert(v);
            }
        }

        // Remove any type variables (discovered from trait_bounds) from the original Vec in-place (preserves order).
        // Note: const param idents were explicitly skipped above when detecting type variables,
        // so they would otherwise remain in `all_types`. We now also *exclude* them when building `concrete_types`.
        all_types.retain(|t| !type_var_idents.contains(t));

        // Convert the remaining types into a HashSet (unique, unordered),
        // excluding any const parameter identifiers so they don't appear as concrete types.
        let concrete_types: HashSet<String> = all_types
            .into_iter()
            .filter(|t| !const_param_idents.contains(t))
            .collect();

        // --- NEW: sanitize both concrete_types and type_variables (keys + values)
        // First, clean concrete types
        let mut cleaned_concrete_types: HashSet<String> = HashSet::new();
        for ct in concrete_types.into_iter() {
            let cleaned = Self::remove_angle_with_quote(&ct);
            if !cleaned.is_empty() {
                cleaned_concrete_types.insert(cleaned);
            }
        }

        // Next, clean type_variables keys and inner sets; merge duplicates that arise after cleaning
        let mut cleaned_type_variables: HashMap<String, HashSet<String>> = HashMap::new();
        for (key, set) in type_variables.into_iter() {
            let cleaned_key = Self::remove_angle_with_quote(&key);
            if cleaned_key.is_empty() {
                // skip keys that become empty after cleaning
                continue;
            }
            let entry = cleaned_type_variables
                .entry(cleaned_key)
                .or_insert_with(HashSet::new);
            for val in set.into_iter() {
                let cleaned_val = Self::remove_angle_with_quote(&val);
                if !cleaned_val.is_empty() {
                    entry.insert(cleaned_val);
                }
            }
        }
        // after pruning and producing a HashMap<String, HashSet<String>>
        let type_variables =
            prune_type_variables(cleaned_type_variables, cleaned_concrete_types.clone());

        // convert to Option<HashMap<_, _>>: None if empty
        let type_variables = if type_variables.is_empty() {
            None
        } else {
            Some(type_variables)
        };

        TypeIdentifiers {
            type_variables,
            concrete_types: cleaned_concrete_types,
        }
    }

    /// Parse trait bounds from `impl_sig` (mutable), remove each found bound occurrence
    /// from `impl_sig`, and return:
    /// - set of identified type variable idents (discovered from the trait bounds),
    /// - map of type variable -> set of concrete types (extracted from bounds).
    fn parse_trait_bounds_from_signature(
        impl_sig: &mut String,
        trait_bounds: Vec<String>,
        const_param_idents: &HashSet<String>,
    ) -> (HashSet<String>, HashMap<String, HashSet<String>>) {
        let mut type_var_idents: HashSet<String> = HashSet::new();
        let mut type_variables: HashMap<String, HashSet<String>> = HashMap::new();

        for bound in trait_bounds.into_iter() {
            // find the ident before this bound in the current impl signature
            if let Some(param) = find_ident_before(impl_sig, &bound) {
                if const_param_idents.contains(&param) {
                    // skip const params when enumerating trait_bounds
                    // (but we still may want to remove the occurrence of the bound if desired;
                    // keep this behavior consistent with the original comment and skip)
                    continue;
                }

                // remove the first occurrence of the bound from the impl signature so subsequent parsers see the updated signature
                // (use replacen to avoid removing duplicates unintentionally)
                *impl_sig = impl_sig.replacen(&bound, "", 1);

                // record that this param is a type variable (so it can be removed from all_types)
                type_var_idents.insert(param.clone());

                // extract concrete types from this bound, then insert them into the map
                let concrete_list = extract_concrete_types_from_bound(&bound);
                if !concrete_list.is_empty() {
                    let entry = type_variables
                        .entry(param.clone())
                        .or_insert_with(HashSet::new);
                    for ct in concrete_list {
                        entry.insert(ct);
                    }
                }
            }
        }

        (type_var_idents, type_variables)
    }

    /// Parse const parameters from `const_params` and ensure they become keys in the returned map.
    /// If a const parameter string occurs in `impl_sig`, remove that occurrence as well.
    /// Returns a map: const_param_ident -> set of capitalized tokens found after the colon.
    fn parse_const_params_from_signature(
        impl_sig: &mut String,
        const_params: &Vec<String>,
    ) -> HashMap<String, HashSet<String>> {
        let mut type_variables: HashMap<String, HashSet<String>> = HashMap::new();

        for p in const_params.iter() {
            if let Some(param_ident) = ident_before_colon(p) {
                // ensure the const param is present as a key (even if no concrete types found)
                let entry = type_variables
                    .entry(param_ident.clone())
                    .or_insert_with(HashSet::new);

                // If this const param pattern appears in the impl signature, remove that occurrence
                // so subsequent processing/parsing uses the updated signature.
                if impl_sig.contains(p) {
                    *impl_sig = impl_sig.replacen(p, "", 1);
                } else {
                    // try to find a looser match: e.g., the impl signature may contain "const N:" without default/value suffix,
                    // try to remove just the ident name if present to keep further parsing cleaner.
                    if let Some(pos) = impl_sig.find(&param_ident) {
                        // remove the identifier occurrence only (single occurrence)
                        let end = pos + param_ident.len();
                        impl_sig.replace_range(pos..end, "");
                    }
                }

                // collect capitalized tokens after the colon from the const param declaration and insert them into the set
                let caps = capitalized_after_colon(p);
                for ct in caps {
                    entry.insert(ct);
                }
            }
        }

        type_variables
    }
}

///remove_lifetimes
impl TypeIdentifiers {
    /// Remove any substring that is enclosed in `<` ... `>` (supports nesting) *if and only if*
    /// that substring contains a `'` (single-quote) somewhere inside. Examples:
    /// - `Foo<'a, T>` -> `Foo`
    /// - `Bar<T<'b>>` -> `Bar<T>` (if inner `< 'b' >` removed)
    /// - `Something< A<'x>, B>` -> `Something< A, B>` (then later pruning/normalization may remove extra whitespace)
    fn remove_angle_with_quote(s: &str) -> String {
        // Collect char indices for safe UTF-8 slicing
        let chars: Vec<(usize, char)> = s.char_indices().collect();
        let mut res = String::with_capacity(s.len());
        let mut idx: usize = 0;
        while idx < chars.len() {
            let (byte_i, ch) = chars[idx];
            if ch == '<' {
                // Try to find matching '>' with nesting
                let mut depth: usize = 1;
                let mut found_quote = false;
                let mut j = idx + 1;
                let mut end_byte: Option<usize> = None;
                while j < chars.len() {
                    let (b2, ch2) = chars[j];
                    if ch2 == '<' {
                        depth += 1;
                    } else if ch2 == '>' {
                        depth -= 1;
                        if depth == 0 {
                            // end byte index is the byte index + the width of '>'
                            end_byte = Some(b2 + ch2.len_utf8());
                            break;
                        }
                    } else if ch2 == '\'' {
                        found_quote = true;
                    }
                    j += 1;
                }

                if let Some(end_b) = end_byte {
                    // matched a `< ... >`
                    if !found_quote {
                        // keep the entire `<...>` substring as-is
                        res.push_str(&s[byte_i..end_b]);
                    }
                    // advance idx to the position after the matched '>'
                    idx = j + 1;
                    continue;
                } else {
                    // no matching '>' found: append the rest and break
                    res.push_str(&s[byte_i..]);
                    break;
                }
            } else {
                // append the single char
                res.push(ch);
                idx += 1;
            }
        }
        // Trim accidental whitespace introduced by removals
        // (optional but usually helpful for identifiers)
        let trimmed = res.trim().to_string();
        trimmed
    }
}

/// from impl signature
impl TypeIdentifiers {
    /// Create TypeIdentifiers from an impl signature and a vector of extracted type identifier tokens.
    /// This now calls `split_type_identifiers` directly (instead of delegating to `from_impl_parts`),
    /// and appends trait-bounds and const-params collected from the impl signature.
    pub fn from_impl_signature(impl_signature: &impl ToString) -> Self {
        // turn signature into owned string so we can pass &str to the collectors
        let sig = impl_signature.to_string();

        // collect trait bounds and const params from the signature
        let trait_bounds = TypeIdentifiers::collect_trait_bounds_from_impl(&sig);
        let const_params = TypeIdentifiers::collect_const_parameters_from_impl(&sig);

        // Ask RustParser to extract all `type_identifier` nodes from the signature
        let mut all_types: Vec<String> = Vec::new();
        match RustParser::new(&sig, "type_identifier") {
            Ok(parser) => {
                parser.save_type_identifiers(&mut all_types);
            }
            Err(err_str) => {
                eprintln!(
                    "RustParser::new failed for impl_signature while extracting type identifiers: {}",
                    err_str
                );
            }
        }

        // Use the unified from_impl_parts that accepts all_types and the signature
        TypeIdentifiers::from_impl_parts(all_types, trait_bounds, const_params, &sig)
    }
}

/// Return a set of all capitalized identifiers appearing after the first `:` in `param`.
///
/// - Tokens are runs of `[alphanumeric or _]` characters.
/// - A token is considered "capitalized" if its first character is uppercase (`char::is_uppercase`).
///
/// Examples:
/// - "x: Option<Result<Foo, Bar>>" -> {"Option", "Result", "Foo", "Bar"}
/// - "const N: usize" -> {}
pub fn capitalized_after_colon(param: &str) -> HashSet<String> {
    let colon_pos = match param.find(':') {
        Some(pos) => pos,
        None => return HashSet::new(),
    };

    let after = &param[colon_pos + 1..];
    let mut cur = String::new();
    let mut result = HashSet::new();

    for ch in after.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            cur.push(ch);
        } else {
            if !cur.is_empty() {
                if cur
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    result.insert(cur.clone());
                }
                cur.clear();
            }
        }
    }

    // final token
    if !cur.is_empty() {
        if cur
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
        {
            result.insert(cur);
        }
    }

    result
}

/// Extract concrete trait/type names from a trait-bound snippet.
/// Delimiters ignored: whitespace, comma ',' and plus '+'.
/// Example: ": Clone + Debug," -> vec!["Clone", "Debug"]
pub fn extract_concrete_types_from_bound(bound: &str) -> Vec<String> {
    // Remove leading colon (":") if present, then split on ',' or '+' and trim pieces.
    let s = bound.trim();
    let s = s.strip_prefix(':').unwrap_or(s).trim();

    s.split(|c: char| c == ',' || c == '+')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(strip_qualified_type)
        .collect()
}

/// Keep only concrete types that appear in `concrete_types`.
///
/// - `type_variables`: map from type-var name -> set of concrete types
/// - `concrete_types`: list of concrete types that are considered valid
///
/// Returns the filtered map. Any concrete type not present in `concrete_types`
/// is removed from each set. Entries with an empty set are removed from the map.
pub fn prune_type_variables(
    mut type_variables: HashMap<String, HashSet<String>>,
    concrete_types: HashSet<String>,
) -> HashMap<String, HashSet<String>> {
    // Convert concrete_types to a HashSet for O(1) membership checks
    let concrete_set: HashSet<String> = concrete_types.into_iter().collect();

    // For each HashSet in the map, retain only elements present in concrete_set
    for (_param, concrete_set_for_param) in type_variables.iter_mut() {
        concrete_set_for_param.retain(|t| concrete_set.contains(t));
    }

    // Optionally remove entries whose HashSet is now empty
    type_variables.retain(|_, set| !set.is_empty());

    type_variables
}

/// Standalone helper: given the impl signature and a pattern like ": Clone",
/// find the identifier immediately to the left of that pattern. Delimiters
/// considered are whitespace, comma, or '<'.
fn find_ident_before(impl_sig: &str, pattern: &str) -> Option<String> {
    if let Some(pos) = impl_sig.find(pattern) {
        // pos is the byte index where pattern starts
        if pos == 0 {
            return None;
        }
        let bytes = impl_sig.as_bytes();

        // Start scanning left from the byte just before `pattern`
        // First skip any whitespace to land at the last char of the identifier
        let mut i = pos;
        while i > 0 {
            let ch = bytes[i - 1] as char;
            if ch.is_whitespace() {
                i -= 1;
            } else {
                break;
            }
        }

        // Now scan left until we hit a delimiter (whitespace, comma, or '<')
        let mut start = i;
        while start > 0 {
            let ch = bytes[start - 1] as char;
            if ch.is_whitespace() || ch == ',' || ch == '<' || ch == '>' {
                break;
            }
            start -= 1;
        }

        // slice out the identifier
        let ident = impl_sig[start..i].trim();
        if ident.is_empty() {
            None
        } else {
            Some(ident.to_string())
        }
    } else {
        None
    }
}

/// Returns the identifier immediately before the first `:` in `param`.
/// Examples:
/// - "const N: usize" -> Some("N")
/// - "  K : u32"      -> Some("K")
/// - "no_colon"       -> None
pub fn ident_before_colon(param: &str) -> Option<String> {
    // find the first ':' (byte index)
    let colon_pos = param.find(':')?;

    let bytes = param.as_bytes();

    // scan left from the byte just before ':' skipping whitespace
    let mut i = colon_pos;
    while i > 0 {
        let ch = bytes[i - 1] as char;
        if ch.is_whitespace() {
            i -= 1;
        } else {
            break;
        }
    }

    if i == 0 {
        return None;
    }

    // now scan left to find the start of the identifier:
    // accept ASCII alphanumeric and underscore as identifier chars
    let mut start = i;
    while start > 0 {
        let ch = bytes[start - 1] as char;
        if ch.is_alphanumeric() || ch == '_' {
            start -= 1;
        } else {
            break;
        }
    }

    // slice and trim just in case
    let ident = param[start..i].trim();
    if ident.is_empty() {
        None
    } else {
        Some(ident.to_string())
    }
}

///collect trait bounds
impl TypeIdentifiers {
    /// Run RustParser on `impl_signature` with the "trait_bounds" query and
    /// return the tokens collected by `save_type_identifiers`.
    pub fn collect_trait_bounds_from_impl(impl_signature: &str) -> Vec<String> {
        match RustParser::new(impl_signature, "trait_bounds") {
            Ok(parser) => {
                let mut vec: Vec<String> = Vec::new();
                parser.save_type_identifiers(&mut vec);
                vec
            }
            Err(err_str) => {
                eprintln!(
                    "RustParser::new failed for trait_bounds on signature: {}: {}",
                    impl_signature, err_str
                );
                Vec::new()
            }
        }
    }
}

///collect const parameter
impl TypeIdentifiers {
    /// Run RustParser on `impl_signature` with the "const_parameter" query and
    /// return the tokens collected by `save_type_identifiers`.
    pub fn collect_const_parameters_from_impl(impl_signature: &str) -> Vec<String> {
        match RustParser::new(impl_signature, "const_parameter") {
            Ok(parser) => {
                let mut vec: Vec<String> = Vec::new();
                parser.save_type_identifiers(&mut vec);
                vec
            }
            Err(err_str) => {
                eprintln!(
                    "RustParser::new failed for const_parameter on signature: {}: {}",
                    impl_signature, err_str
                );
                Vec::new()
            }
        }
    }
}

fn strip_qualified_type(s: &str) -> String {
    let s = s.trim();
    s.rsplit("::").next().unwrap_or(s).to_string()
}

#[cfg(test)]
mod tests {
    use super::*; // adjust if MethodData / AstGrepMatch live in another module
    use std::collections::{HashMap, HashSet};

    #[test]
    fn from_impl_parts_splits_all_types_into_concrete_and_type_vars() {
        // same signature as your example
        let impl_signature = r#"impl<'a, T: Clone, N: usize> Array<T, N> where T: Debug,"#;

        // arrange tokens so that when an identifier is followed by `:` in the signature,
        // the next element in this Vec is the concrete type we expect to capture.
        // (We use "Debug" here as the concrete token to match expected normalization
        // used in your previous test.)
        let all_types: Vec<String> = vec![
            "T".into(), // -> "Clone"
            "Clone".into(),
            "N".into(),
            "Array".into(),
            "T".into(),
            "N".into(),
            "T".into(),
            "Debug".into(),
        ];

        // pass the trait bounds (as collected by your `collects_trait_bounds` test)
        let trait_bounds: Vec<String> = vec![": Clone".into(), ": Debug".into(), ": usize".into()];

        // mark `N` as a const param so it isn't treated as a concrete type
        let const_params: Vec<String> = vec!["AAA".into()];

        // Call the refactored function under test
        let identifiers =
            TypeIdentifiers::from_impl_parts(all_types, trait_bounds, const_params, impl_signature);

        // Build expected map:
        let mut expected: HashMap<String, HashSet<String>> = HashMap::new();
        let set_t = expected.entry("T".into()).or_insert_with(HashSet::new);
        set_t.insert("Clone".into());
        set_t.insert("Debug".into());

        // Expected concrete_types as a HashSet (order-agnostic, no duplicates)
        let expected_concrete: HashSet<String> = vec![
            "Clone".to_string(),
            "Array".to_string(),
            "Debug".to_string(),
        ]
        .into_iter()
        .collect();

        // Convert concrete_types to strings and collect into a HashSet for comparison.
        // This makes the test agnostic to insertion order and deduplicates automatically.
        let actual_concrete: HashSet<String> = identifiers
            .concrete_types
            .iter()
            .map(|t| t.to_string())
            .collect();

        assert_eq!(identifiers.type_variables, Some(expected));
        assert_eq!(actual_concrete, expected_concrete);
    }

    #[test]
    fn from_impl_parts_splits_all_types_into_concrete_and_type_vars_const() {
        // same signature as your example
        let impl_signature =
            r#"impl<'a, T: Clone, N: usize, const M: Usize> Array<T, N> where T: Debug,"#;

        // arrange tokens so that when an identifier is followed by `:` in the signature,
        // the next element in this Vec is the concrete type we expect to capture.
        // (We use "Debug" here as the concrete token to match expected normalization
        // used in your previous test.)
        let all_types: Vec<String> = vec![
            "T".into(), // -> "Clone"
            "Clone".into(),
            "N".into(),
            "M".into(),
            "Usize".into(),
            "Array".into(),
            "T".into(),
            "N".into(),
            "T".into(),
            "Debug".into(),
        ];

        // pass the trait bounds (as collected by your `collects_trait_bounds` test)
        let trait_bounds: Vec<String> = vec![
            ": Clone".into(),
            ": Usize".into(),
            ": Debug".into(),
            ": usize".into(),
        ];

        // mark `N` as a const param so it isn't treated as a concrete type
        let const_params: Vec<String> = vec!["const M: Usize".into()];

        // Call the refactored function under test
        let identifiers =
            TypeIdentifiers::from_impl_parts(all_types, trait_bounds, const_params, impl_signature);

        // Build expected map:
        let mut expected: HashMap<String, HashSet<String>> = HashMap::new();
        let set_t = expected.entry("T".into()).or_insert_with(HashSet::new);
        set_t.insert("Clone".into());
        set_t.insert("Debug".into());
        let set_n = expected.entry("M".into()).or_insert_with(HashSet::new);
        set_n.insert("Usize".into());

        // Expected concrete_types as a HashSet (order-agnostic, no duplicates)
        let expected_concrete: HashSet<String> = vec![
            "Clone".to_string(),
            "Array".to_string(),
            "Debug".to_string(),
            "Usize".to_string(),
        ]
        .into_iter()
        .collect();

        // Convert concrete_types to strings and collect into a HashSet for comparison.
        // This makes the test agnostic to insertion order and deduplicates automatically.
        let actual_concrete: HashSet<String> = identifiers
            .concrete_types
            .iter()
            .map(|t| t.to_string())
            .collect();

        assert_eq!(identifiers.type_variables, Some(expected));
        assert_eq!(actual_concrete, expected_concrete);
    }

    #[test]
    fn test_capitalized_after_colon_examples() {
        assert!(capitalized_after_colon("const N: usize").is_empty());
        assert!(capitalized_after_colon("  K : u32").is_empty());

        let s = "x: Option<Result<Foo, Bar>>";
        let got: HashSet<String> = capitalized_after_colon(s);
        let expected: HashSet<String> = ["Option", "Result", "Foo", "Bar"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(got, expected);

        // handle crate paths and generics
        let s2 = "m: ::std::collections::HashMap<String, Value>";
        let got2 = capitalized_after_colon(s2);
        let expect2: HashSet<String> = ["HashMap", "String", "Value"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(got2, expect2);

        // colon at start -> still finds tokens after colon
        let s3 = ":Bad";
        let got3 = capitalized_after_colon(s3);
        let expect3: HashSet<String> = ["Bad"].iter().map(|s| s.to_string()).collect();
        assert_eq!(got3, expect3);
    }

    #[test]
    fn test_ident_before_colon() {
        assert_eq!(ident_before_colon("const N: usize"), Some("N".to_string()));
        assert_eq!(ident_before_colon("  K : u32"), Some("K".to_string()));
        assert_eq!(ident_before_colon("no_colon_here"), None);
        assert_eq!(ident_before_colon(":bad"), None);
    }

    #[test]
    fn extract_concrete_types_simple_plus_comma() {
        let bound = ": Clone + Debug,";
        let types = extract_concrete_types_from_bound(bound);
        let set: HashSet<_> = types.into_iter().collect();

        let mut expected = HashSet::new();
        expected.insert("Clone".to_string());
        expected.insert("Debug".to_string());

        assert_eq!(set, expected);
    }

    #[test]
    fn extract_concrete_types_qualified_names() {
        let bound = ": std::fmt::Debug + core::marker::Send,";
        let types = extract_concrete_types_from_bound(bound);
        let set: HashSet<_> = types.into_iter().collect();

        let mut expected = HashSet::new();
        expected.insert("Debug".to_string());
        expected.insert("Send".to_string());

        assert_eq!(set, expected);
    }

    #[test]
    fn extract_concrete_types_empty_or_noise() {
        // only delimiters/whitespace should yield empty result
        assert!(extract_concrete_types_from_bound(": , + ").is_empty());
    }

    #[test]
    fn find_ident_before_clone() {
        let sig = r#"impl<'a, T: Clone, const N: usize> Array<T, N> where T: std::fmt::Debug,"#;
        let result = find_ident_before(sig, ": Clone");
        assert_eq!(result, Some("T".to_string()));
    }

    #[test]
    fn find_ident_before_debug() {
        let sig = r#"impl<'a, T: Clone, const N: usize> Array<T, N> where T: std::fmt::Debug,"#;
        let result = find_ident_before(sig, ": std::fmt::Debug");
        assert_eq!(result, Some("T".to_string()));
    }

    #[test]
    fn collects_trait_bounds_method() {
        // simple, predictable source with trait bounds in left-to-right order
        let source = r#"impl<'a, T: Clone, const N: usize> Array<T, N> where T: std::fmt::Debug,
"#;
        let expected = vec![": Clone".to_string(), ": std::fmt::Debug".to_string()];

        let found = TypeIdentifiers::collect_trait_bounds_from_impl(source);
        assert_eq!(found, expected);
    }

    #[test]
    fn collects_const_parameter_method() {
        // simple, predictable source with one const parameter
        let source = r#"impl<'a, T: Clone, const N: usize> Array<T, N> where T: std::fmt::Debug,
"#;
        let expected = vec!["const N: usize".to_string()];

        let found = TypeIdentifiers::collect_const_parameters_from_impl(source);
        assert_eq!(found, expected);
    }

    #[test]
    fn test_remove_ds_structure() {
        let mut ti = TypeIdentifiers::default();

        // fill concrete types
        ti.concrete_types.insert("List".into());
        ti.concrete_types.insert("Node".into());

        // prepare sets
        let mut set_t = HashSet::new();
        set_t.insert("List".to_string());
        set_t.insert("String".to_string());

        let mut set_u = HashSet::new();
        set_u.insert("Node".to_string());

        // Since type_variables is Option<HashMap<..>>, get_or_insert the inner map
        ti.type_variables
            .get_or_insert_with(HashMap::new)
            .insert("T".into(), set_t);

        ti.type_variables
            .get_or_insert_with(HashMap::new)
            .insert("U".into(), set_u);

        // Remove "List"
        let removed = ti.remove_ds_structure("List");
        assert_eq!(removed, 2); // removed from concrete_types and from T set

        assert!(!ti.concrete_types.contains("List"));

        // Check that the inner HashSet for "T" does NOT contain "List"
        assert!(
            !ti.type_variables
                .as_ref()
                .and_then(|m| m.get("T"))
                .map_or(false, |s| s.contains("List"))
        );

        // Remove "Node" — after removal U set becomes empty and should be removed entirely
        let removed2 = ti.remove_ds_structure("Node");
        assert_eq!(removed2, 2); // one removal from concrete_types and one from U set
        assert!(!ti.concrete_types.contains("Node"));

        // assert there is no "U" key in the inner map (or the whole Option is None)
        assert!(
            !ti.type_variables
                .as_ref()
                .map_or(false, |m| m.contains_key("U"))
        );
    }
}
