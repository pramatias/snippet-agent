use clap::{ArgAction, Args, Parser, Subcommand};
use console::style;
use log::LevelFilter;

use anyhow::{Context, Result, anyhow};
// use regex::Regex;
use serde::Deserialize;
use std::collections::BTreeMap;

use regex::Regex;
use std::io::Write;
use tempfile::NamedTempFile;
use thiserror::Error;
use walkdir::WalkDir;

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

    /// Emit JSON as a newline-delimited stream
    #[arg(long = "json-stream", help = "Emit JSON as a newline-delimited stream")]
    pub json_stream: bool,
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

fn initialize_logger(level: LevelFilter) -> Result<()> {
    // Initialize env_logger and set the max level based on the CLI verbosity.
    let mut builder = env_logger::Builder::new();
    builder.filter(None, level);
    builder
        .try_init()
        .context("failed to initialize env_logger")?;
    Ok(())
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
    log::debug!(
        "{} {:?}",
        style("Logger initialized with verbosity:").cyan(),
        verbosity_level
    );

    log::debug!("{}", style("Default configuration loaded").green());

    match &cli.command {
        Mode::Method(args) => {
            let directory = args.directory.as_deref().unwrap_or("<auto-detected>");
            log::debug!("Running method mode: directory = {}", directory);

            if let Err(e) = run_method(args) {
                eprintln!("Error in method mode: {}", e);
                std::process::exit(1);
            }
        }
        Mode::Function(args) => {
            log::debug!("Running function mode: directory = {}", args.directory);
            if let Err(e) = run_function(args) {
                eprintln!("Error in function mode: {}", e);
                std::process::exit(1);
            }
        }
        Mode::Struct(args) => {
            log::debug!("Running struct mode: directory = {}", args.directory);
            if let Err(e) = run_struct(args) {
                eprintln!("Error in struct mode: {}", e);
                std::process::exit(1);
            }
        }
        Mode::StructMethods(args) => {
            log::debug!(
                "Running struct_methods mode: directory = {}",
                args.directory
            );
            if let Err(e) = run_struct_methods(args) {
                eprintln!("Error in struct_methods mode: {}", e);
                std::process::exit(1);
            }
        }
    }

    log::debug!("Execution completed successfully");
    Ok(())
}

/// Top-level runner: keep returning anyhow::Result for callers that expect
/// an opaque error type + rich context.
pub fn run_method(args: &MethodArgs) -> Result<()> {
    // Resolve path based on new priority:
    // --file > --directory > --crate > --root > cwd
    let resolved_path = if let Some(file) = args.file.as_ref() {
        let p = PathBuf::from(file);
        absolutize(&p)?
    } else if let Some(dir) = args.directory.as_ref() {
        let p = PathBuf::from(dir);
        absolutize(&p)?
    } else if args.crate_dir {
        let cwd = std::env::current_dir().map_err(|e| anyhow!("failed to get cwd: {}", e))?;
        find_nearest_project_root(&cwd)?
    } else if args.root {
        let cwd = std::env::current_dir().map_err(|e| anyhow!("failed to get cwd: {}", e))?;
        find_project_root(&cwd)?
    } else {
        std::env::current_dir().map_err(|e| anyhow!("failed to get cwd: {}", e))?
    };

    // Use provided method name if present, otherwise fall back to previous default.
    let method_name = args
        .name
        .as_deref()
        .unwrap_or("")
        .to_string();

    let finder = MethodFind::new(resolved_path.to_string_lossy().into_owned(), method_name);

    finder
        .query()
        .map_err(|e| anyhow!("MethodFind failed: {:#}", e))
}

#[derive(Debug, Deserialize)]
pub struct AstGrepMatch {
    /// the path to the matched file
    file: String,
    /// the matched text (ast-grep provides a `text` field in JSON)
    text: String,
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

/// MethodFind runs `ast-grep scan --rule <tempfile> <directory> --json` and
/// prints grouped-and-cleaned output similar to the jq pipeline you
/// provided.
pub struct MethodFind {
    pub directory: String,
    /// the method name to search (will be placed into the rule's regex)
    pub method: String,
}

impl MethodFind {
    pub fn new(directory: impl Into<String>, method: impl Into<String>) -> Self {
        MethodFind {
            directory: directory.into(),
            method: method.into(),
        }
    }

    /// Run the full pipeline: find matches then print them.
    pub fn query(&self) -> Result<(), MethodFindError> {
        let items = self.find_matches()?;
        if items.is_empty() {
            return Ok(());
        }
        self.print_matches(&items)
    }

    /// Run ast-grep and return a Vec of AstGrepMatch.
    pub fn find_matches(&self) -> Result<Vec<AstGrepMatch>, MethodFindError> {
        // Build the YAML rule template
        // The template contains a `<METHOD>` placeholder that
        // will be replaced with an escaped regex for the method name at runtime.
        let rule_template = r#"id: find-foo
language: rust
rule:
  all:
    - kind: function_item
    - inside:
        kind: impl_item
        stopBy: end
    - has:
        kind: identifier
        field: name
        regex: '^<METHOD>'
"#
        .to_string();

        // Escape the method to safely embed in a regex.
        let escaped = regex::escape(&self.method);
        let regex_pattern = format!("^{}", escaped);

        // Replace the placeholder in the template with the actual regex.
        let yaml = rule_template.replace("<METHOD>", &regex_pattern);

        // Create a named temp file and write the YAML to it.
        let mut tmp: NamedTempFile = NamedTempFile::new()?; // maps to MethodFindError::Io
        tmp.write_all(yaml.as_bytes())?; // maps to MethodFindError::Io

        // Ensure we have a valid path to pass to ast-grep.
        let rule_path = tmp
            .path()
            .to_str()
            .ok_or(MethodFindError::InvalidTempPath)?
            .to_string();

        // Run ast-grep with the temporary rule file using duct.
        let stdout = duct::cmd(
            "ast-grep",
            ["scan", "--rule", &rule_path, &self.directory, "--json"],
        )
        .read()
        .map_err(|e| MethodFindError::Duct(e.to_string()))?; // maps to MethodFindError::Duct

        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            // nothing found -> return empty vec
            return Ok(Vec::new());
        }

        // ast-grep's JSON can be either a JSON array or newline-delimited
        // JSON objects. Try to handle both using iterator combinators.
        let items: Vec<AstGrepMatch> = if trimmed.starts_with('[') {
            serde_json::from_str(trimmed)? // maps to MethodFindError::Json
        } else {
            // Trim lines, drop empty ones, parse each JSON line, collect into Vec.
            trimmed
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| serde_json::from_str::<AstGrepMatch>(line))
                .collect::<Result<Vec<_>, _>>()? // maps to MethodFindError::Json
        };

        Ok(items)
    }

    /// Print a set of AstGrepMatch items in the grouped/cleaned format.
    pub fn print_matches(&self, items: &[AstGrepMatch]) -> Result<(), MethodFindError> {
        // Prepare regex to collapse whitespace.
        let ws_re = Regex::new(r"\s+")?; // maps to MethodFindError::Regex

        // Sort by file so grouping is stable. Work with references to avoid cloning.
        let mut refs: Vec<&AstGrepMatch> = items.iter().collect();
        refs.sort_by(|a, b| a.file.cmp(&b.file));

        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for it in refs {
            let before_brace = it.text.split('{').next().unwrap_or("");
            let replaced_newlines = before_brace.replace('\n', " ");
            let collapsed = ws_re
                .replace_all(&replaced_newlines, " ")
                .trim()
                .to_string();

            groups.entry(it.file.clone()).or_default().push(collapsed);
        }

        // Print groups in the style of your jq pipeline: file path on its own
        // line followed by each cleaned match on subsequent lines.
        for (file, texts) in groups {
            println!("{}", file);
            for t in texts {
                println!("  {}", t);
            }
        }

        Ok(())
    }
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
    let start = start.as_ref().canonicalize().map_err(|e| {
        anyhow!(
            "failed to canonicalize start path '{}': {}",
            start.as_ref().display(),
            e
        )
    })?;

    // Keep only ancestors that contain Cargo.toml (checked by inspecting immediate entries),
    // then take the last one (farthest from the start, i.e. nearest to root).
    start
        .ancestors()
        .filter(|ancestor| {
            WalkDir::new(ancestor)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|entry| {
                    let p = entry.path();
                    p.is_file() && p.file_name().and_then(|s| s.to_str()) == Some("Cargo.toml")
                })
        })
        .last()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| {
            anyhow!(
                "could not find Cargo.toml in '{}' or any of its parents",
                start.display()
            )
        })
}

/// Find the *nearest* ancestor (the first one encountered going up) that contains Cargo.toml.
pub fn find_nearest_project_root(start: impl AsRef<Path>) -> Result<PathBuf> {
    let start = start.as_ref().canonicalize().map_err(|e| {
        anyhow!(
            "failed to canonicalize start path '{}': {}",
            start.as_ref().display(),
            e
        )
    })?;

    start
        .ancestors()
        .find(|ancestor| {
            WalkDir::new(ancestor)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|entry| {
                    let p = entry.path();
                    p.is_file() && p.file_name().and_then(|s| s.to_str()) == Some("Cargo.toml")
                })
        })
        .map(|p| p.to_path_buf())
        .ok_or_else(|| {
            anyhow!(
                "could not find Cargo.toml in '{}' or any of its parents",
                start.display()
            )
        })
}

fn normalize_path(p: PathBuf) -> PathBuf {
    let mut stack: Vec<OsString> = Vec::new();

    for comp in p.components() {
        match comp {
            Component::CurDir => {} // skip "."
            Component::ParentDir => {
                // pop a previous normal segment if possible, otherwise keep ".."
                if let Some(last) = stack.last() {
                    if *last != OsString::from("..")
                        && last.as_os_str() != std::path::MAIN_SEPARATOR.to_string().as_str()
                    {
                        stack.pop();
                    } else {
                        stack.push(OsString::from(".."));
                    }
                } else {
                    stack.push(OsString::from(".."));
                }
            }
            other => stack.push(other.as_os_str().to_os_string()),
        }
    }

    let mut out = PathBuf::new();
    for part in stack {
        out.push(part);
    }
    out
}

fn absolutize(path: &Path) -> anyhow::Result<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("failed to get cwd: {}", e))?
            .join(path)
    };
    Ok(normalize_path(joined))
}
