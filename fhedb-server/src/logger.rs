//! # Logging Setup
//!
//! Logging initialization with configurable level and output destinations.

use fern;
use log::LevelFilter;
use std::{io, path::PathBuf};

/// Sets up the logging system with the specified configuration.
///
/// ## Arguments
///
/// * `level` - The minimum [`LevelFilter`] to display.
/// * `file` - Optional [`PathBuf`] to write logs to. If [`None`], logs are written to stdout.
///
/// ## Returns
///
/// Returns [`Ok`]\(()) on success, or [`Err`]\([`fern::InitError`]) if initialization fails.
pub fn setup_logger(level: LevelFilter, file: Option<PathBuf>) -> Result<(), fern::InitError> {
    let mut logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ));
        })
        .level(level);

    if let Some(file_path) = file {
        logger = logger.chain(fern::log_file(file_path)?);
    } else {
        logger = logger.chain(io::stdout());
    }

    logger.apply()?;

    Ok(())
}
