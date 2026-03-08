//syn_method.rs
use crate::syn::impl_sig_types::*;
use crate::json_selection::unprocessed_elements::*;
use crate::syn::syn_elements::*;
use syntax_queries::RustParser;
use crate::syn::{ImplBody, MethodBody};

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
    let mut impl_signature: ImplSignature = String::new();

    match RustParser::new(&impl_body.text, "declaration_list") {
        Ok(parser) => {
            if let Some(sig) = parser.delete_node_till_end() {
                impl_signature = sig.trim().to_string();
            }
        }
        Err(err_str) => {
            eprintln!("RustParser::new failed for impl_body: {}", err_str);
        }
    }

    impl_signature
}

/// Extract the function signature from a `MethodBody`.
pub fn extract_function_signature(method_body: &MethodBody) -> FunctionSignature {
    let mut function_signature: FunctionSignature = String::new();

    match RustParser::new(&method_body.text, "block") {
        Ok(parser) => {
            if let Some(sig) = parser.delete_node_till_end() {
                function_signature = sig.trim().to_string();
            }
        }
        Err(err_str) => {
            eprintln!("RustParser::new failed for method_body: {}", err_str);
        }
    }

    function_signature
}
