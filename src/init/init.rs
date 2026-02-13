use anyhow::{Context, Result};
use directories::ProjectDirs;
use env_logger::Builder;
use log::LevelFilter;
use std::fs::{OpenOptions, create_dir_all};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Mutex;

/// Initialize a simple logger that writes to a single file in the XDG project data directory
/// for `ast-grep-test`. The file will be truncated when it exceeds 5 MB (no rolling files).
///
/// - `log_level`: desired global log level (e.g. LevelFilter::Debug)
///
/// Log file location (in order of preference):
/// 1. `$XDG_DATA_HOME/ast-grep-test/logs/app.log` (via directories::ProjectDirs)
/// 2. `./logs/app.log` (fallback)
pub fn initialize_logger(log_level: LevelFilter) -> Result<()> {
    // 5 MiB limit
    const SIZE_LIMIT: u64 = 5 * 1024 * 1024;

    // Pick an XDG-ish project path for `ast-grep-test`.
    // ProjectDirs::from(qualifier, organization, application)
    let log_dir: PathBuf = ProjectDirs::from("com", "example", "ast-grep-test")
        .map(|pd| pd.data_dir().join("logs"))
        .unwrap_or_else(|| PathBuf::from("logs"));

    create_dir_all(&log_dir).context("Failed to create log directory")?;

    let log_file_path = log_dir.join("app.log");

    // Open (or create) the single log file in append mode.
    // We keep the File in a Mutex so the format closure can write to it.
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true) // needed if we query metadata/seek
        .open(&log_file_path)
        .with_context(|| format!("Failed to open log file {:?}", log_file_path))?;

    let file = Mutex::new(file);

    // clone the path for use inside the closure so we still have the original afterwards
    let log_file_path_for_closure = log_file_path.clone();

    let mut builder = Builder::new();
    builder.filter(None, log_level);

    // Format both to console (styled) and to the single file (plain text).
    builder.format(move |buf, record| {
        use console::style;

        let ts = buf.timestamp();
        let level = record.level();
        let msg = record.args();

        // ANSI style for console
        let color = match level {
            log::Level::Error => console::Color::Red,
            log::Level::Warn => console::Color::Yellow,
            log::Level::Info => console::Color::Green,
            log::Level::Debug => console::Color::Blue,
            log::Level::Trace => console::Color::Cyan,
        };
        let styled_level = style(level).fg(color);

        // Write to console (with colors)
        writeln!(buf, "[{:<5}] {} - {}", styled_level, ts, msg)?;

        // Prepare plain text entry for file
        let log_entry = format!("{} - {} - {}\n", ts, level, msg);

        // Write to file, truncating the file if it exceeds SIZE_LIMIT.
        // Use a separate scope for the lock to avoid holding it longer than necessary.
        if let Ok(mut f) = file.lock() {
            // Check size and truncate if necessary.
            match f.metadata() {
                Ok(meta) if meta.len() >= SIZE_LIMIT => {
                    // Truncate and reset cursor to start, then write a marker.
                    if let Err(e) = f.set_len(0) {
                        // Avoid recursive logging by using eprintln instead of log macros.
                        eprintln!(
                            "Failed to truncate log file {:?}: {:?}",
                            log_file_path_for_closure, e
                        );
                    } else if let Err(e) = f.seek(SeekFrom::Start(0)) {
                        eprintln!(
                            "Failed to seek log file {:?}: {:?}",
                            log_file_path_for_closure, e
                        );
                    } else {
                        let _ = f.write_all(b"--- Log truncated due to size limit ---\n");
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Failed to stat log file {:?}: {:?}",
                        log_file_path_for_closure, e
                    );
                }
                _ => {}
            }

            if let Err(e) = f.write_all(log_entry.as_bytes()) {
                // avoid recursion: use eprintln so we don't re-enter the logger
                eprintln!(
                    "Failed to write to log file {:?}: {:?}",
                    log_file_path_for_closure, e
                );
            }
        } else {
            // If lock is poisoned or cannot be acquired, write to stderr.
            eprintln!(
                "Failed to acquire lock for log file {:?}",
                log_file_path_for_closure
            );
        }

        Ok(())
    });

    builder
        .try_init()
        .context("Failed to initialize global logger")?;

    Ok(())
}
