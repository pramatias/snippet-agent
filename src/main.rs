mod ast_grep_selection;
mod impl_sig_types;
mod init;
mod method_find;
mod path_resolution;

use clap::{ArgAction, Args, Parser, Subcommand};
use init::initialize_logger;
use log::LevelFilter;

use anyhow::{Context, Result, anyhow};

use log::{debug, error};

use crate::ast_grep_selection::ByteRange;
use crate::impl_sig_types::*;
use crate::method_find::*;
use crate::method_find::{FilePath, ImplBody, MethodBody, MethodName};
use crate::path_resolution::resolve_path;
use syntax_queries::RustParser;

/// Configure logging verbosity using -v/--verbose and -q/--quiet flags.
#[derive(Args, Debug)]
pub struct Verbosity {
    /// Increase the level of verbosity (repeatable).
    #[arg(short = 'v', long, action = ArgAction::Count, display_order = 99)]
    pub verbose: u8,

    /// Decrease the level of verbosity (repeatable).
    #[arg(short = 'q', long, action = ArgAction::Count, display_order = 100)]
    pub quiet: u8,
}

impl Verbosity {
    pub fn log_level_filter(&self) -> LevelFilter {
        if self.quiet > 0 {
            LevelFilter::Warn
        } else {
            match self.verbose {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            }
        }
    }
}

/// Mode-line interface for the module management tool.
#[derive(Parser, Debug)]
#[command(
    author = "you",
    version = "0.1",
    about = "Template CLI with directory modes"
)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity,

    #[command(subcommand)]
    command: Mode,
}

/// CLI modes (each takes a directory parameter).
#[derive(Subcommand, Debug)]
enum Mode {
    /// Inspect/transform a method in the given directory
    Method(MethodArgs),

    /// Inspect/transform a free function in the given directory
    Function(FunctionArgs),

    /// Inspect/transform a `struct` in the given directory
    Struct(StructArgs),

    /// Inspect/transform methods of a `struct` in the given directory
    StructMethods(StructMethodsArgs),
}

#[derive(Args, Debug)]
pub struct MethodArgs {
    /// File to process (highest priority)
    #[arg(short = 'n', long = "name", help = "Method name")]
    pub name: Option<String>,

    /// File to process (highest priority)
    #[arg(short = 'f', long = "file", help = "File to inspect/process")]
    pub file: Option<String>,

    /// Crate/directory to process
    #[arg(
        short = 'c',
        long = "crate",
        help = "Crate or directory to inspect/process"
    )]
    pub crate_dir: bool,

    /// Use the project root (auto-detected by walking up to Cargo.toml)
    #[arg(short = 'r', long = "root", help = "Use project root (auto-detected)")]
    pub root: bool,

    /// Directory to process (overrides auto-detection)
    #[arg(short = 'd', long = "directory", help = "Directory to inspect/process")]
    pub directory: Option<String>,

    /// Emit JSON
    #[arg(long = "json", help = "Emit JSON")]
    pub json: bool,
}

#[derive(Args, Debug)]
struct FunctionArgs {
    /// Directory to process
    #[arg(short = 'd', long = "directory", help = "Directory to inspect/process")]
    directory: String,
}

#[derive(Args, Debug)]
struct StructArgs {
    /// Directory to process
    #[arg(short = 'd', long = "directory", help = "Directory to inspect/process")]
    directory: String,
}

#[derive(Args, Debug)]
struct StructMethodsArgs {
    /// Directory to process
    #[arg(short = 'd', long = "directory", help = "Directory to inspect/process")]
    directory: String,
}

fn run_function(args: &FunctionArgs) -> Result<()> {
    // TODO: implement function mode
    todo!("run_function for directory = {}", args.directory);
}

fn run_struct(args: &StructArgs) -> Result<()> {
    // TODO: implement struct mode
    todo!("run_struct for directory = {}", args.directory);
}

fn run_struct_methods(args: &StructMethodsArgs) -> Result<()> {
    // TODO: implement struct_methods mode
    todo!("run_struct_methods for directory = {}", args.directory);
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let verbosity_level = cli.verbose.log_level_filter();
    initialize_logger(verbosity_level).context("Failed to initialize logger")?;

    match &cli.command {
        Mode::Method(args) => {
            if let Err(e) = run_method(args) {
                eprintln!("Error in method mode: {}", e);
                std::process::exit(1);
            }
        }
        Mode::Function(args) => {
            if let Err(e) = run_function(args) {
                eprintln!("Error in function mode: {}", e);
                std::process::exit(1);
            }
        }
        Mode::Struct(args) => {
            if let Err(e) = run_struct(args) {
                eprintln!("Error in struct mode: {}", e);
                std::process::exit(1);
            }
        }
        Mode::StructMethods(args) => {
            if let Err(e) = run_struct_methods(args) {
                eprintln!("Error in struct_methods mode: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

/// `run_method` uses `MethodDataMatch::from_ast_grep_match` to
/// create each match. The initializer takes an `AstGrepMatch`, initializes the
/// common fields, and runs the existing `extract_signatures_impl_fn` method twice to
/// populate the two signature fields.
fn run_method(args: &MethodArgs) -> Result<MethodData> {
    debug!("Starting run_method with args: {:?}", args);

    // Call resolve_path with each MethodArgs field passed explicitly.
    let resolved_path = resolve_path(
        args.file.clone(),
        args.directory.clone(),
        args.crate_dir,
        args.root,
    )
    .map_err(|e| {
        error!("resolve_path error: {}", e);
        anyhow!("Path resolution failed: {}", e)
    })?;
    debug!("Resolved path to search: {:?}", resolved_path);

    // Use provided method name if present, otherwise fall back to previous default.
    let method_name = args.name.clone().unwrap_or_default();
    debug!("Method name to search for: '{}'", method_name);

    debug!(
        "Initializing MethodFind with path: {}",
        resolved_path.to_string_lossy()
    );
    let mut finder = MethodFind::new(
        resolved_path.to_string_lossy().into_owned(),
        method_name.clone(),
    );

    debug!("Executing query...");
    // Run the actual query (this populates finder.matches)
    finder.query().map_err(|e| {
        error!("MethodFind query failed: {:#}", e);
        anyhow!("MethodFind failed: {:#}", e)
    })?;

    // Now hand the raw AstGrepMatch vector to MethodData::new and let it
    // construct MethodDataMatch entries and initialize signatures.
    let method_data = MethodData::new(method_name, finder.matches.clone());

    method_data.print_all();
    debug!(
        "run_method produced MethodData with {} matches",
        method_data.matches.len()
    );

    Ok(method_data)
}

/// New signature type aliases requested
pub type ImplSignature = String;
pub type FunctionSignature = String;
pub type DSName = String;

/// New per-match struct that mirrors AstGrepMatch but adds two extra fields.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MethodDataMatch {
    pub file: FilePath,
    pub method_body: MethodBody,
    pub method_body_range: ByteRange,

    pub impl_body: ImplBody,
    pub impl_body_range: ByteRange,

    pub method_name: MethodName,
    pub method_name_range: ByteRange,

    pub impl_signature: ImplSignature,
    pub function_signature: FunctionSignature,
    pub ds_structure: DSName,
    pub type_identifiers: TypeIdentifiers,
}

/// The top-level MethodData that will be produced by run_method.
/// Contains metadata (directory + method) and the vector of matches.
#[derive(Debug, Clone)]
pub struct MethodData {
    pub method: String,
    pub matches: Vec<MethodDataMatch>,
}

///new
impl MethodData {
    pub fn new(method: String, matches: Vec<AstGrepMatch>) -> Self {
        let mut mapped_matches: Vec<MethodDataMatch> = Vec::with_capacity(matches.len());

        for ast_match in matches.into_iter() {
            // 1) extract signatures from the textual pieces (pure functions, now split)
            let fn_sig = MethodDataMatch::extract_function_signature(&ast_match.method_body);
            let impl_sig = MethodDataMatch::extract_impl_signature(&ast_match.impl_body);

            // 2) extract type identifiers using the new static helper that returns TypeIdentifiers
            let type_ids = MethodDataMatch::extract_type_identifiers(&impl_sig);

            // 3) DS extraction that may depend on signatures/types
            // <-- pass the discovered type_ids so extract_ds_structure may consult them
            let ds = MethodDataMatch::extract_ds_structure(&impl_sig, &type_ids);

            // 4) construct the MethodDataMatch with everything already computed
            let md =
                MethodDataMatch::from_ast_grep_match(ast_match, impl_sig, fn_sig, ds, type_ids);

            mapped_matches.push(md);
        }

        MethodData {
            method,
            matches: mapped_matches,
        }
    }
}

///print all
impl MethodData {
    /// Print brief information about this MethodData and each match.
    /// For method_body and impl_body we only show the first two lines and append `..`.
    pub fn print_all(&self) {
        println!("Method: {}", self.method);
        println!("Matches: {}", self.matches.len());
        for (idx, m) in self.matches.iter().enumerate() {
            println!("\n--- Match #{} ---", idx + 1);
            println!("File: {}\n", m.file);

            println!("DS structure: {}", m.ds_structure);
            println!("Impl signature: {}", m.impl_signature);
            println!("Type identifiers: {:?}\n", m.type_identifiers);

            println!("Method name: {}", m.method_name);
            println!("Method signature: {}", m.function_signature);
            println!("Method body (preview):\n{}\n", Self::preview_first_two_lines(&m.method_body));

            // println!("Method name range: {:?}", m.method_name_range);
            println!("Impl body (preview):\n{}", Self::preview_first_two_lines(&m.impl_body));
            // println!("Impl body range: {:?}", m.impl_body_range);
            // println!("Method body range: {:?}", m.method_body_range);
        }
    }

    /// Return a String containing the first two lines of `s` (joined by `\n`)
    /// followed by `..`. If `s` has fewer than two lines, returns whatever lines
    /// exist and still appends `..` (per your request).
    fn preview_first_two_lines(s: &str) -> String {
        let mut lines = s.lines();
        let mut picked = Vec::with_capacity(2);
        for _ in 0..2 {
            if let Some(l) = lines.next() {
                picked.push(l);
            } else {
                break;
            }
        }
        let mut out = picked.join("\n");
        out.push_str("..");
        out
    }
}

///extract ds structure
impl MethodDataMatch {
    /// Extract the data-structure (DS) name from an impl signature.
    /// Strategy:
    /// 0) If the provided TypeIdentifiers has exactly one concrete type, return it immediately.
    /// 1) Try to find the token "for" and return the next token (stripped of any '<' etc).
    /// 2) If "for" is not present, use RustParser::delete_till_start targeting "type_parameters"
    ///    and take the first token from the resulting remainder.
    pub fn extract_ds_structure(impl_signature: &ImplSignature, type_ids: &TypeIdentifiers) -> DSName {
        // Local result to return
        let mut ds_structure: DSName = String::new();

        // Quick return on empty input
        let source = impl_signature.as_str();

        // --- New early-return based on discovered type identifiers ---
        // If there is exactly one concrete type encountered, return it immediately as the DS name.
        // Helper to normalize a token into the identifier we want (strip generics/ punctuation)
        fn normalize_token(token: &str) -> String {
            // cut at first '<' or other delimiter, then trim non-ident chars from both ends
            let first_piece = token
                .split(|c: char| c == '<' || c == ',' || c == ':' || c == ';' || c == ')' || c == '{' || c == '(')
                .next()
                .unwrap_or(token);

            first_piece
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '_') // remove stray punctuation
                .to_string()
        }

        // Step A: try to find "for" as a token and take next token
        {
            // split on whitespace to obtain tokens like ["impl", "From", "for", "Wrapper<T>"]
            let tokens: Vec<&str> = source.split_whitespace().collect();
            if let Some(pos) = tokens.iter().position(|&t| t == "for") {
                if let Some(next_token) = tokens.get(pos + 1) {
                    let candidate = normalize_token(next_token);
                    if !candidate.is_empty() {
                        return candidate;
                    } else {
                        // continue to fallback if normalization produced nothing
                        ds_structure.clear();
                    }
                }
            }
        }

        if !type_ids.concrete_types.is_empty() && type_ids.concrete_types.len() == 1 {
            // safe to unwrap: len == 1
            if let Some(single) = type_ids.concrete_types.iter().next() {
                return single.clone();
            }
        }

        // Step B: fallback - use RustParser::delete_till_start with "type_parameters"
        // The delete_till_start method is expected to return the remainder (text after the deleted part),
        // from which we can select the first token as the DS name.
        match RustParser::new(source, "type_parameters") {
            Ok(parser) => {
                match parser.delete_till_start("type_parameters") {
                    Some(remainder) => {
                        // first whitespace-delimited token from remainder
                        if let Some(first_tok) = remainder.split_whitespace().next() {
                            let candidate = normalize_token(first_tok);
                            if !candidate.is_empty() {
                                ds_structure = candidate;
                            } else {
                                ds_structure.clear();
                            }
                        } else {
                            ds_structure.clear();
                        }
                    }
                    None => {
                        // delete_till_start returned nothing meaningful
                        ds_structure.clear();
                    }
                }
            }
            Err(err_str) => {
                eprintln!(
                    "RustParser::new failed for ds_structure (fallback type_parameters): {}",
                    err_str
                );
                ds_structure.clear();
            }
        }

        ds_structure
    }
}

///extract function signature
impl MethodDataMatch {
    /// Extract the function signature (if any) from the provided method_body.
    /// Pure function: does not require nor mutate any instance.
    pub fn extract_function_signature(method_body: &MethodBody) -> FunctionSignature {
        let mut function_signature: FunctionSignature = String::new();

        // Parse the method body to get the function signature (if any)
        match RustParser::new(method_body, "block") {
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
}

///extract impl signature
impl MethodDataMatch {
    /// Extract the impl signature (if any) from the provided impl_body.
    /// Pure function: does not require nor mutate any instance.
    pub fn extract_impl_signature(impl_body: &ImplBody) -> ImplSignature {
        let mut impl_signature: ImplSignature = String::new();

        // Parse the impl body to get the impl signature (if any)
        match RustParser::new(impl_body, "declaration_list") {
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
}

///extract type identifiers
impl MethodDataMatch {
    /// Extract required types for this match using a RustParser method `save_type_identifiers`.
    /// Now a pure/static function that takes the file (for error messages) and the impl signature
    /// and returns a `TypeIdentifiers` instance.
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
}

///from ast grep match
impl MethodDataMatch {
    pub fn from_ast_grep_match(
        m: AstGrepMatch,
        impl_signature: ImplSignature,
        function_signature: FunctionSignature,
        ds_structure: DSName,
        type_identifiers: TypeIdentifiers,
    ) -> Self {
        MethodDataMatch {
            file: m.file,
            method_body: m.method_body,
            method_body_range: m.method_body_range,

            impl_body: m.impl_body,
            impl_body_range: m.impl_body_range,

            method_name: m.method_name,
            method_name_range: m.method_name_range,

            impl_signature,
            function_signature,
            ds_structure,
            type_identifiers,
        }
    }
}

/// New enum to pick which signature/parser to run.
pub enum SignatureTarget {
    Function,
    Impl,
}

#[cfg(test)]
mod tests {
    use super::*; // adjust if MethodData / AstGrepMatch live in another module
    use crate::ast_grep_selection::SelectorByteOffsetRange;
    use std::collections::HashMap;
    use std::collections::HashSet;

    #[test]
    fn methoddata_new_maps_fields_and_signatures() {
        // sample pieces used in other tests — keep them consistent with your helpers
        let impl_body = "impl std::fmt::Display for MyType {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        write!(f, \"MyType\")\n    }\n}";
        let method_body = "fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        write!(f, \"MyType\")\n    }";
        let method_name = "fmt";

        let ast_match = AstGrepMatch {
            file: "/ast-grep-test/sample_program.rs".into(),
            method_body: method_body.into(),
            method_body_range: SelectorByteOffsetRange {
                start: 651,
                end: 753,
            },
            impl_body: impl_body.into(),
            impl_body_range: SelectorByteOffsetRange {
                start: 611,
                end: 755,
            },
            method_name: method_name.into(),
            method_name_range: SelectorByteOffsetRange {
                start: 654,
                end: 657,
            },
        };

        // Call the constructor under test
        let md = MethodData::new(method_name.to_string(), vec![ast_match.clone()]);

        // Top-level expectations
        assert_eq!(md.method, method_name);
        assert_eq!(md.matches.len(), 1, "expected exactly one mapped match");

        // Validate the mapping into MethodDataMatch
        let mapped = &md.matches[0];

        // Fields copied through
        assert_eq!(mapped.file, ast_match.file);
        assert_eq!(mapped.method_body, ast_match.method_body);
        assert_eq!(mapped.impl_body, ast_match.impl_body);
        assert_eq!(mapped.method_name, method_name);

        // Signature extraction expectations
        let expected_impl_sig = "impl std::fmt::Display for MyType";
        let expected_fn_sig = "fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result";
        let expected_ds_structure = "MyType";

        assert_eq!(
            mapped.impl_signature, expected_impl_sig,
            "impl signature did not match expected"
        );
        assert_eq!(
            mapped.function_signature, expected_fn_sig,
            "function signature did not match expected"
        );
        assert_eq!(
            mapped.ds_structure, expected_ds_structure,
            "function signature did not match expected"
        );

        assert_eq!(
            mapped.type_identifiers.type_variables, None,
            "expected no generic type variables for a non-generic impl"
        );

        let expected_concrete: HashSet<String> = vec!["Display".to_string(), "MyType".to_string()]
            .into_iter()
            .collect();

        let actual_concrete: HashSet<String> = mapped
            .type_identifiers
            .concrete_types
            .iter()
            .map(|t| t.to_string())
            .collect();

        assert_eq!(
            actual_concrete, expected_concrete,
            "concrete types did not match expected (normalization removes module paths)"
        );
    }

    #[test]
    fn methoddata_new_maps_fields_and_signatures_with_generics() {
        // sample pieces used in other tests — keep them consistent with your helpers
        let impl_body = r#"impl<'a, T: Clone, const N: usize> Array<T, N> where T: std::fmt::Debug + Deserialize,{
    fn new_from_slice(_s: &'a [T]) -> Self { unimplemented!() }
}"#;
        let method_body = r#"fn new_from_slice(_s: &'a [T]) -> Self { unimplemented!() }"#;
        let method_name = "new_from_slice";

        let ast_match = AstGrepMatch {
            file: "/ast-grep-test/sample_program.rs".into(),
            method_body: method_body.into(),
            method_body_range: SelectorByteOffsetRange {
                start: 900,
                end: 980,
            },
            impl_body: impl_body.into(),
            impl_body_range: SelectorByteOffsetRange {
                start: 840,
                end: 1000,
            },
            method_name: method_name.into(),
            method_name_range: SelectorByteOffsetRange {
                start: 945,
                end: 958,
            },
        };

        // Call the constructor under test
        let md = MethodData::new(method_name.to_string(), vec![ast_match.clone()]);

        // Validate the mapping into MethodDataMatch
        let mapped = &md.matches[0];

        // Signature extraction expectations
        let expected_impl_sig = "impl<'a, T: Clone, const N: usize> Array<T, N> where T: std::fmt::Debug + Deserialize,";
        let expected_fn_sig = "fn new_from_slice(_s: &'a [T]) -> Self";
        let expected_ds_structure = "Array";

        assert_eq!(
            mapped.impl_signature, expected_impl_sig,
            "impl signature did not match expected"
        );
        assert_eq!(
            mapped.function_signature, expected_fn_sig,
            "function signature did not match expected"
        );
        assert_eq!(
            mapped.ds_structure, expected_ds_structure,
            "ds_structure did not match expected"
        );

        // Type variable expectations:
        // T should map to Clone, Debug, Deserialize
        let mut expected_type_vars: HashMap<String, HashSet<String>> = HashMap::new();
        let set_t = expected_type_vars
            .entry("T".into())
            .or_insert_with(HashSet::new);
        set_t.insert("Clone".into());
        set_t.insert("Debug".into());
        set_t.insert("Deserialize".into());

        assert_eq!(
            mapped.type_identifiers.type_variables, Some(expected_type_vars),
            "type variables did not match expected"
        );

        // Concrete types expected (order-agnostic):
        // - traits / types seen (Clone, Debug, Deserialize)
        // - the container name "Array"
        // Do NOT include the const parameter `N` or primitive `usize` as a concrete type.
        let expected_concrete: HashSet<String> = vec![
            "Clone".to_string(),
            "Array".to_string(),
            "Debug".to_string(),
            "Deserialize".to_string(),
        ]
        .into_iter()
        .collect();

        let actual_concrete: HashSet<String> = mapped
            .type_identifiers
            .concrete_types
            .iter()
            .map(|t| t.to_string())
            .collect();

        assert_eq!(
            actual_concrete, expected_concrete,
            "concrete types did not match expected (normalization removes module paths)"
        );
    }

    #[test]
    fn mmethoddata_new_maps_fields_and_signaturesethoddata_new_maps_fields_and_signatures_with_generics_and_serde_de() {
        // new impl body (user-provided)
        let impl_body = r#"impl<T, const N: usize> Array<T, N> where T: Clone + std::fmt::Debug + serde::de::Deserialize<'de>,
{
    fn new_from_slice(_s: &'a [T]) -> Self { unimplemented!() }
}"#;
        let method_body = r#"fn new_from_slice(_s: &'a [T]) -> Self { unimplemented!() }"#;
        let method_name = "new_from_slice";

        let ast_match = AstGrepMatch {
            file: "/ast-grep-test/sample_program.rs".into(),
            method_body: method_body.into(),
            method_body_range: SelectorByteOffsetRange {
                start: 900,
                end: 980,
            },
            impl_body: impl_body.into(),
            impl_body_range: SelectorByteOffsetRange {
                start: 840,
                end: 1000,
            },
            method_name: method_name.into(),
            method_name_range: SelectorByteOffsetRange {
                start: 945,
                end: 958,
            },
        };

        // Call the constructor under test
        let md = MethodData::new(method_name.to_string(), vec![ast_match.clone()]);

        // Validate the mapping into MethodDataMatch
        let mapped = &md.matches[0];

        // Fields copied through
        assert_eq!(mapped.file, ast_match.file);
        assert_eq!(mapped.method_body, ast_match.method_body);
        assert_eq!(mapped.impl_body, ast_match.impl_body);
        assert_eq!(mapped.method_name, method_name);

        // Signature extraction expectations
        let expected_impl_sig = "impl<T, const N: usize> Array<T, N> where T: Clone + std::fmt::Debug + serde::de::Deserialize<'de>,";
        let expected_fn_sig = "fn new_from_slice(_s: &'a [T]) -> Self";
        let expected_ds_structure = "Array";

        assert_eq!(
            mapped.impl_signature, expected_impl_sig,
            "impl signature did not match expected"
        );
        assert_eq!(
            mapped.function_signature, expected_fn_sig,
            "function signature did not match expected"
        );
        assert_eq!(
            mapped.ds_structure, expected_ds_structure,
            "ds_structure did not match expected"
        );

        // Type variable expectations:
        // T should map to Clone, Debug, Deserialize (serde::de::Deserialize<'de> normalized to Deserialize)
        let mut expected_type_vars: HashMap<String, HashSet<String>> = HashMap::new();
        let set_t = expected_type_vars
            .entry("T".into())
            .or_insert_with(HashSet::new);
        set_t.insert("Clone".into());
        set_t.insert("Debug".into());
        set_t.insert("Deserialize".into());

        assert_eq!(
            mapped.type_identifiers.type_variables, Some(expected_type_vars),
            "type variables did not match expected"
        );

        // Concrete types expected (order-agnostic):
        // - traits / types seen (Clone, Debug, Deserialize)
        // - the container name "Array"
        // Do NOT include the const parameter `N` or primitive `usize` as a concrete type.
        let expected_concrete: HashSet<String> = vec![
            "Clone".to_string(),
            "Array".to_string(),
            "Debug".to_string(),
            "Deserialize".to_string(),
        ]
        .into_iter()
        .collect();

        let actual_concrete: HashSet<String> = mapped
            .type_identifiers
            .concrete_types
            .iter()
            .map(|t| t.to_string())
            .collect();

        assert_eq!(
            actual_concrete, expected_concrete,
            "concrete types did not match expected (normalization removes module paths)"
        );
    }

    // Helper to build a MethodDataMatch pre-populated with dummy strings and consistent ranges.
    fn make_dummy_methoddatamatch() -> MethodDataMatch {
        MethodDataMatch {
            file: "/ast-grep-test/sample_program.rs".into(),
            method_body: "fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        write!(f, \"MyType\")\n    }".into(),
            method_body_range: SelectorByteOffsetRange { start: 651, end: 753 },

            impl_body: "impl std::fmt::Display for MyType {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        write!(f, \"MyType\")\n    }\n}".into(),
            impl_body_range: SelectorByteOffsetRange { start: 611, end: 755 },

            method_name: "fmt".into(),
            method_name_range: SelectorByteOffsetRange { start: 654, end: 657 },

            // Populated signatures (dummy / illustrative)
            impl_signature: "impl std::fmt::Display for Wrapper".into(),
            function_signature: "fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result".into(),

            // initially empty; extract_ds_structure should populate this
            ds_structure: "Wrapper".into(),
            type_identifiers: TypeIdentifiers::from_impl_signature(&String::new()),
        }
    }

#[test]
fn methoddatamatch_extract_ds_structure_multiple_concrete_types() {
    // original test fixed: supply TypeIdentifiers with two concrete types (From, Wrapper)
    let mut m = make_dummy_methoddatamatch();
    m.impl_signature = "impl From for Wrapper".into();

    // build TypeIdentifiers: empty type_variables, concrete_types contains "From" and "Wrapper"
    let mut type_ids = TypeIdentifiers::default();
    type_ids.concrete_types.insert("From".to_string());
    type_ids.concrete_types.insert("Wrapper".to_string());
    // type_variables intentionally left empty (not useful for this test)

    // Call the new static function form with both parameters and check the returned DS name.
    let ds = MethodDataMatch::extract_ds_structure(&m.impl_signature, &type_ids);

    assert_eq!(ds, "Wrapper");

    // (Optional) If you want the instance to hold the result too:
    // m.ds_structure = ds.clone();
    // assert_eq!(m.ds_structure, "Wrapper");
}

#[test]
fn methoddatamatch_extract_ds_structure_single_concrete_type() {
    // new test: when concrete_types has exactly one element, the function should return it immediately
    let mut m = make_dummy_methoddatamatch();
    m.impl_signature = "impl Wrapper".into();

    // single concrete type -> early return
    let mut type_ids = TypeIdentifiers::default();
    type_ids.concrete_types.insert("Wrapper".to_string());
    // type_variables intentionally left empty

    let ds = MethodDataMatch::extract_ds_structure(&m.impl_signature, &type_ids);

    assert_eq!(ds, "Wrapper");
}

    // If SelectorByteOffsetRange is in another module, import it appropriately:
    // use crate::selector::SelectorByteOffsetRange;
    // helper (not a test)
    fn check_method_signature_extraction(
        method_body: &str,
        method_name: &str,
        impl_body: &str,
        expected_impl_sig: &str,
        expected_fn_sig: &str,
    ) {
        // Build a representative AstGrepMatch using the passed-in values
        let ast_match = AstGrepMatch {
            file: "/ast-grep-test/sample_program.rs".into(),
            method_body: method_body.into(),
            method_body_range: SelectorByteOffsetRange {
                start: 651,
                end: 753,
            },
            impl_body: impl_body.into(),
            impl_body_range: SelectorByteOffsetRange {
                start: 611,
                end: 755,
            },
            method_name: method_name.into(),
            method_name_range: SelectorByteOffsetRange {
                start: 654,
                end: 657,
            },
        };

        // Create a MethodData via the new(...) constructor that accepts Vec<AstGrepMatch>
        let md = MethodData::new(method_name.to_string(), vec![ast_match]);

        // Expect exactly one match
        assert_eq!(md.matches.len(), 1, "expected exactly one MethodDataMatch");

        let m = &md.matches[0];

        // Check extracted signatures
        assert_eq!(
            m.impl_signature, expected_impl_sig,
            "impl signature did not match expected"
        );
        assert_eq!(
            m.function_signature, expected_fn_sig,
            "function signature did not match expected"
        );

        // (Optional) Sanity-check other fields were copied as-is
        assert_eq!(m.method_name, method_name);
    }

    #[test]
    fn methoddata_signature_extraction() {
        let method_body = "fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        write!(f, \"MyType\")\n    }";
        let method_name = "fmt";
        let impl_body = "impl std::fmt::Display for MyType {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        write!(f, \"MyType\")\n    }\n}";
        let expected_impl_sig = "impl std::fmt::Display for MyType";
        let expected_fn_sig = "fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result";

        // Call the extracted helper with the parameters
        check_method_signature_extraction(
            method_body,
            method_name,
            impl_body,
            expected_impl_sig,
            expected_fn_sig,
        );
    }

    #[test]
    fn methoddata_signature_extraction_from() {
        let method_body = "fn from(t: T) -> Self { Wrapper(t) }";
        let method_name = "from";
        let impl_body = "impl<T> From<T> for Wrapper<T> where T: Sized, { fn from(t: T) -> Self { Wrapper(t) }}";
        let expected_impl_sig = "impl<T> From<T> for Wrapper<T> where T: Sized,";
        let expected_fn_sig = "fn from(t: T) -> Self";

        // Call the extracted helper with the parameters
        check_method_signature_extraction(
            method_body,
            method_name,
            impl_body,
            expected_impl_sig,
            expected_fn_sig,
        );
    }

    #[test]
    fn methoddata_signature_extraction_new_from_slice() {
        let method_body = "fn new_from_slice(_s: &'a [T]) -> Self { unimplemented!() }";
        let method_name = "new_from_slice";
        let impl_body = "impl<'a, T: Clone, const N: usize> Array<T, N>\nwhere\n    T: std::fmt::Debug,\n{\n    fn new_from_slice(_s: &'a [T]) -> Self { unimplemented!() }\n}";
        let expected_impl_sig =
            "impl<'a, T: Clone, const N: usize> Array<T, N>\nwhere\n    T: std::fmt::Debug,";
        let expected_fn_sig = "fn new_from_slice(_s: &'a [T]) -> Self";

        // Call the extracted helper with the parameters
        check_method_signature_extraction(
            method_body,
            method_name,
            impl_body,
            expected_impl_sig,
            expected_fn_sig,
        );
    }
}
