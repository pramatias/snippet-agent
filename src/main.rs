mod init;
mod path;
mod ast_grep;
mod json_selection;
mod syn;

use clap::{ArgAction, Args, Parser, Subcommand};
use init::init::initialize_logger;
use log::LevelFilter;

use anyhow::{Context, Result, anyhow};
use std::path::Path;
use log::{debug, error};
use crate::json_selection::unprocessed_elements::FilePath;
use std::collections::HashMap;
use crate::path::path_resolution::read_rs_files;
use crate::syn::syn_elements::AllSynElements;

// use crate::syn::impl_sig_types::*;
use crate::path::path_resolution::resolve_path;
use crate::json_selection::unprocessed_elements::AllUnprocessedElements;
use crate::ast_grep::ast_grep_yaml_rules::run_ast_grep_rule;

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

    let ag_dir = resolved_path.to_string_lossy().into_owned();

    let ag_json = run_ast_grep_rule(&ag_dir).map_err(|e| {
        error!("run_ast_grep_rule failed: {:?}", e);
        anyhow!("ast-grep run failed: {:?}", e)
    })?;

    let all_unprocessed = AllUnprocessedElements::from_raw_json(&ag_json).map_err(|e| {
        error!("AllUnprocessedElements::from_raw_json failed: {}", e);
        anyhow!("Failed parsing ast-grep JSON: {}", e)
    })?;

    let file_contents: HashMap<FilePath, String> = read_rs_files(
        args.file.clone(),
        args.directory.clone(),
        args.crate_dir,
        args.root,
    )
    .map_err(|e| {
        error!("read_rs_files failed: {}", e);
        anyhow!("Failed reading source files: {}", e)
    })?
    .into_iter()
    .map(|(contents, path)| (path, contents))
    .collect();

    let mut all_syn = AllSynElements::from_unprocessed(all_unprocessed, &file_contents);
    all_syn.pick_blanket_impls();

    all_syn.print_impls();
    all_syn.print_attributes();
    all_syn.print_tests_mods();
    all_syn.print_functions();
    all_syn.print_methods();
    all_syn.print_structs();
    all_syn.print_traits();
    all_syn.print_trait_method_sigs();
    all_syn.print_trait_method_defs();
    all_syn.print_type_aliases();
    all_syn.print_enums();
    all_syn.print_unions();

    Ok(())
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
pub fn run_method(args: &MethodArgs) -> Result<String> {
    // Resolve path as before
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

    // Use provided method name or empty string.
    let _method_name = args.name.clone().unwrap_or_default();

    let ag_dir = resolved_path.to_string_lossy().into_owned();

    // Run ast-grep rule, get JSON
    let ag_json = run_ast_grep_rule(&ag_dir).map_err(|e| {
        error!("run_ast_grep_rule failed: {:?}", e);
        anyhow!("ast-grep run failed: {:?}", e)
    })?;

    // Parse JSON into AllSynElements
    let all_syn = AllUnprocessedElements::from_raw_json(&ag_json).map_err(|e| {
        error!("AllSynElements::from_json failed: {}", e);
        anyhow!("Failed parsing ast-grep JSON: {}", e)
    })?;

    // all_syn.print_all();

    Ok("found method".to_string())
}

