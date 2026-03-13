mod ast_grep;
mod init;
mod json_selection;
mod path;
mod syn;

use clap::{ArgAction, Args, Parser, Subcommand};
use init::init::initialize_logger;
use log::LevelFilter;

use crate::syn::syn_element::FilePath;
use anyhow::{Context, Result, anyhow};
use log::error;

use crate::path::path_resolution::read_rs_files;
use crate::syn::syn_elements::AllSynElements;
use std::collections::HashMap;
use std::sync::Arc;

use crate::ast_grep::ast_grep_yaml_rules::run_ast_grep_rule;
use crate::json_selection::unprocessed_elements::AllUnprocessedElements;
use crate::path::path_resolution::resolve_path;
// use crate::syn::file_syn_elements::FileSynElements;
// use crate::syn::file_syn_elements::FileSynElements;
use crate::syn::file_syn_elements::AnyFileSynElement;
use crate::syn::file_syn_elements::FileSynElementsMap;
use crate::syn::file_syn_elements_tree::filter_elements;

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

//main.rs
fn run_struct(args: &StructArgs) -> Result<()> {
    let ag_dir = resolve_path(
        args.file.clone(),
        args.directory.clone(),
        args.crate_dir,
        args.root,
    )
    .map_err(|e| anyhow!("Path resolution failed: {}", e))?
    .to_string_lossy()
    .into_owned();

    let ag_json =
        run_ast_grep_rule(&ag_dir).map_err(|e| anyhow!("ast-grep run failed: {:?}", e))?;

    let all_unprocessed = AllUnprocessedElements::from_raw_json(&ag_json)
        .map_err(|e| anyhow!("Failed parsing ast-grep JSON: {}", e))?;
    drop(ag_json);

    let file_contents: HashMap<FilePath, String> = read_rs_files(
        args.file.clone(),
        args.directory.clone(),
        args.crate_dir,
        args.root,
    )
    .map_err(|e| anyhow!("Failed reading source files: {}", e))?
    .into_iter()
    .map(|(contents, path)| (Arc::from(path.as_str()), contents))
    .collect();

    let mut all_syn = AllSynElements::from_unprocessed(all_unprocessed, &file_contents);

    all_syn.pick_blanket_impls(&file_contents);
    all_syn.filter_by_tree_in_place(3, 2);

    all_syn.print_attributes(&file_contents);
    all_syn.print_methods(&file_contents);
    all_syn.print_structs(&file_contents);

    Ok(())
}

fn run_method(args: &MethodArgs) -> Result<String> {
    let ag_dir = resolve_path(
        args.file.clone(),
        args.directory.clone(),
        args.crate_dir,
        args.root,
    )
    .map_err(|e| {
        error!("resolve_path error: {}", e);
        anyhow!("Path resolution failed: {}", e)
    })?
    .to_string_lossy()
    .into_owned();

    let ag_json = run_ast_grep_rule(&ag_dir).map_err(|e| {
        error!("run_ast_grep_rule failed: {:?}", e);
        anyhow!("ast-grep run failed: {:?}", e)
    })?;

    let all_unprocessed = AllUnprocessedElements::from_raw_json(&ag_json).map_err(|e| {
        error!("AllSynElements::from_json failed: {}", e);
        anyhow!("Failed parsing ast-grep JSON: {}", e)
    })?;
    drop(ag_json);

    let file_contents: HashMap<FilePath, String> = read_rs_files(
        args.file.clone(),
        args.directory.clone(),
        args.crate_dir,
        args.root,
    )
    .map_err(|e| anyhow!("Failed reading source files: {}", e))?
    .into_iter()
    .map(|(contents, path)| (Arc::from(path.as_str()), contents))
    .collect();

    let mut all_syn = AllSynElements::from_unprocessed(all_unprocessed, &file_contents);

    all_syn.pick_blanket_impls(&file_contents);

    let mut map = FileSynElementsMap::from_all_syn_elements(&all_syn);
    map.filter_by_tree(3, 2);

    let all_filtered = AllSynElements::from_file_syn_elements_map(map);
    all_filtered.print_attributes(&file_contents);
    all_filtered.print_methods(&file_contents);
    all_filtered.print_impls(&file_contents);
    all_filtered.print_structs(&file_contents);
    all_filtered.print_traits(&file_contents);
    all_filtered.print_functions(&file_contents);
    all_filtered.print_tests_mods(&file_contents);
    all_filtered.print_enums(&file_contents);
    all_filtered.print_unions(&file_contents);
    all_filtered.print_type_aliases(&file_contents);
    all_filtered.print_trait_method_sigs(&file_contents);
    all_filtered.print_trait_method_defs(&file_contents);

    Ok("found method".to_string())
}

fn run_function(args: &FunctionArgs) -> Result<()> {
    let ag_dir = resolve_path(
        args.file.clone(),
        args.directory.clone(),
        args.crate_dir,
        args.root,
    )
    .map_err(|e| anyhow!("Path resolution failed: {}", e))?
    .to_string_lossy()
    .into_owned();

    let ag_json =
        run_ast_grep_rule(&ag_dir).map_err(|e| anyhow!("ast-grep run failed: {:?}", e))?;

    let all_unprocessed = AllUnprocessedElements::from_raw_json(&ag_json)
        .map_err(|e| anyhow!("Failed parsing ast-grep JSON: {}", e))?;
    drop(ag_json);

    let file_contents: HashMap<FilePath, String> = read_rs_files(
        args.file.clone(),
        args.directory.clone(),
        args.crate_dir,
        args.root,
    )
    .map_err(|e| anyhow!("Failed reading source files: {}", e))?
    .into_iter()
    .map(|(contents, path)| (Arc::from(path.as_str()), contents))
    .collect();

    let mut all_syn = AllSynElements::from_unprocessed(all_unprocessed, &file_contents);

    all_syn.pick_blanket_impls(&file_contents);

    let mut map = FileSynElementsMap::from_all_syn_elements(&all_syn);
    map.filter_by_tree(3, 2);

    let all_filtered = AllSynElements::from_file_syn_elements_map(map);
    all_filtered.print_attributes(&file_contents);
    all_filtered.print_methods(&file_contents);
    all_filtered.print_impls(&file_contents);
    all_filtered.print_structs(&file_contents);
    all_filtered.print_traits(&file_contents);
    all_filtered.print_functions(&file_contents);
    all_filtered.print_tests_mods(&file_contents);
    all_filtered.print_enums(&file_contents);
    all_filtered.print_unions(&file_contents);
    all_filtered.print_type_aliases(&file_contents);
    all_filtered.print_trait_method_sigs(&file_contents);
    all_filtered.print_trait_method_defs(&file_contents);

    Ok(())
}
