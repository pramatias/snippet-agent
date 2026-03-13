//syn_method.rs
use crate::json_selection::unprocessed_elements::*;
use crate::syn::syn_elements::*;
use crate::syn::{FunctionSignature, ImplBody, ImplSignature, MethodBody, SynElement};
use log::debug;
use syntax_queries::RustParser;
use syntax_queries::byte_range_ordering::*;

///from_unprocessed
impl SynMethod {
    /// Replaces `From<UnprocessedMethod>` — requires `file_content` to resolve
    /// text for parsing. Call sites must supply it from their `file_contents` map.
    pub fn from_unprocessed(u: UnprocessedMethod, file_content: &str) -> Self {
        let (impl_sig, impl_sig_text) = extract_impl_signature(&u.impl_body);

        let (function_sig, _fn_sig_text) = extract_function_signature(&u.method_body, file_content);

        let (type_ids, ds_name) = TypeIdentifiers::from_impl_signature(&impl_sig_text);

        let ds: DSName = ds_name.unwrap_or_default();
        SynMethod {
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

/// Returns the `ImplSignature` element (range only) *and* the parsed text
/// string so the caller doesn't need to re-resolve it via `file_content`.
pub fn extract_impl_signature(impl_body: &ImplBody) -> (ImplSignature, String) {
    let file_offset = impl_body.range.byte_range.start;

    match RustParser::new(impl_body.text_str(), "declaration_list") {
        Ok(parser) => parser
            .delete_node_till_end_as_match()
            .map(|m| {
                let text = m.text.clone();
                let rebased = SynRange {
                    byte_range: ByteRange {
                        start: m.range.byte_range.start + file_offset,
                        end: m.range.byte_range.end + file_offset,
                    },
                    characters_dimension: CharactersDimension {
                        start: SynPosition {
                            line: impl_body.range.characters_dimension.start.line
                                + m.range.characters_dimension.start.line,
                            column: m.range.characters_dimension.start.column,
                        },
                        end: SynPosition {
                            line: impl_body.range.characters_dimension.start.line
                                + m.range.characters_dimension.end.line,
                            column: m.range.characters_dimension.end.column,
                        },
                    },
                };
                (SynElement::from_syn_range(rebased), text)
            })
            .unwrap_or_else(|| (SynElement::default(), String::new())),
        Err(err_str) => {
            eprintln!("RustParser::new failed for impl_body: {}", err_str);
            (SynElement::default(), String::new())
        }
    }
}

/// Returns the `FunctionSignature` element (range only) *and* the parsed text.
pub fn extract_function_signature(
    method_body: &MethodBody,
    file_content: &str,
) -> (FunctionSignature, String) {
    let file_offset = method_body.range.byte_range.start; // ← needed for rebasing
    match RustParser::new(method_body.text(file_content), "block") {
        Ok(parser) => parser
            .delete_node_till_end_as_match()
            .map(|m| {
                let text = m.text.clone();
                let rebased = SynRange {
                    byte_range: ByteRange {
                        start: m.range.byte_range.start + file_offset,
                        end: m.range.byte_range.end + file_offset,
                    },
                    characters_dimension: CharactersDimension {
                        start: SynPosition {
                            line: method_body.range.characters_dimension.start.line
                                + m.range.characters_dimension.start.line,
                            column: m.range.characters_dimension.start.column,
                        },
                        end: SynPosition {
                            line: method_body.range.characters_dimension.start.line
                                + m.range.characters_dimension.end.line,
                            column: m.range.characters_dimension.end.column,
                        },
                    },
                };
                (SynElement::from_syn_range(rebased), text)
            })
            .unwrap_or_else(|| (SynElement::default(), String::new())),
        Err(err_str) => {
            eprintln!("RustParser::new failed for method_body: {}", err_str);
            (SynElement::default(), String::new())
        }
    }
}
