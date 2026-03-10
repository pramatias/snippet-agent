use std::io::Write;
use tempfile::NamedTempFile;

/// Example error type used in your original snippet.
/// If you already have one, keep using yours (ensure From<std::io::Error> is implemented).
#[derive(Debug)]
pub enum AstGrepRunError {
    Io(),
    Duct(),
    InvalidTempPath,
}

impl From<std::io::Error> for AstGrepRunError {
    fn from(_e: std::io::Error) -> Self {
        AstGrepRunError::Io()
    }
}

/// Writes the static YAML to a temp file, runs ast-grep against `directory`,
/// and returns the JSON stdout.
pub fn run_ast_grep_rule(directory: &str) -> Result<String, AstGrepRunError> {
    // create temp file
    let mut tmp: NamedTempFile = NamedTempFile::new()?; // maps to AstGrepRunError::Io via From

    // write the static YAML into it
    tmp.write_all(AST_GREP_RULE_YAML.as_bytes())?;
    // make sure contents are flushed to disk before external process reads
    tmp.as_file_mut().sync_all()?;

    // get path to the temp file
    let rule_path = tmp
        .path()
        .to_str()
        .ok_or(AstGrepRunError::InvalidTempPath)?
        .to_string();

    // run ast-grep; keep `tmp` in scope so the file still exists while command runs
    let stdout = duct::cmd(
        "ast-grep",
        ["scan", "--rule", &rule_path, directory, "--json"],
    )
    .read()
    .map_err(|_e| AstGrepRunError::Duct())?;

    Ok(stdout)
}

const AST_GREP_RULE_YAML: &str = r#"id: find-all-syntactic-elements
language: rust
rule:
  any:
    # ------------------------------------------------------------
    # IMPL BLOCK (contains methods with identifiers)
    # ------------------------------------------------------------
    - all:
        - kind: function_item
          pattern: $METHOD_BODY
        - inside:
            kind: impl_item
            pattern: $METHOD_IMPL_BODY
            stopBy: end
        - has:
            kind: identifier
            field: name
            pattern: $METHOD_NAME
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # STRUCT (with captured name)
    # ------------------------------------------------------------
    - all:
        - kind: struct_item
          pattern: $STRUCT_BODY
        - has:
            field: name
            pattern: $STRUCT_NAME
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # METHOD SIGNATURES INSIDE TRAITS (no body)
    # ------------------------------------------------------------
    - all:
        - kind: function_signature_item
          pattern: $TRAIT_METHOD_SIGNATURE
        - has:
            field: name
            kind: identifier
            pattern: $TRAIT_METHOD_SIGNATURE_NAME
        - inside:
            stopBy: end
            kind: trait_item
            pattern: $TRAIT_BODY_WITH_METHOD_SIGNATURE
            has:
              field: name
              kind: type_identifier
              pattern: $TRAIT_NAME_METHOD_SIGNATURE
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # METHOD DEFINITIONS INSIDE TRAITS (with body)
    # ------------------------------------------------------------
    - all:
        - kind: function_item
          pattern: $TRAIT_METHOD_BODY
        - has:
            field: name
            kind: identifier
            pattern: $TRAIT_METHOD_NAME
        - inside:
            stopBy: end
            kind: trait_item
            pattern: $TRAIT_BODY_WITH_METHOD
            has:
              field: name
              kind: type_identifier
              pattern: $TRAIT_NAME_WITH_METHOD
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # TRAIT DECLARATION (capture the trait itself)
    # ------------------------------------------------------------
    - all:
        - kind: trait_item
          pattern: $TRAIT_BODY
        - has:
            field: name
            kind: type_identifier
            pattern: $TRAIT_NAME
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # TYPE ALIAS (type_item with its identifier)
    # ------------------------------------------------------------
    - all:
        - kind: type_item
          pattern: $TYPE_ALIAS_BODY
        - has:
            kind: type_identifier
            field: name
            pattern: $TYPE_ALIAS_NAME
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # ENUM ITEM (capture the enum and its name)
    # ------------------------------------------------------------
    - all:
      - kind: enum_item
        pattern: $ENUM_BODY
      - has:
          kind: type_identifier
          pattern: $ENUM_NAME
      - inside:
          kind: source_file
          stopBy: end

    # ------------------------------------------------------------
    # UNION ITEM (capture the union and its name)
    # ------------------------------------------------------------
    - all:
      - kind: union_item
        pattern: $UNION_BODY
      - has:
          kind: type_identifier
          field: name
          pattern: $UNION_NAME
      - inside:
          kind: source_file
          stopBy: end

    # ------------------------------------------------------------
    # MOD ITEM with identifier "tests"
    # ------------------------------------------------------------
    - all:
        - kind: mod_item
          pattern: $TESTS_MOD
        - has:
            kind: identifier
            field: name
            regex: ^tests$
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # FREE / TOP-LEVEL FUNCTION (function_item)
    # ------------------------------------------------------------
    - all:
        - kind: function_item
          pattern: $FUNCTION_BODY
        - has:
            kind: identifier
            field: name
            pattern: $FUNCTION_NAME
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # ALL IMPL BLOCKS
    # ------------------------------------------------------------
    - all:
        - kind: impl_item
          pattern: $IMPL_BODY
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # ATTRIBUTE ITEMS
    # ------------------------------------------------------------
    - all:
        - kind: attribute_item
          pattern: $ATTRIBUTES
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # MOD ITEM (general — any named module)
    # ------------------------------------------------------------
    - all:
        - kind: mod_item
          pattern: $MOD_BODY
        - has:
            kind: identifier
            field: name
            pattern: $MOD_NAME
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # EXPRESSION STATEMENT
    # ------------------------------------------------------------
    - all:
        - kind: expression_statement
          pattern: $EXPRESSION_STATEMENT
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # USE DECLARATION
    # ------------------------------------------------------------
    - all:
        - kind: use_declaration
          pattern: $USE_DECLARATION
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # MACRO DEFINITION
    # ------------------------------------------------------------
    - all:
        - kind: macro_definition
          pattern: $MACRO_DEFINITION_BODY
        - has:
            kind: identifier
            field: name
            pattern: $MACRO_DEFINITION_NAME
        - inside:
            kind: source_file
            stopBy: end

    # ------------------------------------------------------------
    # MACRO INVOCATION
    # ------------------------------------------------------------
    - all:
        - kind: macro_invocation
          pattern: $MACRO_INVOCATION
        - inside:
            kind: expression_statement
            inside:
              kind: source_file

"#;
