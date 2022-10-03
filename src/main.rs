mod cli;

use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use spdlog::sink::{FileSink, StdStream, StdStreamSink};
use spdlog::terminal_style::StyleMode;
use spdlog::{Level, LevelFilter, LoggerBuilder};
use structopt::StructOpt;
use tokio::runtime::Runtime;

#[macro_export]
macro_rules! fallible_step {
    ( $desc:expr, $e:expr ) => {
        match $e {
            Ok(res) => res,
            Err(err) => {
                return Err(Box::<dyn std::error::Error>::from(format!(
                    "{} failed: {}",
                    $desc, err
                )));
            }
        }
    };
}

fn main() {
    if let Err(err) = main_impl() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

fn main_impl() -> Result<(), Box<dyn Error>> {
    let args = UblogArgs::from_args();

    fallible_step!("initialize logger", init_logger(args.log_args()));

    let runtime = fallible_step!("initialize async runtime", Runtime::new());
    runtime.block_on(async {
        match args {
            UblogArgs::FetchNotion(args) => crate::cli::notion::fetch_notion(&args).await,
            UblogArgs::Serve(args) => crate::cli::server::serve(&args).await,
        }
    })
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ublog",
    about = "ublog is Lancern's personal blog system",
    author = "Sirui Mu <msrlancern@gmail.com>",
    version = "0.1.0"
)]
enum UblogArgs {
    FetchNotion(FetchNotionArgs),
    Serve(ServerArgs),
}

impl UblogArgs {
    fn log_args(&self) -> &LogArgs {
        match self {
            Self::FetchNotion(args) => &args.log,
            Self::Serve(args) => &args.log,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "fetch-notion", about = "Fetch content from Notion database")]
struct FetchNotionArgs {
    /// Notion access token.
    #[structopt(short, long)]
    token: String,

    /// Path to the ublog database.
    #[structopt(short, long, default_value = "ublog.db")]
    database: PathBuf,

    /// Target Notion database ID.
    notion_database_id: String,

    #[structopt(flatten)]
    log: LogArgs,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "serve", about = "Start ublog backend service")]
struct ServerArgs {
    /// The address the server binds to.
    #[structopt(short, long, default_value = "127.0.0.1")]
    addr: String,

    /// The port the server listens on.
    #[structopt(short, long, default_value = "8000")]
    port: u16,

    /// Path to the certificate file.
    ///
    /// If this argument is missing, an HTTP server will be started.
    #[structopt(short, long)]
    cert: Option<PathBuf>,

    /// Path to the ublog database.
    #[structopt(short, long, default_value = "ublog.db")]
    database: PathBuf,

    #[structopt(flatten)]
    log: LogArgs,
}

#[derive(Debug, StructOpt)]
struct LogArgs {
    /// Enable debug output.
    #[structopt(long)]
    debug: bool,

    /// Path to the output log file.
    #[structopt(long)]
    log_file: Option<PathBuf>,
}

fn init_logger(args: &LogArgs) -> Result<(), Box<dyn Error>> {
    let mut logger_builder = LoggerBuilder::new();

    logger_builder.sink(Arc::new(StdStreamSink::new(
        StdStream::Stdout,
        StyleMode::Auto,
    )));

    if let Some(log_file_path) = &args.log_file {
        let file_sink = fallible_step!(
            "initialize file log sink",
            FileSink::new(log_file_path, true)
        );
        logger_builder.sink(Arc::new(file_sink));
    }

    if args.debug {
        logger_builder.level_filter(LevelFilter::All);
    } else {
        logger_builder.level_filter(LevelFilter::MoreSevereEqual(Level::Info));
    }

    let logger = logger_builder.build();
    spdlog::set_default_logger(Arc::new(logger));

    fallible_step!("initialize standard logger", spdlog::init_log_crate_proxy());

    Ok(())
}
