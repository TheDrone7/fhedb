use fern;
use log::LevelFilter;
use std::path::PathBuf;

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
        logger = logger.chain(std::io::stdout());
    }

    logger.apply()?;

    Ok(())
}
