use anyhow::{Result, anyhow};
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

/// Top-level runner: keep returning anyhow::Result for callers that expect
/// an opaque error type + rich context.
#[derive(Debug, Error)]
pub enum PathResolveError {
    #[error(
        "multiple path arguments provided: {provided:?}. Provide exactly one of --file, --directory, --crate, or --root."
    )]
    MultiplePathArgs { provided: Vec<String> },

    #[error("{0}")]
    Other(String),
}

/// Enum used internally to decide which path source to use.
#[derive(Debug)]
enum PathSource {
    File(PathBuf),
    Directory(PathBuf),
    CrateDir,
    Root,
    Cwd,
}

/// New signature: four explicit parameters (file, directory, crate_dir flag, root flag).
/// Returns a PathBuf or a PathResolveError (thiserror).
pub fn resolve_path(
    file: Option<String>,
    directory: Option<String>,
    crate_dir: bool,
    root: bool,
) -> Result<PathBuf, PathResolveError> {

    // Collect which parameters were provided so we can error on multiple being set.
    let mut provided = Vec::new();
    if file.is_some() {
        provided.push("file".to_string());
    }
    if directory.is_some() {
        provided.push("directory".to_string());
    }
    if crate_dir {
        provided.push("crate_dir".to_string());
    }
    if root {
        provided.push("root".to_string());
    }

    if provided.len() > 1 {
        return Err(PathResolveError::MultiplePathArgs { provided });
    }

    // Build the PathSource enum by pattern matching the parameters.
    let source = if let Some(f) = file {
        PathSource::File(PathBuf::from(f))
    } else if let Some(d) = directory {
        PathSource::Directory(PathBuf::from(d))
    } else if crate_dir {
        PathSource::CrateDir
    } else if root {
        PathSource::Root
    } else {
        PathSource::Cwd
    };

    // Pattern match on the chosen source to produce the final PathBuf.
    match source {
        PathSource::File(p) => {
            absolutize(&p)
                .map_err(|e| PathResolveError::Other(format!("absolutize(file) failed: {}", e)))
        }

        PathSource::Directory(p) => {
            absolutize(&p).map_err(|e| {
                PathResolveError::Other(format!("absolutize(directory) failed: {}", e))
            })
        }

        PathSource::CrateDir => {
            let cwd = std::env::current_dir()
                .map_err(|e| PathResolveError::Other(format!("failed to get cwd: {}", e)))?;
            find_nearest_project_root(&cwd).map_err(|e| {
                PathResolveError::Other(format!("find_nearest_project_root failed: {}", e))
            })
        }

        PathSource::Root => {
            let cwd = std::env::current_dir()
                .map_err(|e| PathResolveError::Other(format!("failed to get cwd: {}", e)))?;
            find_project_root(&cwd)
                .map_err(|e| PathResolveError::Other(format!("find_project_root failed: {}", e)))
        }

        PathSource::Cwd => {
            let cwd = std::env::current_dir()
                .map_err(|e| PathResolveError::Other(format!("failed to get cwd: {}", e)))?;
            Ok(cwd)
        }
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
fn find_project_root(start: impl AsRef<Path>) -> Result<PathBuf> {
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
fn find_nearest_project_root(start: impl AsRef<Path>) -> Result<PathBuf> {
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
