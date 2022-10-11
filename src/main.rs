mod notion;
mod server;
mod utils;

use std::error::Error;
use std::path::PathBuf;

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

    let runtime = fallible_step!("initialize async runtime", Runtime::new());
    runtime.block_on(async {
        match args {
            UblogArgs::FetchNotion(args) => crate::notion::fetch_notion(&args).await,
            UblogArgs::Serve(args) => crate::server::serve(&args).await,
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

    /// Enable debug output.
    #[structopt(long)]
    debug: bool,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "serve", about = "Start ublog backend service")]
struct ServerArgs {
    /// The address the server binds to.
    #[structopt(short, long, default_value = "0.0.0.0")]
    addr: String,

    /// The port the server listens on.
    #[structopt(short, long, default_value = "8000")]
    port: u16,

    /// Path to the site information file.
    #[structopt(short, long, default_value = "site.json")]
    site: PathBuf,

    /// Path to the ublog database.
    #[structopt(short, long, default_value = "ublog.db")]
    database: PathBuf,

    /// Enable debug output.
    #[structopt(long)]
    debug: bool,

    /// Path to the log file directory.
    #[structopt(short, long, default_value = "ublog.logs.d")]
    logs_dir: PathBuf,
}
