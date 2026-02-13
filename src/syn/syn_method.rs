use crate::syn::impl_sig_types::*;
use crate::json_selection::unprocessed_elements::*;
use crate::syn::syn_elements::*;
use syntax_queries::RustParser;

impl From<UnprocessedMethod> for SynMethod {
    fn from(u: UnprocessedMethod) -> Self {
        // 1) extract impl signature from impl_body
        let impl_sig: ImplSignature = extract_impl_signature(&u.impl_body);

        // 2) extract function signature from method_body
        let function_sig: FunctionSignature = extract_function_signature(&u.method_body);

        // 3) extract type identifiers from impl signature
        let type_ids: TypeIdentifiers = extract_type_identifiers(&impl_sig);

        // 4) extract DS name (may use impl_sig + discovered type identifiers)
        let ds: DSName = extract_ds_structure(&impl_sig, &type_ids);

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

    // pass the inner string to RustParser::new
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

    // pass the inner string to RustParser::new
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

/// Extract type identifiers from an impl signature (uses RustParser if available,
/// but falls back to `TypeIdentifiers::from_impl_signature`).
pub fn extract_type_identifiers(impl_signature: &ImplSignature) -> TypeIdentifiers {
    // Use the impl signature text as the parser source
    let source = impl_signature.to_string();

    match RustParser::new(&source, "type_identifier") {
        Ok(parser) => {
            // parser may populate tokens into vec if needed by future implementations
            let mut vec: Vec<String> = Vec::new();
            parser.save_type_identifiers(&mut vec);

            // build TypeIdentifiers from the impl signature (existing constructor)
            TypeIdentifiers::from_impl_signature(impl_signature)
        }
        Err(err_str) => {
            eprintln!("RustParser::new failed for impl_signature: {}", err_str);

            // fallback behavior: still construct from signature
            TypeIdentifiers::from_impl_signature(impl_signature)
        }
    }
}

/// Extract the DS name from an impl signature and discovered type identifiers.
pub fn extract_ds_structure(impl_signature: &ImplSignature, type_ids: &TypeIdentifiers) -> DSName {
    // Helper to normalize a token into the identifier we want (strip generics/punctuation)
    fn normalize_token(token: &str) -> String {
        let first_piece = token
            .split(|c: char| {
                c == '<' || c == ',' || c == ':' || c == ';' || c == ')' || c == '{' || c == '('
            })
            .next()
            .unwrap_or(token);

        first_piece
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
            .to_string()
    }

    let source = impl_signature.as_str();

    // Step A: try to find "for" as a token and take next token
    {
        let tokens: Vec<&str> = source.split_whitespace().collect();
        if let Some(pos) = tokens.iter().position(|&t| t == "for") {
            if let Some(next_token) = tokens.get(pos + 1) {
                let candidate = normalize_token(next_token);
                if !candidate.is_empty() {
                    return candidate;
                }
            }
        }
    }

    // Quick return if exactly one concrete type was found
    if !type_ids.concrete_types.is_empty() && type_ids.concrete_types.len() == 1 {
        if let Some(single) = type_ids.concrete_types.iter().next() {
            return single.clone();
        }
    }

    // Step B: fallback using RustParser::delete_till_start("type_parameters")
    match RustParser::new(source, "type_parameters") {
        Ok(parser) => {
            match parser.delete_till_start("type_parameters") {
                Some(remainder) => {
                    if let Some(first_tok) = remainder.split_whitespace().next() {
                        let candidate = normalize_token(first_tok);
                        if !candidate.is_empty() {
                            return candidate;
                        }
                    }
                }
                None => { /* fallback to empty */ }
            }
        }
        Err(err_str) => {
            eprintln!("RustParser::new failed for ds_structure (fallback type_parameters): {}", err_str);
        }
    }

    // If nothing found, return empty string (caller can interpret as missing)
    String::new()
}
