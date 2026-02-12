use crate::json_selection::raw_elements::*;
use serde::Deserialize;
use std::collections::HashMap;

/// The shape of a meta-variable item in ast-grep output:
/// { "text": "...", "range": { ... } }
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MetaVarItem {
    text: String,
    range: Range,
}

/// Only parse the parts of the ast-grep JSON we need.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMatch {
    file: String,
    meta_variables: Option<MetaVariables>,
}

/// `metaVariables` -> `single` in the JSON is a map of names -> MetaVarItem
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetaVariables {
    single: Option<HashMap<String, MetaVarItem>>,
    // we ignore `multi` / `transformed` for this extractor
}

fn extract_enums(
    file: &str,
    single_map: &HashMap<String, MetaVarItem>,
) -> Option<Vec<EnumSelection>> {
    // Best case: ENUM_BODY + ENUM_NAME
    if let (Some(body), Some(name)) = (single_map.get("ENUM_BODY"), single_map.get("ENUM_NAME")) {
        return Some(vec![EnumSelection {
            file: file.to_string(),
            enum_body_text: body.text.clone(),
            enum_body_range: body.range.clone(),
            enum_name_text: name.text.clone(),
            enum_name_range: name.range.clone(),
        }]);
    }

    None
}

fn extract_unions(
    file: &str,
    single_map: &HashMap<String, MetaVarItem>,
) -> Option<Vec<UnionSelection>> {
    // Best case: UNION_BODY + UNION_NAME
    if let (Some(body), Some(name)) = (single_map.get("UNION_BODY"), single_map.get("UNION_NAME")) {
        return Some(vec![UnionSelection {
            file: file.to_string(),
            union_body_text: body.text.clone(),
            union_body_range: body.range.clone(),
            union_name_text: name.text.clone(),
            union_name_range: name.range.clone(),
        }]);
    }

    None
}

/// Extract TRAIT method definitions (with bodies) as captured by: TRAIT_METHOD_BODY - TRAIT_METHOD_NAME - TRAIT_BODY_WITH_METHOD - TRAIT_NAME_WITH_METHOD
fn extract_trait_methods(
    file: &str,
    single_map: &HashMap<String, MetaVarItem>,
) -> Option<Vec<TraitMethodDefinitionSelection>> {
    // Best case: method body + method name (and optional enclosing trait info)
    if let (Some(method_body), Some(method_name)) = (
        single_map.get("TRAIT_METHOD_BODY"),
        single_map.get("TRAIT_METHOD_NAME"),
    ) {
        // If the trait body is present use it; otherwise use an empty string and
        // fall back to the method's range (so we still provide a concrete Range).
        let (trait_body_text, trait_body_range) =
            if let Some(tb) = single_map.get("TRAIT_BODY_WITH_METHOD") {
                (tb.text.clone(), tb.range.clone())
            } else {
                (String::new(), method_body.range.clone())
            };

        // Similarly for trait name: fall back to empty text and the method name's range.
        let (trait_name_text, trait_name_range) =
            if let Some(tn) = single_map.get("TRAIT_NAME_WITH_METHOD") {
                (tn.text.clone(), tn.range.clone())
            } else {
                (String::new(), method_name.range.clone())
            };

        return Some(vec![TraitMethodDefinitionSelection {
            file: file.to_string(),
            trait_body_text,
            trait_body_range,
            method_body_text: method_body.text.clone(),
            method_body_range: method_body.range.clone(),
            method_name_text: method_name.text.clone(),
            method_name_range: method_name.range.clone(),
            trait_name_text,
            trait_name_range,
        }]);
    }

    None
}

/// Extract TYPE ALIAS captures: TYPE_ALIAS_BODY - TYPE_ALIAS_NAME
fn extract_type_aliases(
    file: &str,
    single_map: &HashMap<String, MetaVarItem>,
) -> Option<Vec<TypeAliasSelection>> {
    if let (Some(body), Some(name)) = (
        single_map.get("TYPE_ALIAS_BODY"),
        single_map.get("TYPE_ALIAS_NAME"),
    ) {
        return Some(vec![TypeAliasSelection {
            file: file.to_string(),
            body_text: body.text.clone(),
            body_range: body.range.clone(),
            name_text: name.text.clone(),
            name_range: name.range.clone(),
        }]);
    }

    None
}

/// Extract method selections (METHOD_BODY + METHOD_NAME)
fn extract_methods(
    file: &str,
    single_map: &HashMap<String, MetaVarItem>,
) -> Option<Vec<MethodSelection>> {
    // Best case: METHOD_BODY + METHOD_NAME (normal case)
    if let (Some(body), Some(name)) = (single_map.get("METHOD_BODY"), single_map.get("METHOD_NAME"))
    {
        // Provide concrete (non-Option) values for MethodSelection fields.
        let (impl_text, impl_range) = match single_map.get("METHOD_IMPL_BODY") {
            Some(impl_body) => (impl_body.text.clone(), impl_body.range.clone()),
            // Default when no impl body: empty text and use body.range as a fallback.
            None => (String::new(), body.range.clone()),
        };

        return Some(vec![MethodSelection {
            file: file.to_string(),
            impl_text,
            impl_range,
            body_text: body.text.clone(),
            body_range: body.range.clone(),
            name_text: name.text.clone(),
            name_range: name.range.clone(),
        }]);
    }

    None
}

/// Extract trait declarations (TRAIT_BODY + TRAIT_NAME)
fn extract_traits(
    file: &str,
    single_map: &HashMap<String, MetaVarItem>,
) -> Option<Vec<TraitSelection>> {
    if let (Some(body), Some(name)) = (single_map.get("TRAIT_BODY"), single_map.get("TRAIT_NAME")) {
        return Some(vec![TraitSelection {
            file: file.to_string(),
            trait_body_text: body.text.clone(),
            trait_body_range: body.range.clone(),
            trait_name_text: name.text.clone(),
            trait_name_range: name.range.clone(),
        }]);
    }

    None
}

fn extract_trait_method_signatures(
    file: &str,
    single_map: &HashMap<String, MetaVarItem>,
) -> Option<Vec<TraitMethodSignatureSelection>> {
    let sig = single_map.get("TRAIT_METHOD_SIGNATURE")?;
    let sig_name = single_map.get("TRAIT_METHOD_SIGNATURE_NAME")?;
    let enclosing = single_map.get("TRAIT_BODY_WITH_METHOD_SIGNATURE")?;
    let trait_name = single_map.get("TRAIT_NAME_METHOD_SIGNATURE")?;

    let sel = TraitMethodSignatureSelection {
        file: file.to_string(),
        signature_text: sig.text.clone(),
        signature_range: sig.range.clone(),
        signature_name_text: sig_name.text.clone(),
        signature_name_range: sig_name.range.clone(),
        enclosing_trait_text: enclosing.text.clone(),
        enclosing_trait_range: enclosing.range.clone(),
        trait_name_text: trait_name.text.clone(),
        trait_name_range: trait_name.range.clone(),
    };

    Some(vec![sel])
}

/// Extract attribute selections from a single-map (if present)
fn extract_attributes(
    file: &str,
    single_map: &HashMap<String, MetaVarItem>,
) -> Option<Vec<AttributeSelection>> {
    single_map.get("ATTRIBUTES").map(|attr| {
        vec![AttributeSelection {
            file: file.to_string(),
            attribute: attr.text.clone(),
            range: attr.range.clone(),
        }]
    })
}

/// Extract tests mod selections from a single-map (if present)
fn extract_tests_mods(
    file: &str,
    single_map: &HashMap<String, MetaVarItem>,
) -> Option<Vec<TestsModSelection>> {
    single_map.get("TESTS_MOD").map(|tmod| {
        vec![TestsModSelection {
            file: file.to_string(),
            tests_mod: tmod.text.clone(),
            range: tmod.range.clone(),
        }]
    })
}

/// Extract function selections (only when both FUNCTION_BODY and FUNCTION_NAME are present)
fn extract_functions(
    file: &str,
    single_map: &HashMap<String, MetaVarItem>,
) -> Option<Vec<FunctionSelection>> {
    match (
        single_map.get("FUNCTION_BODY"),
        single_map.get("FUNCTION_NAME"),
    ) {
        (Some(body), Some(name)) => Some(vec![FunctionSelection {
            file: file.to_string(),
            body_text: body.text.clone(),
            body_range: body.range.clone(),
            name_text: name.text.clone(),
            name_range: name.range.clone(),
        }]),
        _ => None,
    }
}

/// Extract IMPL_BODY selections
fn extract_impls(
    file: &str,
    single_map: &HashMap<String, MetaVarItem>,
) -> Option<Vec<ImplSelection>> {
    if let Some(impl_body) = single_map.get("IMPL_BODY") {
        Some(vec![ImplSelection {
            file: file.to_string(),
            impl_text: impl_body.text.clone(),
            impl_range: impl_body.range.clone(),
        }])
    } else {
        None
    }
}

/// Extract struct selections (STRUCT_BODY + STRUCT_NAME)
fn extract_structs(
    file: &str,
    single_map: &HashMap<String, MetaVarItem>,
) -> Option<Vec<StructSelection>> {
    match (single_map.get("STRUCT_BODY"), single_map.get("STRUCT_NAME")) {
        (Some(body), Some(name)) => Some(vec![StructSelection {
            file: file.to_string(),
            body_text: body.text.clone(),
            body_range: body.range.clone(),
            name_text: name.text.clone(),
            name_range: name.range.clone(),
        }]),
        _ => None,
    }
}

/// Extract the requested selections from a JSON string (ast-grep output).
/// Returns `Option<Vec<...>>` for each selection type; `None` means no matches were
/// produced by the corresponding extractor across the entire JSON.
pub fn extract_selections_from_ast_grep_json(
    json: &str,
) -> Result<
    (
        Option<Vec<AttributeSelection>>,
        Option<Vec<TestsModSelection>>,
        Option<Vec<FunctionSelection>>,
        Option<Vec<MethodSelection>>,
        Option<Vec<ImplSelection>>,
        Option<Vec<StructSelection>>,
        Option<Vec<TraitSelection>>,
        Option<Vec<TraitMethodSignatureSelection>>,
        Option<Vec<TraitMethodDefinitionSelection>>,
        Option<Vec<TypeAliasSelection>>,
        Option<Vec<EnumSelection>>,
        Option<Vec<UnionSelection>>,
    ),
    serde_json::Error,
> {
    let raw_matches: Vec<RawMatch> = serde_json::from_str(json)?;

    let mut attributes: Option<Vec<AttributeSelection>> = None;
    let mut tests_mods: Option<Vec<TestsModSelection>> = None;
    let mut functions: Option<Vec<FunctionSelection>> = None;
    let mut methods: Option<Vec<MethodSelection>> = None;
    let mut impls: Option<Vec<ImplSelection>> = None;
    let mut structs: Option<Vec<StructSelection>> = None;
    let mut traits: Option<Vec<TraitSelection>> = None;
    let mut trait_method_sigs: Option<Vec<TraitMethodSignatureSelection>> = None;
    let mut enums: Option<Vec<EnumSelection>> = None;
    let mut unions: Option<Vec<UnionSelection>> = None;
    let mut trait_method_defs: Option<Vec<TraitMethodDefinitionSelection>> = None;
    let mut type_aliases: Option<Vec<TypeAliasSelection>> = None;

    for m in &raw_matches {
        let file = m.file.clone();

        if let Some(meta_vars) = &m.meta_variables {
            if let Some(single_map) = &meta_vars.single {
                if let Some(v) = extract_attributes(&file, single_map) {
                    match &mut attributes {
                        Some(vec) => vec.extend(v),
                        None => attributes = Some(v),
                    }
                }
                if let Some(v) = extract_tests_mods(&file, single_map) {
                    match &mut tests_mods {
                        Some(vec) => vec.extend(v),
                        None => tests_mods = Some(v),
                    }
                }
                if let Some(v) = extract_functions(&file, single_map) {
                    match &mut functions {
                        Some(vec) => vec.extend(v),
                        None => functions = Some(v),
                    }
                }
                if let Some(v) = extract_methods(&file, single_map) {
                    match &mut methods {
                        Some(vec) => vec.extend(v),
                        None => methods = Some(v),
                    }
                }
                if let Some(v) = extract_impls(&file, single_map) {
                    match &mut impls {
                        Some(vec) => vec.extend(v),
                        None => impls = Some(v),
                    }
                }
                if let Some(v) = extract_structs(&file, single_map) {
                    match &mut structs {
                        Some(vec) => vec.extend(v),
                        None => structs = Some(v),
                    }
                }

                if let Some(v) = extract_traits(&file, single_map) {
                    match &mut traits {
                        Some(vec) => vec.extend(v),
                        None => traits = Some(v),
                    }
                }
                if let Some(v) = extract_trait_method_signatures(&file, single_map) {
                    match &mut trait_method_sigs {
                        Some(vec) => vec.extend(v),
                        None => trait_method_sigs = Some(v),
                    }
                }

                if let Some(v) = extract_trait_methods(&file, single_map) {
                    match &mut trait_method_defs {
                        Some(vec) => vec.extend(v),
                        None => trait_method_defs = Some(v),
                    }
                }
                if let Some(v) = extract_type_aliases(&file, single_map) {
                    match &mut type_aliases {
                        Some(vec) => vec.extend(v),
                        None => type_aliases = Some(v),
                    }
                }
                if let Some(v) = extract_enums(&file, single_map) {
                    match &mut enums {
                        Some(vec) => vec.extend(v),
                        None => enums = Some(v),
                    }
                }
                if let Some(v) = extract_unions(&file, single_map) {
                    match &mut unions {
                        Some(vec) => vec.extend(v),
                        None => unions = Some(v),
                    }
                }
            }
        }
    }

    Ok((
        attributes,
        tests_mods,
        functions,
        methods,
        impls,
        structs,
        traits,
        trait_method_sigs,
        trait_method_defs,
        type_aliases,
        enums,
        unions,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // sample_json() gathers the part functions and concatenates them into the final array.
    fn sample_json() -> String {
        let parts = vec![
            attributes(),
            tests_mod(),
            struct_body(),
            impl_body(),
            method_body(),
            trait_method_signature(),
            trait_body(),
            function_body(),
            trait_method_with_body(),
            type_alias_object(),
            union_body(),
            enum_body(),
        ];

        // join with commas and wrap in brackets
        format!("[\n{}\n]", parts.join(",\n"))
    }

    #[test]
    fn test_extract_selections_from_ast_grep_json() {
        let json = sample_json();
        let (
            attributes,
            tests_mods,
            functions,
            methods,
            impls,
            structs,
            traits,
            trait_method_signatures,
            trait_method_defs,
            type_aliases,
            enums,
            unions,
        ) = extract_selections_from_ast_grep_json(&json).expect("JSON should parse");

        // unwrap all the Option<Vec<...>> into Vec<...>
        let attributes = attributes.expect("expected attributes");
        let tests_mods = tests_mods.expect("expected tests_mods");
        let functions = functions.expect("expected functions");
        let methods = methods.expect("expected methods");
        let impls = impls.expect("expected impls");
        let structs = structs.expect("expected structs");
        let traits = traits.expect("expected traits");
        let trait_method_signatures =
            trait_method_signatures.expect("expected trait_method_signatures");
        let trait_method_defs = trait_method_defs.expect("expected trait_method_defs");
        let type_aliases = type_aliases.expect("expected type_aliases");
        let enums = enums.expect("expected enums");
        let unions = unions.expect("expected unions");

        // ATTRIBUTES
        assert_eq!(attributes.len(), 1, "expected one attribute selection");
        let attr = &attributes[0];
        assert_eq!(attr.file, "sample_program.rs");
        assert_eq!(attr.attribute, "#[cfg(test)]");

        // TESTS_MOD
        assert_eq!(tests_mods.len(), 1, "expected one tests_mod selection");
        let tmod = &tests_mods[0];
        assert_eq!(tmod.file, "sample_program.rs");
        assert!(tmod.tests_mod.contains("mod tests {"));
        assert!(tmod.tests_mod.contains("fn test_json_selectors_ast_grep()"));

        // FUNCTION
        assert_eq!(functions.len(), 1, "expected one function selection");
        let func = &functions[0];
        assert_eq!(func.file, "sample_program.rs");
        assert!(
            func.body_text.contains(
                "fn make_iter() -> impl Iterator<Item = u8> {\n    std::iter::once(1u8)\n}"
            )
        );
        assert_eq!(func.name_text, "make_iter");

        // METHODS (inherent method)
        assert_eq!(methods.len(), 1, "expected one method selection");
        let method = &methods[0];
        assert_eq!(method.file, "sample_program.rs");
        assert_eq!(method.name_text, "inherent_method");
        assert!(method.body_text.contains("fn inherent_method(&self) {}"));
        // impl_text is a String now
        assert!(
            !method.impl_text.is_empty(),
            "expected impl_text for the method"
        );
        assert!(method.impl_text.contains("impl MyType"));

        // IMPLS
        assert_eq!(impls.len(), 1, "expected one impl selection");
        let imp = &impls[0];
        assert_eq!(imp.file, "sample_program.rs");
        assert!(imp.impl_text.contains("impl<T> SomeTrait for Wrapper<T>"));
        assert!(imp.impl_text.contains("type Assoc = Wrapper<T>;"));

        // STRUCTS
        assert_eq!(structs.len(), 1, "expected one struct selection");
        let st = &structs[0];
        assert_eq!(st.file, "sample_program.rs");
        assert_eq!(st.name_text, "MyType");
        assert!(st.body_text.contains("struct MyType;"));

        // TRAITS
        assert_eq!(traits.len(), 1, "expected one trait selection");
        let tr = &traits[0];
        assert_eq!(tr.file, "sample_program.rs");
        assert_eq!(tr.trait_name_text, "SomeTrait");
        assert!(tr.trait_body_text.contains("fn my_function();"));
        assert!(tr.trait_body_text.contains("trait SomeTrait<T>"));

        // TRAIT METHOD SIGNATURES
        assert_eq!(
            trait_method_signatures.len(),
            1,
            "expected one trait method signature selection"
        );
        let sig = &trait_method_signatures[0];
        assert_eq!(sig.file, "sample_program.rs");
        assert_eq!(sig.signature_text, "fn my_function();");
        assert_eq!(sig.signature_name_text, "my_function");
        assert!(
            !sig.enclosing_trait_text.is_empty(),
            "expected enclosing trait text"
        );
        assert!(sig.enclosing_trait_text.contains("trait SomeTrait<T>"));
        assert!(!sig.trait_name_text.is_empty(), "expected trait name text");
        assert_eq!(sig.trait_name_text, "SomeTrait");

        // TRAIT METHOD DEFINITION
        assert_eq!(
            trait_method_defs.len(),
            1,
            "expected one trait method definition"
        );
        let tm = &trait_method_defs[0];
        assert_eq!(tm.file, "sample_program.rs");
        assert_eq!(tm.method_name_text, "trait_function");
        assert!(
            tm.method_body_text
                .contains("fn trait_function(t: T) -> Self { Wrapper(t) }")
        );
        assert!(!tm.trait_body_text.is_empty());
        assert!(tm.trait_body_text.contains("trait SomeTrait"));
        assert!(!tm.trait_name_text.is_empty());
        assert_eq!(tm.trait_name_text, "SomeTrait");

        // TYPE ALIAS
        assert_eq!(type_aliases.len(), 1, "expected one type alias selection");
        let ta = &type_aliases[0];
        assert_eq!(ta.file, "sample_program.rs");
        assert_eq!(ta.name_text, "Assoc");
        assert_eq!(ta.body_text, "type Assoc = Wrapper<T>;");

        // UNIONS
        assert_eq!(unions.len(), 1, "expected one union selection");
        let un = &unions[0];
        assert_eq!(un.file, "sample_program.rs");
        assert_eq!(un.union_name_text, "IntOrFloat");
        assert!(un.union_body_text.contains("pub union IntOrFloat"));
        assert_eq!(un.union_body_range.start.line, 11);

        // ENUMS
        assert_eq!(enums.len(), 1, "expected one enum selection");
        let en = &enums[0];
        assert_eq!(en.file, "sample_program.rs");
        assert_eq!(en.enum_name_text, "Result");
        assert!(en.enum_body_text.contains("enum Result<T, E>"));
        assert_eq!(en.enum_body_range.start.line, 16);
    }

    /// Returns the JSON object for the `mod tests { ... }` (capture: $TESTS_MOD).
    fn tests_mod() -> String {
        r###"
{
  "text": "mod tests {\n    #[test]\n    fn test_json_selectors_ast_grep() {\n        use super::*;\n        let temp_dir = tempfile::tempdir().expect(\"Failed to create temp directory\");\n    }\n}",
  "range": {
    "byteOffset": {
      "start": 13,
      "end": 192
    },
    "start": {
      "line": 1,
      "column": 0
    },
    "end": {
      "line": 7,
      "column": 1
    }
  },
  "file": "sample_program.rs",
  "lines": "mod tests {\n    #[test]\n    fn test_json_selectors_ast_grep() {\n        use super::*;\n        let temp_dir = tempfile::tempdir().expect(\"Failed to create temp directory\");\n    }\n}",
  "charCount": {
    "leading": 0,
    "trailing": 0
  },
  "language": "Rust",
  "metaVariables": {
    "single": {
      "TESTS_MOD": {
        "text": "mod tests {\n    #[test]\n    fn test_json_selectors_ast_grep() {\n        use super::*;\n        let temp_dir = tempfile::tempdir().expect(\"Failed to create temp directory\");\n    }\n}",
        "range": {
          "byteOffset": {
            "start": 13,
            "end": 192
          },
          "start": {
            "line": 1,
            "column": 0
          },
          "end": {
            "line": 7,
            "column": 1
          }
        }
      }
    },
    "multi": {
      "secondary": [
        {
          "text": "tests",
          "range": {
            "byteOffset": {
              "start": 17,
              "end": 22
            },
            "start": {
              "line": 1,
              "column": 4
            },
            "end": {
              "line": 1,
              "column": 9
            }
          }
        }
      ]
    },
    "transformed": {}
  },
  "ruleId": "find-all-syntactic-elements",
  "severity": "hint",
  "note": null,
  "message": "",
  "labels": [
    {
      "text": "mod tests {\n    #[test]\n    fn test_json_selectors_ast_grep() {\n        use super::*;\n        let temp_dir = tempfile::tempdir().expect(\"Failed to create temp directory\");\n    }\n}",
      "range": {
        "byteOffset": {
          "start": 13,
          "end": 192
        },
        "start": {
          "line": 1,
          "column": 0
        },
        "end": {
          "line": 7,
          "column": 1
        }
      },
      "style": "primary"
    },
    {
      "text": "tests",
      "range": {
        "byteOffset": {
          "start": 17,
          "end": 22
        },
        "start": {
          "line": 1,
          "column": 4
        },
        "end": {
          "line": 1,
          "column": 9
        }
      },
      "style": "secondary"
    }
  ]
}
"###.to_string()
    }

    // Returns the JSON object for the union (capture: $UNION_BODY).
    fn union_body() -> String {
        r###"
{
  "text": "pub union IntOrFloat {\n    pub i: u32,\n    pub f: f32,\n}",
  "range": {
    "byteOffset": {
      "start": 228,
      "end": 284
    },
    "start": {
      "line": 11,
      "column": 0
    },
    "end": {
      "line": 14,
      "column": 1
    }
  },
  "file": "sample_program.rs",
  "lines": "pub union IntOrFloat {\n    pub i: u32,\n    pub f: f32,\n}",
  "charCount": {
    "leading": 0,
    "trailing": 0
  },
  "language": "Rust",
  "metaVariables": {
    "single": {
      "UNION_NAME": {
        "text": "IntOrFloat",
        "range": {
          "byteOffset": {
            "start": 238,
            "end": 248
          },
          "start": {
            "line": 11,
            "column": 10
          },
          "end": {
            "line": 11,
            "column": 20
          }
        }
      },
      "UNION_BODY": {
        "text": "pub union IntOrFloat {\n    pub i: u32,\n    pub f: f32,\n}",
        "range": {
          "byteOffset": {
            "start": 228,
            "end": 284
          },
          "start": {
            "line": 11,
            "column": 0
          },
          "end": {
            "line": 14,
            "column": 1
          }
        }
      }
    },
    "multi": {
      "secondary": [
        {
          "text": "IntOrFloat",
          "range": {
            "byteOffset": {
              "start": 238,
              "end": 248
            },
            "start": {
              "line": 11,
              "column": 10
            },
            "end": {
              "line": 11,
              "column": 20
            }
          }
        }
      ]
    },
    "transformed": {}
  },
  "ruleId": "find-all-syntactic-elements",
  "severity": "hint",
  "note": null,
  "message": "",
  "labels": [
    {
      "text": "pub union IntOrFloat {\n    pub i: u32,\n    pub f: f32,\n}",
      "range": {
        "byteOffset": {
          "start": 228,
          "end": 284
        },
        "start": {
          "line": 11,
          "column": 0
        },
        "end": {
          "line": 14,
          "column": 1
        }
      },
      "style": "primary"
    },
    {
      "text": "IntOrFloat",
      "range": {
        "byteOffset": {
          "start": 238,
          "end": 248
        },
        "start": {
          "line": 11,
          "column": 10
        },
        "end": {
          "line": 11,
          "column": 20
        }
      },
      "style": "secondary"
    }
  ]
}
"###
        .to_string()
    }

    // Returns the JSON object for the enum (capture: $ENUM_BODY).
    fn enum_body() -> String {
        r###"
{
  "text": "enum Result<T, E>\nwhere\n    E: std::error::Error,\n{\n    Ok(T),\n    Err(E),\n}",
  "range": {
    "byteOffset": {
      "start": 286,
      "end": 362
    },
    "start": {
      "line": 16,
      "column": 0
    },
    "end": {
      "line": 22,
      "column": 1
    }
  },
  "file": "sample_program.rs",
  "lines": "enum Result<T, E>\nwhere\n    E: std::error::Error,\n{\n    Ok(T),\n    Err(E),\n}",
  "charCount": {
    "leading": 0,
    "trailing": 0
  },
  "language": "Rust",
  "metaVariables": {
    "single": {
      "ENUM_NAME": {
        "text": "Result",
        "range": {
          "byteOffset": {
            "start": 291,
            "end": 297
          },
          "start": {
            "line": 16,
            "column": 5
          },
          "end": {
            "line": 16,
            "column": 11
          }
        }
      },
      "ENUM_BODY": {
        "text": "enum Result<T, E>\nwhere\n    E: std::error::Error,\n{\n    Ok(T),\n    Err(E),\n}",
        "range": {
          "byteOffset": {
            "start": 286,
            "end": 362
          },
          "start": {
            "line": 16,
            "column": 0
          },
          "end": {
            "line": 22,
            "column": 1
          }
        }
      }
    },
    "multi": {
      "secondary": [
        {
          "text": "Result",
          "range": {
            "byteOffset": {
              "start": 291,
              "end": 297
            },
            "start": {
              "line": 16,
              "column": 5
            },
            "end": {
              "line": 16,
              "column": 11
            }
          }
        }
      ]
    },
    "transformed": {}
  },
  "ruleId": "find-all-syntactic-elements",
  "severity": "hint",
  "note": null,
  "message": "",
  "labels": [
    {
      "text": "enum Result<T, E>\nwhere\n    E: std::error::Error,\n{\n    Ok(T),\n    Err(E),\n}",
      "range": {
        "byteOffset": {
          "start": 286,
          "end": 362
        },
        "start": {
          "line": 16,
          "column": 0
        },
        "end": {
          "line": 22,
          "column": 1
        }
      },
      "style": "primary"
    },
    {
      "text": "Result",
      "range": {
        "byteOffset": {
          "start": 291,
          "end": 297
        },
        "start": {
          "line": 16,
          "column": 5
        },
        "end": {
          "line": 16,
          "column": 11
        }
      },
      "style": "secondary"
    }
  ]
}
"###.to_string()
    }

    /// Returns the JSON object for the inherent method (capture: $METHOD_BODY).
    fn method_body() -> String {
        r###"
{
  "text": "fn inherent_method(&self) {}",
  "range": {
    "byteOffset": {
      "start": 626,
      "end": 654
    },
    "start": {
      "line": 38,
      "column": 4
    },
    "end": {
      "line": 38,
      "column": 32
    }
  },
  "file": "sample_program.rs",
  "lines": "    fn inherent_method(&self) {}",
  "charCount": {
    "leading": 4,
    "trailing": 0
  },
  "language": "Rust",
  "metaVariables": {
    "single": {
      "METHOD_IMPL_BODY": {
        "text": "impl MyType {\n    fn inherent_method(&self) {}\n}",
        "range": {
          "byteOffset": {
            "start": 608,
            "end": 656
          },
          "start": {
            "line": 37,
            "column": 0
          },
          "end": {
            "line": 39,
            "column": 1
          }
        }
      },
      "METHOD_BODY": {
        "text": "fn inherent_method(&self) {}",
        "range": {
          "byteOffset": {
            "start": 626,
            "end": 654
          },
          "start": {
            "line": 38,
            "column": 4
          },
          "end": {
            "line": 38,
            "column": 32
          }
        }
      },
      "METHOD_NAME": {
        "text": "inherent_method",
        "range": {
          "byteOffset": {
            "start": 629,
            "end": 644
          },
          "start": {
            "line": 38,
            "column": 7
          },
          "end": {
            "line": 38,
            "column": 22
          }
        }
      }
    },
    "multi": {
      "secondary": [
        {
          "text": "impl MyType {\n    fn inherent_method(&self) {}\n}",
          "range": {
            "byteOffset": {
              "start": 608,
              "end": 656
            },
            "start": {
              "line": 37,
              "column": 0
            },
            "end": {
              "line": 39,
              "column": 1
            }
          }
        },
        {
          "text": "inherent_method",
          "range": {
            "byteOffset": {
              "start": 629,
              "end": 644
            },
            "start": {
              "line": 38,
              "column": 7
            },
            "end": {
              "line": 38,
              "column": 22
            }
          }
        }
      ]
    },
    "transformed": {}
  },
  "ruleId": "find-all-syntactic-elements",
  "severity": "hint",
  "note": null,
  "message": "",
  "labels": [
    {
      "text": "fn inherent_method(&self) {}",
      "range": {
        "byteOffset": {
          "start": 626,
          "end": 654
        },
        "start": {
          "line": 38,
          "column": 4
        },
        "end": {
          "line": 38,
          "column": 32
        }
      },
      "style": "primary"
    },
    {
      "text": "impl MyType {\n    fn inherent_method(&self) {}\n}",
      "range": {
        "byteOffset": {
          "start": 608,
          "end": 656
        },
        "start": {
          "line": 37,
          "column": 0
        },
        "end": {
          "line": 39,
          "column": 1
        }
      },
      "style": "secondary"
    },
    {
      "text": "inherent_method",
      "range": {
        "byteOffset": {
          "start": 629,
          "end": 644
        },
        "start": {
          "line": 38,
          "column": 7
        },
        "end": {
          "line": 38,
          "column": 22
        }
      },
      "style": "secondary"
    }
  ]
}
"###
        .to_string()
    }

    /// Returns the JSON object for the attribute item (capture: $ATTRIBUTES).
    fn attributes() -> String {
        r###"
{
  "text": "#[cfg(test)]",
  "range": {
    "byteOffset": {
      "start": 0,
      "end": 12
    },
    "start": {
      "line": 0,
      "column": 0
    },
    "end": {
      "line": 0,
      "column": 12
    }
  },
  "file": "sample_program.rs",
  "lines": "#[cfg(test)]",
  "charCount": {
    "leading": 0,
    "trailing": 0
  },
  "language": "Rust",
  "metaVariables": {
    "single": {
      "ATTRIBUTES": {
        "text": "#[cfg(test)]",
        "range": {
          "byteOffset": {
            "start": 0,
            "end": 12
          },
          "start": {
            "line": 0,
            "column": 0
          },
          "end": {
            "line": 0,
            "column": 12
          }
        }
      }
    },
    "multi": {},
    "transformed": {}
  },
  "ruleId": "find-all-syntactic-elements",
  "severity": "hint",
  "note": null,
  "message": "",
  "labels": [
    {
      "text": "#[cfg(test)]",
      "range": {
        "byteOffset": {
          "start": 0,
          "end": 12
        },
        "start": {
          "line": 0,
          "column": 0
        },
        "end": {
          "line": 0,
          "column": 12
        }
      },
      "style": "primary"
    }
  ]
}
"###
        .to_string()
    }

    /// Returns the JSON object for `struct MyType;` (capture: $STRUCT_BODY / $STRUCT_NAME).
    fn struct_body() -> String {
        r###"
{
  "text": "struct MyType;",
  "range": {
    "byteOffset": {
      "start": 364,
      "end": 378
    },
    "start": {
      "line": 24,
      "column": 0
    },
    "end": {
      "line": 24,
      "column": 14
    }
  },
  "file": "sample_program.rs",
  "lines": "struct MyType;",
  "charCount": {
    "leading": 0,
    "trailing": 0
  },
  "language": "Rust",
  "metaVariables": {
    "single": {
      "STRUCT_NAME": {
        "text": "MyType",
        "range": {
          "byteOffset": {
            "start": 371,
            "end": 377
          },
          "start": {
            "line": 24,
            "column": 7
          },
          "end": {
            "line": 24,
            "column": 13
          }
        }
      },
      "STRUCT_BODY": {
        "text": "struct MyType;",
        "range": {
          "byteOffset": {
            "start": 364,
            "end": 378
          },
          "start": {
            "line": 24,
            "column": 0
          },
          "end": {
            "line": 24,
            "column": 14
          }
        }
      }
    },
    "multi": {
      "secondary": [
        {
          "text": "MyType",
          "range": {
            "byteOffset": {
              "start": 371,
              "end": 377
            },
            "start": {
              "line": 24,
              "column": 7
            },
            "end": {
              "line": 24,
              "column": 13
            }
          }
        }
      ]
    },
    "transformed": {}
  },
  "ruleId": "find-all-syntactic-elements",
  "severity": "hint",
  "note": null,
  "message": "",
  "labels": [
    {
      "text": "struct MyType;",
      "range": {
        "byteOffset": {
          "start": 364,
          "end": 378
        },
        "start": {
          "line": 24,
          "column": 0
        },
        "end": {
          "line": 24,
          "column": 14
        }
      },
      "style": "primary"
    },
    {
      "text": "MyType",
      "range": {
        "byteOffset": {
          "start": 371,
          "end": 377
        },
        "start": {
          "line": 24,
          "column": 7
        },
        "end": {
          "line": 24,
          "column": 13
        }
      },
      "style": "secondary"
    }
  ]
}
"###
        .to_string()
    }

    /// Returns the JSON object for the impl block (capture: $IMPL_BODY).
    fn impl_body() -> String {
        r###"
{
  "text": "impl<T> SomeTrait for Wrapper<T> {\n    type Assoc = Wrapper<T>;\n}",
  "range": {
    "byteOffset": {
      "start": 542,
      "end": 607
    },
    "start": {
      "line": 34,
      "column": 0
    },
    "end": {
      "line": 36,
      "column": 1
    }
  },
  "file": "sample_program.rs",
  "lines": "impl<T> SomeTrait for Wrapper<T> {\n    type Assoc = Wrapper<T>;\n}",
  "charCount": {
    "leading": 0,
    "trailing": 0
  },
  "language": "Rust",
  "metaVariables": {
    "single": {
      "IMPL_BODY": {
        "text": "impl<T> SomeTrait for Wrapper<T> {\n    type Assoc = Wrapper<T>;\n}",
        "range": {
          "byteOffset": {
            "start": 542,
            "end": 607
          },
          "start": {
            "line": 34,
            "column": 0
          },
          "end": {
            "line": 36,
            "column": 1
          }
        }
      }
    },
    "multi": {},
    "transformed": {}
  },
  "ruleId": "find-all-syntactic-elements",
  "severity": "hint",
  "note": null,
  "message": "",
  "labels": [
    {
      "text": "impl<T> SomeTrait for Wrapper<T> {\n    type Assoc = Wrapper<T>;\n}",
      "range": {
        "byteOffset": {
          "start": 542,
          "end": 607
        },
        "start": {
          "line": 34,
          "column": 0
        },
        "end": {
          "line": 36,
          "column": 1
        }
      },
      "style": "primary"
    }
  ]
}
"###
        .to_string()
    }

    /// Returns the JSON object for the trait method signature inside a trait (capture: $TRAIT_METHOD_SIGNATURE).
    fn trait_method_signature() -> String {
        r###"
{
  "text": "fn my_function();",
  "range": {
    "byteOffset": {
      "start": 468,
      "end": 485
    },
    "start": {
      "line": 29,
      "column": 4
    },
    "end": {
      "line": 29,
      "column": 21
    }
  },
  "file": "sample_program.rs",
  "lines": "    fn my_function();",
  "charCount": {
    "leading": 4,
    "trailing": 0
  },
  "language": "Rust",
  "metaVariables": {
    "single": {
      "TRAIT_METHOD_SIGNATURE_NAME": {
        "text": "my_function",
        "range": {
          "byteOffset": {
            "start": 471,
            "end": 482
          },
          "start": {
            "line": 29,
            "column": 7
          },
          "end": {
            "line": 29,
            "column": 18
          }
        }
      },
      "TRAIT_METHOD_SIGNATURE": {
        "text": "fn my_function();",
        "range": {
          "byteOffset": {
            "start": 468,
            "end": 485
          },
          "start": {
            "line": 29,
            "column": 4
          },
          "end": {
            "line": 29,
            "column": 21
          }
        }
      },
      "TRAIT_BODY_WITH_METHOD_SIGNATURE": {
        "text": "trait SomeTrait<T> {\n    fn my_function();\n}",
        "range": {
          "byteOffset": {
            "start": 443,
            "end": 487
          },
          "start": {
            "line": 28,
            "column": 0
          },
          "end": {
            "line": 30,
            "column": 1
          }
        }
      },
      "TRAIT_NAME_METHOD_SIGNATURE": {
        "text": "SomeTrait",
        "range": {
          "byteOffset": {
            "start": 449,
            "end": 458
          },
          "start": {
            "line": 28,
            "column": 6
          },
          "end": {
            "line": 28,
            "column": 15
          }
        }
      }
    },
    "multi": {
      "secondary": [
        {
          "text": "my_function",
          "range": {
            "byteOffset": {
              "start": 471,
              "end": 482
            },
            "start": {
              "line": 29,
              "column": 7
            },
            "end": {
              "line": 29,
              "column": 18
            }
          }
        },
        {
          "text": "SomeTrait",
          "range": {
            "byteOffset": {
              "start": 449,
              "end": 458
            },
            "start": {
              "line": 28,
              "column": 6
            },
            "end": {
              "line": 28,
              "column": 15
            }
          }
        },
        {
          "text": "trait SomeTrait<T> {\n    fn my_function();\n}",
          "range": {
            "byteOffset": {
              "start": 443,
              "end": 487
            },
            "start": {
              "line": 28,
              "column": 0
            },
            "end": {
              "line": 30,
              "column": 1
            }
          }
        }
      ]
    },
    "transformed": {}
  },
  "ruleId": "find-all-syntactic-elements",
  "severity": "hint",
  "note": null,
  "message": "",
  "labels": [
    {
      "text": "fn my_function();",
      "range": {
        "byteOffset": {
          "start": 468,
          "end": 485
        },
        "start": {
          "line": 29,
          "column": 4
        },
        "end": {
          "line": 29,
          "column": 21
        }
      },
      "style": "primary"
    },
    {
      "text": "my_function",
      "range": {
        "byteOffset": {
          "start": 471,
          "end": 482
        },
        "start": {
          "line": 29,
          "column": 7
        },
        "end": {
          "line": 29,
          "column": 18
        }
      },
      "style": "secondary"
    },
    {
      "text": "SomeTrait",
      "range": {
        "byteOffset": {
          "start": 449,
          "end": 458
        },
        "start": {
          "line": 28,
          "column": 6
        },
        "end": {
          "line": 28,
          "column": 15
        }
      },
      "style": "secondary"
    },
    {
      "text": "trait SomeTrait<T> {\n    fn my_function();\n}",
      "range": {
        "byteOffset": {
          "start": 443,
          "end": 487
        },
        "start": {
          "line": 28,
          "column": 0
        },
        "end": {
          "line": 30,
          "column": 1
        }
      },
      "style": "secondary"
    }
  ]
}
"###
        .to_string()
    }

    /// Returns the JSON object for the trait declaration (capture: $TRAIT_BODY).
    fn trait_body() -> String {
        r###"
{
  "text": "trait SomeTrait<T> {\n    fn my_function();\n}",
  "range": {
    "byteOffset": {
      "start": 443,
      "end": 487
    },
    "start": {
      "line": 28,
      "column": 0
    },
    "end": {
      "line": 30,
      "column": 1
    }
  },
  "file": "sample_program.rs",
  "lines": "trait SomeTrait<T> {\n    fn my_function();\n}",
  "charCount": {
    "leading": 0,
    "trailing": 0
  },
  "language": "Rust",
  "metaVariables": {
    "single": {
      "TRAIT_BODY": {
        "text": "trait SomeTrait<T> {\n    fn my_function();\n}",
        "range": {
          "byteOffset": {
            "start": 443,
            "end": 487
          },
          "start": {
            "line": 28,
            "column": 0
          },
          "end": {
            "line": 30,
            "column": 1
          }
        }
      },
      "TRAIT_NAME": {
        "text": "SomeTrait",
        "range": {
          "byteOffset": {
            "start": 449,
            "end": 458
          },
          "start": {
            "line": 28,
            "column": 6
          },
          "end": {
            "line": 28,
            "column": 15
          }
        }
      }
    },
    "multi": {
      "secondary": [
        {
          "text": "SomeTrait",
          "range": {
            "byteOffset": {
              "start": 449,
              "end": 458
            },
            "start": {
              "line": 28,
              "column": 6
            },
            "end": {
              "line": 28,
              "column": 15
            }
          }
        }
      ]
    },
    "transformed": {}
  },
  "ruleId": "find-all-syntactic-elements",
  "severity": "hint",
  "note": null,
  "message": "",
  "labels": [
    {
      "text": "trait SomeTrait<T> {\n    fn my_function();\n}",
      "range": {
        "byteOffset": {
          "start": 443,
          "end": 487
        },
        "start": {
          "line": 28,
          "column": 0
        },
        "end": {
          "line": 30,
          "column": 1
        }
      },
      "style": "primary"
    },
    {
      "text": "SomeTrait",
      "range": {
        "byteOffset": {
          "start": 449,
          "end": 458
        },
        "start": {
          "line": 28,
          "column": 6
        },
        "end": {
          "line": 28,
          "column": 15
        }
      },
      "style": "secondary"
    }
  ]
}
"###
        .to_string()
    }

    /// Returns the JSON object for the free function `make_iter` (capture: $FUNCTION_BODY / $FUNCTION_NAME).
    fn function_body() -> String {
        r###"
{
  "text": "fn make_iter() -> impl Iterator<Item = u8> {\n    std::iter::once(1u8)\n}",
  "range": {
    "byteOffset": {
      "start": 1829,
      "end": 1900
    },
    "start": {
      "line": 82,
      "column": 0
    },
    "end": {
      "line": 84,
      "column": 1
    }
  },
  "file": "sample_program.rs",
  "lines": "fn make_iter() -> impl Iterator<Item = u8> {\n    std::iter::once(1u8)\n}",
  "charCount": {
    "leading": 0,
    "trailing": 0
  },
  "language": "Rust",
  "metaVariables": {
    "single": {
      "FUNCTION_BODY": {
        "text": "fn make_iter() -> impl Iterator<Item = u8> {\n    std::iter::once(1u8)\n}",
        "range": {
          "byteOffset": {
            "start": 1829,
            "end": 1900
          },
          "start": {
            "line": 82,
            "column": 0
          },
          "end": {
            "line": 84,
            "column": 1
          }
        }
      },
      "FUNCTION_NAME": {
        "text": "make_iter",
        "range": {
          "byteOffset": {
            "start": 1832,
            "end": 1841
          },
          "start": {
            "line": 82,
            "column": 3
          },
          "end": {
            "line": 82,
            "column": 12
          }
        }
      }
    },
    "multi": {
      "secondary": [
        {
          "text": "make_iter",
          "range": {
            "byteOffset": {
              "start": 1832,
              "end": 1841
            },
            "start": {
              "line": 82,
              "column": 3
            },
            "end": {
              "line": 82,
              "column": 12
            }
          }
        }
      ]
    },
    "transformed": {}
  },
  "ruleId": "find-all-syntactic-elements",
  "severity": "hint",
  "note": null,
  "message": "",
  "labels": [
    {
      "text": "fn make_iter() -> impl Iterator<Item = u8> {\n    std::iter::once(1u8)\n}",
      "range": {
        "byteOffset": {
          "start": 1829,
          "end": 1900
        },
        "start": {
          "line": 82,
          "column": 0
        },
        "end": {
          "line": 84,
          "column": 1
        }
      },
      "style": "primary"
    },
    {
      "text": "make_iter",
      "range": {
        "byteOffset": {
          "start": 1832,
          "end": 1841
        },
        "start": {
          "line": 82,
          "column": 3
        },
        "end": {
          "line": 82,
          "column": 12
        }
      },
      "style": "secondary"
    }
  ]
}
"###
        .to_string()
    }

    fn trait_method_with_body() -> String {
        r###"
{
  "text": "fn trait_function(t: T) -> Self { Wrapper(t) }",
  "range": {
    "byteOffset": {
      "start": 538,
      "end": 584
    },
    "start": {
      "line": 32,
      "column": 18
    },
    "end": {
      "line": 32,
      "column": 64
    }
  },
  "file": "sample_program.rs",
  "lines": "trait SomeTrait { fn trait_function(t: T) -> Self { Wrapper(t) }}",
  "charCount": {
    "leading": 18,
    "trailing": 1
  },
  "language": "Rust",
  "metaVariables": {
    "single": {
      "TRAIT_METHOD_BODY": {
        "text": "fn trait_function(t: T) -> Self { Wrapper(t) }",
        "range": {
          "byteOffset": {
            "start": 538,
            "end": 584
          },
          "start": {
            "line": 32,
            "column": 18
          },
          "end": {
            "line": 32,
            "column": 64
          }
        }
      },
      "TRAIT_NAME_WITH_METHOD": {
        "text": "SomeTrait",
        "range": {
          "byteOffset": {
            "start": 526,
            "end": 535
          },
          "start": {
            "line": 32,
            "column": 6
          },
          "end": {
            "line": 32,
            "column": 15
          }
        }
      },
      "TRAIT_METHOD_NAME": {
        "text": "trait_function",
        "range": {
          "byteOffset": {
            "start": 541,
            "end": 555
          },
          "start": {
            "line": 32,
            "column": 21
          },
          "end": {
            "line": 32,
            "column": 35
          }
        }
      },
      "TRAIT_BODY_WITH_METHOD": {
        "text": "trait SomeTrait { fn trait_function(t: T) -> Self { Wrapper(t) }}",
        "range": {
          "byteOffset": {
            "start": 520,
            "end": 585
          },
          "start": {
            "line": 32,
            "column": 0
          },
          "end": {
            "line": 32,
            "column": 65
          }
        }
      }
    },
    "multi": {
      "secondary": []
    },
    "transformed": {}
  },
  "ruleId": "find-all-syntactic-elements",
  "severity": "hint"
}
"###
        .to_string()
    }

    fn type_alias_object() -> String {
        r###"
{
  "text": "type Assoc = Wrapper<T>;",
  "range": {
    "byteOffset": {
      "start": 647,
      "end": 671
    },
    "start": {
      "line": 36,
      "column": 4
    },
    "end": {
      "line": 36,
      "column": 28
    }
  },
  "file": "sample_program.rs",
  "lines": "    type Assoc = Wrapper<T>;",
  "charCount": {
    "leading": 4,
    "trailing": 0
  },
  "language": "Rust",
  "metaVariables": {
    "single": {
      "TYPE_ALIAS_NAME": {
        "text": "Assoc",
        "range": {
          "byteOffset": {
            "start": 652,
            "end": 657
          },
          "start": {
            "line": 36,
            "column": 9
          },
          "end": {
            "line": 36,
            "column": 14
          }
        }
      },
      "TYPE_ALIAS_BODY": {
        "text": "type Assoc = Wrapper<T>;",
        "range": {
          "byteOffset": {
            "start": 647,
            "end": 671
          },
          "start": {
            "line": 36,
            "column": 4
          },
          "end": {
            "line": 36,
            "column": 28
          }
        }
      }
    },
    "multi": {
      "secondary": []
    },
    "transformed": {}
  },
  "ruleId": "find-all-syntactic-elements",
  "severity": "hint"
}
"###
        .to_string()
    }
}
