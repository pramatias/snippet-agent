use anyhow::Result;
use serde::Deserialize;

use std::cmp::Ordering;
use std::io::Write;
use tempfile::NamedTempFile;
use thiserror::Error;

/// Resulting byte-range we expose (simple copy of SelectorByteOffsetRange)
pub type ByteRange = SelectorByteOffsetRange;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorByteOffsetRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorRange {
    pub byte_offset: SelectorByteOffsetRange,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorSecondary {
    pub text: String,
    pub range: SelectorRange,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorMulti {
    pub secondary: Vec<SelectorSecondary>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorMetaVariables {
    pub multi: SelectorMulti,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorAstGrepMatch {
    pub file: String,
    pub text: String,
    pub range: SelectorRange,
    pub meta_variables: SelectorMetaVariables,
}

impl PartialEq for SelectorByteOffsetRange {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}
impl Eq for SelectorByteOffsetRange {}

impl PartialOrd for SelectorByteOffsetRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SelectorByteOffsetRange {
    fn cmp(&self, other: &Self) -> Ordering {
        // Primary by `start`, tie-break by `end`
        self.start
            .cmp(&other.start)
            .then_with(|| self.end.cmp(&other.end))
    }
}

/// New error type specifically for the ast-grep runner / JSON parsing functions.
#[derive(Debug, Error)]
pub enum AstGrepError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),

    // Keep Duct as a String so we can preserve display text from the duct error.
    #[error("execution error from `duct`: {0}")]
    Duct(String),

    #[error("failed to parse JSON from ast-grep: {0}")]
    Json(#[from] serde_json::Error),

    #[error("temporary rule file path is not valid UTF-8")]
    InvalidTempPath,
}

/// Encapsulates running ast-grep and parsing its JSON output
/// into a Vec<SelectorAstGrepMatch>. Returns AstGrepError.
pub fn json_selectors_ast_grep(
    method: &str,
    directory: &str,
) -> Result<Vec<SelectorAstGrepMatch>, AstGrepError> {
    let stdout = run_ast_grep_rule(method, directory)?;

    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    // parse JSON array
    // If any required field is missing or null, serde_json will return an error here.
    let items = serde_json::from_str::<Vec<SelectorAstGrepMatch>>(trimmed)?;
    Ok(items)
}

/// Run ast-grep with a temporary YAML rule file and return stdout as String.
/// Returns AstGrepError instead of MethodFindError.
pub fn run_ast_grep_rule(method: &str, directory: &str) -> Result<String, AstGrepError> {
    // Rule template lives here now
    let rule_template = r#"id: find-foo-both
language: rust
rule:
  any:
    - # match the impl block that contains the target method
      kind: impl_item
      has:
        kind: function_item
        has:
          kind: identifier
          field: name
          regex: '^<METHOD>'
    - # match the method itself (inside an impl)
      all:
        - kind: function_item
        - inside:
            kind: impl_item
            stopBy: end
        - has:
            kind: identifier
            field: name
            regex: '^<METHOD>'
"#;

    // escape & prepare regex
    let escaped = regex::escape(method);
    let regex_pattern = format!("^{}", escaped);
    let yaml = rule_template.replace("<METHOD>", &regex_pattern);

    // write temp rule file
    let mut tmp: NamedTempFile = NamedTempFile::new()?; // <-- maps to AstGrepError::Io
    tmp.write_all(yaml.as_bytes())?;
    let rule_path = tmp
        .path()
        .to_str()
        .ok_or(AstGrepError::InvalidTempPath)?
        .to_string();

    // run ast-grep
    let stdout = duct::cmd(
        "ast-grep",
        ["scan", "--rule", &rule_path, directory, "--json"],
    )
    .read()
    .map_err(|e| AstGrepError::Duct(e.to_string()))?;

    Ok(stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn test_json_selectors_ast_grep() {
        use super::*;
        use std::fs;
        // use std::path::Path;

        // Create a temporary directory and write a Rust source file that contains
        // the exact items we want ast-grep to pick up.
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

        let rust_code = r#"
pub trait TraitToImpl {
    // trait items (if any)
}

/// Create a new MethodFind. `matches` starts empty.
impl TraitToImpl {
    /// Create a new MethodFind. `matches` starts empty.
    pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {}
    pub fn query(&mut self) -> Result<(), AstGrepError> {}
}

// Another unrelated function
pub fn helper_new() {
    // just to have another occurrence of the token "new"
}
"#;

        let rust_file_path = temp_dir.path().join("test.rs");
        fs::write(&rust_file_path, rust_code).expect("Failed to write test Rust file");

        // Call the function under test with empty method string.
        let matches = json_selectors_ast_grep("", temp_dir.path().to_str().unwrap())
            .expect("json_selectors_ast_grep should succeed");

        // Expectation 1: one of the top-level `text` fields equals this exact signature.
        let expected_text1 =
            "pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {}";
        let found_text1 = matches.iter().any(|m| m.text == expected_text1);
        assert!(
            found_text1,
            "Expected to find a top-level SelectorAstGrepMatch.text equal to `{}`. Got: {:#?}",
            expected_text1, matches
        );

        // Expectation 2: one of the secondary.text fields equals the full impl block text
        // (exact whitespace and newlines must match what the AST-grep returns).
        let expected_secondary_full = "impl TraitToImpl {\n    /// Create a new MethodFind. `matches` starts empty.\n    pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {}\n    pub fn query(&mut self) -> Result<(), AstGrepError> {}\n}";

        let mut found_secondary_full = false;
        let mut found_secondary_new = false;

        for m in &matches {
            for sec in &m.meta_variables.multi.secondary {
                if sec.text == expected_secondary_full {
                    found_secondary_full = true;
                }
                if sec.text == "new" {
                    found_secondary_new = true;
                }
            }
        }

        assert!(
            found_secondary_full,
            "Expected to find a secondary.text equal to the impl block (with both methods). Got: {:#?}",
            matches
        );

        // Expectation 3: another secondary text should be the identifier "new"
        assert!(
            found_secondary_new,
            "Expected to find a secondary.text equal to `new`. Got: {:#?}",
            matches
        );

        // If you want to assert additional exact top-level texts, add them here similarly.
    }

    #[test]
    fn order_by_start() {
        let a = SelectorByteOffsetRange { start: 0, end: 5 };
        let b = SelectorByteOffsetRange { start: 10, end: 15 };

        // primary ordering is by `start`
        assert!(a < b);
        assert_eq!(a.cmp(&b), Ordering::Less);
        assert_eq!(b.cmp(&a), Ordering::Greater);
    }

    #[test]
    fn tie_break_by_end() {
        let a = SelectorByteOffsetRange { start: 0, end: 5 };
        let b = SelectorByteOffsetRange { start: 0, end: 6 };

        // same start: compare by `end`
        assert!(a < b);
        assert_eq!(a.cmp(&b), Ordering::Less);
        // equality case
        let c = SelectorByteOffsetRange { start: 0, end: 5 };
        assert_eq!(a, c);
        assert_eq!(a.cmp(&c), Ordering::Equal);
    }
}
