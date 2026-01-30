mod init;

use init::initialize_logger;
use clap::{ArgAction, Args, Parser, Subcommand};
use console::style;
use log::LevelFilter;

use anyhow::{Context, Result, anyhow};
// use regex::Regex;
use serde::Deserialize;

// use regex::Regex;
use std::io::Write;
use tempfile::NamedTempFile;
use thiserror::Error;
use walkdir::WalkDir;
use log::{error, debug};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

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
    debug!(
        "{} {:?}",
        style("Logger initialized with verbosity:").cyan(),
        verbosity_level
    );

    debug!("{}", style("Default configuration loaded").green());

    match &cli.command {
        Mode::Method(args) => {
            let directory = args.directory.as_deref().unwrap_or("<auto-detected>");
            debug!("Running method mode: directory = {}", directory);

            if let Err(e) = run_method(args) {
                eprintln!("Error in method mode: {}", e);
                std::process::exit(1);
            }
        }
        Mode::Function(args) => {
            debug!("Running function mode: directory = {}", args.directory);
            if let Err(e) = run_function(args) {
                eprintln!("Error in function mode: {}", e);
                std::process::exit(1);
            }
        }
        Mode::Struct(args) => {
            debug!("Running struct mode: directory = {}", args.directory);
            if let Err(e) = run_struct(args) {
                eprintln!("Error in struct mode: {}", e);
                std::process::exit(1);
            }
        }
        Mode::StructMethods(args) => {
            debug!(
                "Running struct_methods mode: directory = {}",
                args.directory
            );
            if let Err(e) = run_struct_methods(args) {
                eprintln!("Error in struct_methods mode: {}", e);
                std::process::exit(1);
            }
        }
    }

    debug!("Execution completed successfully");
    Ok(())
}

/// Top-level runner: keep returning anyhow::Result for callers that expect
/// an opaque error type + rich context.
pub fn run_method(args: &MethodArgs) -> Result<()> {
    debug!("Starting run_method with args: {:?}", args);

    // Resolve path based on new priority:
    // --file > --directory > --crate > --root > cwd
    let resolved_path = if let Some(file) = args.file.as_ref() {
        debug!("Using --file argument: {}", file);
        let p = PathBuf::from(file);
        let abs_path = absolutize(&p)?;
        debug!("Absolute file path: {:?}", abs_path);
        abs_path
    } else if let Some(dir) = args.directory.as_ref() {
        debug!("Using --directory argument: {}", dir);
        let p = PathBuf::from(dir);
        let abs_path = absolutize(&p)?;
        debug!("Absolute directory path: {:?}", abs_path);
        abs_path
    } else if args.crate_dir {
        debug!("Using --crate-dir flag, finding nearest project root");
        let cwd = std::env::current_dir().map_err(|e| anyhow!("failed to get cwd: {}", e))?;
        debug!("Current working directory: {:?}", cwd);
        let project_root = find_nearest_project_root(&cwd)?;
        debug!("Found nearest project root: {:?}", project_root);
        project_root
    } else if args.root {
        debug!("Using --root flag, finding project root");
        let cwd = std::env::current_dir().map_err(|e| anyhow!("failed to get cwd: {}", e))?;
        debug!("Current working directory: {:?}", cwd);
        let project_root = find_project_root(&cwd)?;
        debug!("Found project root: {:?}", project_root);
        project_root
    } else {
        debug!("No path arguments provided, using current working directory");
        let cwd = std::env::current_dir().map_err(|e| anyhow!("failed to get cwd: {}", e))?;
        debug!("Current working directory: {:?}", cwd);
        cwd
    };

    debug!("Resolved path to search: {:?}", resolved_path);

    // Use provided method name if present, otherwise fall back to previous default.
    let method_name = args.name.as_deref().unwrap_or("").to_string();
    debug!("Method name to search for: '{}'", method_name);

    debug!("Initializing MethodFind with path: {}", resolved_path.to_string_lossy());
    let mut finder = MethodFind::new(resolved_path.to_string_lossy().into_owned(), method_name);

    debug!("Executing query...");
    finder
        .query()
        .map_err(|e| {
            error!("MethodFind query failed: {:#}", e);
            anyhow!("MethodFind failed: {:#}", e)
        })
        .map(|result| {
            debug!("Query completed successfully");
            result
        })
}

/// Type aliases for clarity
pub type FilePath = String;
pub type MethodText = String; // the full method text (first secondary)
pub type ImplText = String; // the whole impl of the method (second secondary)
pub type NameText = String; // the method identifier/name (third secondary)

/// Byte offset range container (maps to JSON `range.byteOffset`)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorByteOffsetRange {
    pub start: Option<u64>,
    pub end: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorRange {
    pub byte_offset: Option<SelectorByteOffsetRange>,
}

/// A single "secondary" metavariable entry under metaVariables.multi.secondary
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorSecondary {
    pub text: Option<String>,
    pub range: Option<SelectorRange>,
}

/// The `multi` object under metaVariables
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorMulti {
    pub secondary: Option<Vec<SelectorSecondary>>,
}

/// The metaVariables top-level object (we only care about `multi` here)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorMetaVariables {
    pub multi: Option<SelectorMulti>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorAstGrepMatch {
    pub file: String,
    /// top-level text (the node's text)
    pub text: Option<String>,
    /// metavariables (we inspect multi.secondary and byte offsets)
    pub meta_variables: Option<SelectorMetaVariables>,
}

/// Resulting byte-range we expose (simple copy of SelectorByteOffsetRange)
pub type ByteRange = SelectorByteOffsetRange;

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

/// The new, explicit AstGrepMatch with ordered fields and byte ranges
#[derive(Debug, Clone)]
pub struct AstGrepMatch {
    pub file: FilePath,
    /// the node's top-level text
    pub top_text: Option<String>,

    /// impl block text and its byte range (now non-optional)
    pub impl_text: String,
    pub impl_range: ByteRange,

    /// method / function body text and its byte range (now non-optional)
    pub method_text: String,
    pub method_range: ByteRange,
    // pub name_text: String,
    // pub name_range: ByteRange,
}

impl MethodFind {
    /// Find matches, store them in `self.matches`, and return a deterministic Vec
    /// of AstGrepMatch items.
    pub fn find_matches(&mut self) -> Result<Vec<AstGrepMatch>, MethodFindError> {}
    pub fn print_matches(&self, items: &[AstGrepMatch]) -> Result<(), MethodFindError> {}
}

fn extract_texts_by_offset(
    selector: &SelectorAstGrepMatch,
) -> Option<((String, Option<ByteRange>), (String, Option<ByteRange>))> {
}

/// Returns a deterministic empty ByteRange used as a fallback when the real
/// byte range wasn't present in the ast-grep output.
///
/// Adjust this to match your `ByteRange` definition if its fields differ.
fn empty_byte_range() -> ByteRange {
}

///print
impl MethodFind {
    /// Print matches grouped by file with deterministic ordering.
}

/// Find the project root by searching *upwards* from `start` (inclusive),
/// checking each ancestor for a `Cargo.toml`. If multiple ancestors contain
/// a `Cargo.toml`, the farthest ancestor (closest to filesystem root) wins
/// — i.e. the "last" directory containing Cargo.toml.
///
/// Implementation uses `walkdir` to inspect entries at each ancestor with
/// `max_depth(1)` (the user requested `walkdir`-based implementation).
/// Find the *farthest* ancestor (closest to the filesystem root) that contains Cargo.toml.
pub fn find_project_root(start: impl AsRef<Path>) -> Result<PathBuf> {
}

/// Find the *nearest* ancestor (the first one encountered going up) that contains Cargo.toml.
pub fn find_nearest_project_root(start: impl AsRef<Path>) -> Result<PathBuf> {
}

fn normalize_path(p: PathBuf) -> PathBuf {
}

fn absolutize(path: &Path) -> anyhow::Result<PathBuf> {
}

impl MethodFind {
    /// Create a new MethodFind. `matches` starts empty.
    pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {
    }
}
impl MethodFind {
    /// Run the full pipeline: find matches then print them.
    pub fn query(&mut self) -> Result<(), MethodFindError> {
    }
}

#[derive(Debug, Error)]
pub enum MethodFindError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("execution error from `duct`: {0}")]
    Duct(String), // capture duct errors as strings (or save the std::io::Error directly)

    #[error("failed to parse JSON from ast-grep: {0}")]
    Json(#[from] serde_json::Error),

    #[error("temporary rule file path is not valid UTF-8")]
    InvalidTempPath,
}

fn run_ast_grep_rule(method: &str, directory: &str) -> Result<String, MethodFindError> {
}
