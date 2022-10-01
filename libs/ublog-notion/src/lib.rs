pub mod api;
pub mod blog;
pub mod render;

use spdlog::{Logger, LoggerBuilder};

pub(crate) fn create_logger<T>(name: T) -> Logger
where
    T: Into<String>,
{
    let default_logger = spdlog::default_logger();
    LoggerBuilder::new()
        .name(name)
        .level_filter(default_logger.level_filter())
        .flush_level_filter(default_logger.flush_level_filter())
        .sinks(default_logger.sinks().iter().cloned())
        .build()
}
