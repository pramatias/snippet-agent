use anyhow::Result;

use crate::ast_grep_selection::*;
// use syntax_queries::RustParser;

/// MethodFind runs `ast-grep scan --rule <tempfile> <directory> --json` and
/// prints grouped-and-cleaned output similar to the jq pipeline you
/// provided.
pub struct MethodFind {
    pub directory: String,
    /// the method name to search (will be placed into the rule's regex)
    pub method: String,
    /// stored matches from the latest run
    pub matches: Vec<AstGrepMatch>,
}

/// Type aliases for clarity
pub type FilePath = String;
pub type MethodBody = String; // the full method text (first secondary)
pub type ImplBody = String; // the whole impl of the method (second secondary)
pub type MethodName = String; // the method identifier/name (third secondary)

/// The new, explicit AstGrepMatch with ordered fields and byte ranges
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AstGrepMatch {
    pub file: FilePath,
    /// the node's top-level text
    pub method_body: MethodBody,
    pub method_body_range: ByteRange,

    /// impl block text and its byte range
    pub impl_body: ImplBody,
    pub impl_body_range: ByteRange,

    /// method / function body text and its byte range
    pub method_name: MethodName,
    pub method_name_range: ByteRange,
}

/// new
impl MethodFind {
    /// Create a new MethodFind. `matches` starts empty.
    pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {
        MethodFind {
            directory: directory.into(),
            method: method.into(),
            matches: Vec::new(),
        }
    }
}

/// query
impl MethodFind {
    /// Run the full pipeline: find matches, run RustParser/delete_node on each
    /// match.method_body (target node = "block"), then print matches.
    pub fn query(&mut self) -> Result<(), AstGrepError> {
        // find matches (keeps previous behavior of returning early if none)
        let items = self.find_matches()?;
        if items.is_empty() {
            return Ok(());
        }

        // store found matches
        self.matches = items;

        Ok(())
    }
}

///order
impl AstGrepMatch {
    /// Ensure that `impl_body_range <= method_name_range`. If not, swap both
    /// the text fields and their corresponding ranges so `impl_body` is the one
    /// with the smaller (or equal) byte range.
    pub fn order_fields(&mut self) {
        if self.impl_body_range > self.method_name_range {
            // swap the textual fields
            std::mem::swap(&mut self.impl_body, &mut self.method_name);
            // swap the corresponding ranges
            std::mem::swap(&mut self.impl_body_range, &mut self.method_name_range);
        }
    }
}

/// find
impl MethodFind {
    /// Find matches, store them in `self.matches`, and return a deterministic Vec
    /// of AstGrepMatch items.
    pub fn find_matches(&mut self) -> Result<Vec<AstGrepMatch>, AstGrepError> {
        // Delegate running ast-grep + JSON parsing
        let selector_items: Vec<SelectorAstGrepMatch> =
            json_selectors_ast_grep(&self.method, &self.directory)?;

        let mut items: Vec<AstGrepMatch> = Vec::with_capacity(selector_items.len());

        for (_, selector) in selector_items.iter().enumerate() {

            // Try to read the first two secondary meta variables directly.
            let secondaries = &selector.meta_variables.multi.secondary;
            if secondaries.len() < 2 {
                // intentionally skip selectors without at least two secondaries
                continue;
            }

            // Use the first two secondary selectors (no pre-ordering here)
            let impl_secondary = &secondaries[0];
            let method_name_secondary = &secondaries[1];

            let impl_body = impl_secondary.text.clone();
            let impl_body_range = impl_secondary.range.byte_offset.clone();

            let method_name = method_name_secondary.text.clone();
            let method_name_range = method_name_secondary.range.byte_offset.clone();

            items.push(AstGrepMatch {
                file: selector.file.clone(),
                method_body: selector.text.clone(),
                method_body_range: selector.range.byte_offset.clone(),
                impl_body,
                impl_body_range,
                method_name,
                method_name_range,
            });
        }

        // Ensure the impl_body/ method_name ordering invariant for every match:
        for (_, item) in items.iter_mut().enumerate() {
            item.order_fields();
        }

        // deterministic ordering: file -> method_name
        items.sort_by(|a, b| a.file.cmp(&b.file).then(a.method_name.cmp(&b.method_name)));

        // store results in the struct and return them
        self.matches = items.clone();
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    #[test]
    fn test_methodfind_matches_contain_expected_fields() {
        // create temp dir and a test file (not used by the logic under test,
        // but useful to mimic real layout if needed)
        let temp_dir = tempdir().expect("failed to create tempdir");
        let rust_code = r#"
pub struct SomeTrait {
    pub directory: String,
    /// stored matches from the latest run
    pub matches: Vec<AstGrepMatch>,
}

impl TraitToImpl{
    /// Create a new MethodFind. `matches` starts empty.
    pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {}
}
impl TraitToImpl{
    /// Create a new MethodFind. `matches` starts empty.
    pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {}
    pub fn query(&mut self) -> Result<(), AstGrepError> {}
}
"#;
        let rust_file_path = temp_dir.path().join("test.rs");
        fs::write(&rust_file_path, rust_code).expect("failed to write test file");

        // Create a MethodFind (matches starts empty).
        let mut finder = MethodFind::new(temp_dir.path().to_string_lossy(), "");

        // Run the search so `matches` are produced, then ensure each match's fields are ordered.
        let mut matches = finder
            .find_matches()
            .expect("find_matches() failed in test");
        for m in matches.iter_mut() {
            m.order_fields();
        }

        // The exact strings we expect to find in at least one AstGrepMatch inside matches
        let expected_method_body =
            "pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {}";
        let expected_impl_body = "impl TraitToImpl{\n    /// Create a new MethodFind. `matches` starts empty.\n    pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {}\n    pub fn query(&mut self) -> Result<(), AstGrepError> {}\n}";
        let expected_method_name = "new";

        // Do NOT construct or push a new AstGrepMatch here.
        // Instead assert that `matches` already contains at least one matching object.
        let found = matches.iter().any(|m| {
            m.method_body == expected_method_body
            && m.impl_body == expected_impl_body
            && m.method_name == expected_method_name
            // optional extra sanity: the file points to our temporary test file
            && m.file.ends_with("test.rs")
        });

        assert!(
            found,
            "No AstGrepMatch found with the expected fields. Current matches: {:#?}",
            matches
        );
    }
    #[test]
    fn test_search_methodfind_matches_contain_expected_fields() {
        // create temp dir and a test file (not used by the logic under test,
        // but useful to mimic real layout if needed)
        let temp_dir = tempdir().expect("failed to create tempdir");
        let rust_code = r#"
pub struct SomeTrait {
    pub directory: String,
    /// stored matches from the latest run
    pub matches: Vec<AstGrepMatch>,
}

impl TraitToImpl{
    /// Create a new MethodFind. `matches` starts empty.
    pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {}
}
impl TraitToImpl{
    /// Create a new MethodFind. `matches` starts empty.
    pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {}
    pub fn query(&mut self) -> Result<(), AstGrepError> {}
}
"#;
        let rust_file_path = temp_dir.path().join("test.rs");
        fs::write(&rust_file_path, rust_code).expect("failed to write test file");

        // Create a MethodFind (matches starts empty).
        let mut finder = MethodFind::new(temp_dir.path().to_string_lossy(), "quer");

        // Run the search so `matches` are produced, then ensure each match's fields are ordered.
        let mut matches = finder
            .find_matches()
            .expect("find_matches() failed in test");
        for m in matches.iter_mut() {
            m.order_fields();
        }

        // The exact strings we expect to find in at least one AstGrepMatch inside matches
        let expected_method_body = "pub fn query(&mut self) -> Result<(), AstGrepError> {}";
        let expected_impl_body = "impl TraitToImpl{\n    /// Create a new MethodFind. `matches` starts empty.\n    pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {}\n    pub fn query(&mut self) -> Result<(), AstGrepError> {}\n}";
        let expected_method_name = "query";

        // Do NOT construct or push a new AstGrepMatch here.
        // Instead assert that `matches` already contains at least one matching object.
        let found = matches.iter().any(|m| {
            m.method_body == expected_method_body
            && m.impl_body == expected_impl_body
            && m.method_name == expected_method_name
            // optional extra sanity: the file points to our temporary test file
            && m.file.ends_with("test.rs")
        });

        assert!(
            found,
            "No AstGrepMatch found with the expected fields. Current matches: {:#?}",
            matches
        );
    }

    // helper to construct a SelectorByteOffsetRange
    fn sr(start: u64, end: u64) -> SelectorByteOffsetRange {
        SelectorByteOffsetRange { start, end }
    }

    #[test]
    fn order_fields_noop_when_already_ordered() {
        // impl_body_range is smaller than method_name_range => no swap expected
        let mut m = AstGrepMatch {
            file: "mod.rs".into(),
            method_body: "fn kept() {}".into(),
            method_body_range: sr(0, 10),
            impl_body: "impl Foo {}".into(),
            impl_body_range: sr(5, 20), // smaller
            method_name: "kept".into(),
            method_name_range: sr(30, 40), // larger
        };

        // remember original values
        let impl_before = m.impl_body.clone();
        let method_before = m.method_name.clone();
        let impl_range_before = m.impl_body_range.clone();
        let method_range_before = m.method_name_range.clone();

        m.order_fields();

        // nothing should have changed
        assert_eq!(m.impl_body, impl_before);
        assert_eq!(m.method_name, method_before);
        assert_eq!(m.impl_body_range, impl_range_before);
        assert_eq!(m.method_name_range, method_range_before);

        // ensure ordering guarantee still holds
        assert!(m.impl_body_range <= m.method_name_range);
    }

    #[test]
    fn order_fields_swaps_when_impl_is_after_method() {
        // impl_body_range is larger than method_name_range => swap expected
        let mut m = AstGrepMatch {
            file: "lib.rs".into(),
            method_body: "fn swapped() {}".into(),
            method_body_range: sr(0, 10),
            impl_body: "impl Later {}".into(),
            impl_body_range: sr(200, 300), // larger (appears later in file)
            method_name: "earlier".into(),
            method_name_range: sr(20, 30), // smaller
        };

        // capture original values to assert they are swapped
        let impl_before = m.impl_body.clone();
        let method_before = m.method_name.clone();
        let impl_range_before = m.impl_body_range.clone();
        let method_range_before = m.method_name_range.clone();

        m.order_fields();

        // textual fields should be swapped
        assert_eq!(m.impl_body, method_before);
        assert_eq!(m.method_name, impl_before);

        // ranges should be swapped too
        assert_eq!(m.impl_body_range, method_range_before);
        assert_eq!(m.method_name_range, impl_range_before);

        // final invariant: impl_body_range <= method_name_range
        assert!(m.impl_body_range <= m.method_name_range);
    }
}
