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

/// Put your rule YAML here (static string)
const AST_GREP_RULE_YAML: &str = r#"id: find-all-syntactic-elements
language: rust
rule:
  any:
    # ------------------------------------------------------------
    # IMPL BLOCK (contains methods with identifiers)
    # ------------------------------------------------------------
    # Matches an `impl_item` that contains one or more `function_item` nodes
    # (i.e., methods). Each matched function_item must contain an `identifier`
    # node for the method name.
    #
    # Captures:
    # - $METHOD_BODY     : each matched function_item node inside the impl
    # - $METHOD_IMPL_BODY: the matched impl_item node (the whole impl block)
    # - $METHOD_NAME     : the identifier node for the method's name
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

    # ------------------------------------------------------------
    # STRUCT (with captured name)
    # ------------------------------------------------------------
    # Matches a `struct_item` and captures:
    # - $STRUCT_BODY     : the entire struct_item node (the declaration + body)
    # - $STRUCT_NAME     : the identifier node that holds the struct's name
    # ------------------------------------------------------------
    - all:
        - kind: struct_item
          pattern: $STRUCT_BODY
        - has:
            field: name
            pattern: $STRUCT_NAME

    # ------------------------------------------------------------
    # METHOD SIGNATURES INSIDE TRAITS (no body)
    # ------------------------------------------------------------
    # Matches `function_signature_item` nodes (method signatures inside traits — declaration only).
    # Captures:
    # - $TRAIT_METHOD_SIGNATURE            : the function_signature_item node (the signature, no body)
    # - $TRAIT_METHOD_SIGNATURE_NAME       : the identifier node for the function name inside the signature
    # - $TRAIT_BODY_WITH_METHOD_SIGNATURE  : the enclosing trait_item node (the trait that contains the signature)
    # - $TRAIT_NAME_METHOD_SIGNATURE       : the trait's identifier (type_identifier)
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

    # ------------------------------------------------------------
    # METHOD DEFINITIONS INSIDE TRAITS (with body)
    # ------------------------------------------------------------
    # Matches `function_item` nodes (method definitions inside traits — include body).
    # Captures:
    # - $TRAIT_METHOD_BODY            : the function_item node (a method definition with a body)
    # - $TRAIT_METHOD_NAME            : the identifier node for the function name of the method
    # - $TRAIT_BODY_WITH_METHOD       : the enclosing trait_item node (the trait that contains the method)
    # - $TRAIT_NAME_WITH_METHOD       : the trait's identifier (type_identifier)
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

    # ------------------------------------------------------------
    # TRAIT DECLARATION (capture the trait itself)
    # ------------------------------------------------------------
    # Matches a `trait_item` and captures:
    # - $TRAIT_BODY      : the whole trait declaration node
    # - $TRAIT_NAME      : the name of the trait (a type_identifier)
    # ------------------------------------------------------------
    - all:
        - kind: trait_item
          pattern: $TRAIT_BODY
        - has:
            field: name
            kind: type_identifier
            pattern: $TRAIT_NAME

    # ------------------------------------------------------------
    # TYPE ALIAS (type_item with its identifier)
    # ------------------------------------------------------------
    # Matches `type_item` (e.g. `type Foo = ...;`) and captures:
    # - $TYPE_ALIAS_BODY : the whole type_item node
    # - $TYPE_ALIAS_NAME : the identifier node (type_identifier) for the alias name
    # ------------------------------------------------------------
    - all:
        - kind: type_item
          pattern: $TYPE_ALIAS_BODY
        - has:
            kind: type_identifier
            field: name
            pattern: $TYPE_ALIAS_NAME

    # ------------------------------------------------------------
    # ENUM ITEM (capture the enum and its name)
    # ------------------------------------------------------------
    # Matches an `enum_item` node and captures:
    # - $ENUM_BODY       : the entire enum declaration node (the whole `enum ... { ... }`)
    # - $ENUM_NAME       : the identifier node containing the enum's name
    # ------------------------------------------------------------
    - all:
      - kind: enum_item
        pattern: $ENUM_BODY
      - has:
          kind: type_identifier
          pattern: $ENUM_NAME

    # ------------------------------------------------------------
    # UNION ITEM (capture the union and its name)
    # ------------------------------------------------------------
    # Matches a `union_item` node and captures:
    # - $UNION_BODY      : the entire union declaration node
    # - $UNION_NAME      : the identifier node containing the union's name
    # ------------------------------------------------------------
    - all:
      - kind: union_item
        pattern: $UNION_BODY
      - has:
          kind: type_identifier
          field: name
          pattern: $UNION_NAME

    # ------------------------------------------------------------
    # MOD ITEM with identifier "tests"
    # ------------------------------------------------------------
    # Matches a `mod_item` where the module name is exactly "tests"
    # Captures:
    # - $TESTS_MOD       : the entire mod_item node
    # ------------------------------------------------------------
    - all:
        - kind: mod_item
          pattern: $TESTS_MOD
        - has:
            kind: identifier
            field: name
            regex: ^tests$

    # ------------------------------------------------------------
    # FREE / TOP-LEVEL FUNCTION (function_item)
    # ------------------------------------------------------------
    # Matches any `function_item` (free function or associated function)
    # that has an `identifier` for the function name.
    #
    # Captures:
    # - $FUNCTION_BODY   : the whole function_item node
    # - $FUNCTION_NAME   : the identifier node for the function's name
    # ------------------------------------------------------------
    - all:
        - kind: function_item
          pattern: $FUNCTION_BODY
        - has:
            kind: identifier
            field: name
            pattern: $FUNCTION_NAME

    # ------------------------------------------------------------
    # ALL IMPL BLOCKS
    # ------------------------------------------------------------
    # Matches every `impl_item`. Captures:
    # - $IMPL_BODY : the whole impl_item node
    # ------------------------------------------------------------
    - all:
        - kind: impl_item
          pattern: $IMPL_BODY

    # ------------------------------------------------------------
    # ATTRIBUTE ITEMS
    # ------------------------------------------------------------
    # Matches attribute nodes such as `#[derive(...)]`, `#![no_std]`, or inner
    # attributes. Captures:
    # - $ATTRIBUTES      : the attribute_item node(s)
    # ------------------------------------------------------------
    - all:
        - kind: attribute_item
          pattern: $ATTRIBUTES

    # ------------------------------------------------------------
    # MOD ITEM (general — any named module)
    # ------------------------------------------------------------
    # Captures:
    # - $MOD_BODY : the entire mod_item node
    # - $MOD_NAME : the identifier node for the module name
    # ------------------------------------------------------------
    - all:
        - kind: mod_item
          pattern: $MOD_BODY
        - has:
            kind: identifier
            field: name
            pattern: $MOD_NAME

    # ------------------------------------------------------------
    # EXPRESSION STATEMENT
    # ------------------------------------------------------------
    # Captures:
    # - $EXPRESSION_STATEMENT : the expression_statement node
    # ------------------------------------------------------------
    - all:
        - kind: expression_statement
          pattern: $EXPRESSION_STATEMENT

    # ------------------------------------------------------------
    # USE DECLARATION
    # ------------------------------------------------------------
    # Captures:
    # - $USE_DECLARATION : the use_declaration node
    # ------------------------------------------------------------
    - all:
        - kind: use_declaration
          pattern: $USE_DECLARATION

    # ------------------------------------------------------------
    # MACRO DEFINITION
    # ------------------------------------------------------------
    # Captures:
    # - $MACRO_DEFINITION_BODY : the entire macro_definition node
    # - $MACRO_DEFINITION_NAME : the identifier node for the macro name
    # ------------------------------------------------------------
    - all:
        - kind: macro_definition
          pattern: $MACRO_DEFINITION_BODY
        - has:
            kind: identifier
            field: name
            pattern: $MACRO_DEFINITION_NAME

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
