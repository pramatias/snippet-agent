//syn_method.rs
use crate::json_selection::unprocessed_elements::*;
use crate::syn::syn_elements::*;
use crate::syn::{FunctionSignature, ImplBody, ImplSignature, MethodBody, SynElement};
use syntax_queries::RustParser;

impl From<UnprocessedMethod> for SynMethod {
    fn from(u: UnprocessedMethod) -> Self {
        let impl_sig: ImplSignature = extract_impl_signature(&u.impl_body);
        let function_sig: FunctionSignature = extract_function_signature(&u.method_body);

        // Single call — no redundant re-parsing of the impl signature
        let (type_ids, ds_name) = TypeIdentifiers::from_impl_signature(&impl_sig);
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

/// Extract the impl signature from an `ImplBody`.
pub fn extract_impl_signature(impl_body: &ImplBody) -> ImplSignature {
    match RustParser::new(&impl_body.text, "declaration_list") {
        Ok(parser) => parser
            .delete_node_till_end_as_match()
            .map(|m| SynElement {
                text: m.text,
                range: m.range,
            })
            .unwrap_or_default(),
        Err(err_str) => {
            eprintln!("RustParser::new failed for impl_body: {}", err_str);
            SynElement::default()
        }
    }
}

/// Extract the function signature from a `MethodBody`.
pub fn extract_function_signature(method_body: &MethodBody) -> FunctionSignature {
    match RustParser::new(&method_body.text, "block") {
        Ok(parser) => parser
            .delete_node_till_end_as_match()
            .map(|m| SynElement {
                text: m.text,
                range: m.range,
            })
            .unwrap_or_default(),
        Err(err_str) => {
            eprintln!("RustParser::new failed for method_body: {}", err_str);
            SynElement::default()
        }
    }
}
