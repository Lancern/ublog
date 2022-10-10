use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use spdlog::sink::{RotatingFileSink, RotationPolicy, StdStream, StdStreamSink};
use spdlog::terminal_style::StyleMode;
use spdlog::{Level, LevelFilter, LoggerBuilder};

use crate::fallible_step;

pub(crate) fn init_basic_logger(debug: bool) -> Result<(), Box<dyn Error>> {
    init_logger(debug, |_| Ok(()))
}

pub(crate) fn init_server_logger<P>(debug: bool, logs_dir: P) -> Result<(), Box<dyn Error>>
where
    P: Into<PathBuf>,
{
    const ROTATION_POLICY: RotationPolicy = RotationPolicy::Daily { hour: 0, minute: 0 };
    const MAX_FILES: usize = 30;

    init_logger(debug, |builder| {
        let log_file_sink = fallible_step!(
            "initialize rotating log file sink",
            RotatingFileSink::new(logs_dir, ROTATION_POLICY, MAX_FILES, false)
        );

        builder.sink(Arc::new(log_file_sink));

        Ok(())
    })
}

fn init_logger<F>(debug: bool, configure: F) -> Result<(), Box<dyn Error>>
where
    F: FnOnce(&mut LoggerBuilder) -> Result<(), Box<dyn Error>>,
{
    let mut logger_builder = LoggerBuilder::new();

    logger_builder.sink(Arc::new(StdStreamSink::new(
        StdStream::Stdout,
        StyleMode::Auto,
    )));

    if debug {
        logger_builder.level_filter(LevelFilter::All);
    } else {
        logger_builder.level_filter(LevelFilter::MoreSevereEqual(Level::Info));
    }

    configure(&mut logger_builder)?;

    let logger = logger_builder.build();
    spdlog::set_default_logger(Arc::new(logger));

    fallible_step!("initialize standard logger", spdlog::init_log_crate_proxy());

    Ok(())
}
