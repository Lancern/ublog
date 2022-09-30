mod cli;

use std::{error::Error, path::PathBuf};

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
            UblogArgs::FetchNotion(args) => crate::cli::notion::fetch_notion(&args).await,
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
}

#[derive(Debug, StructOpt)]
#[structopt(name = "fetch-notion", about = "Fetch content from Notion database")]
struct FetchNotionArgs {
    /// Notion access token.
    #[structopt(short, long)]
    token: String,

    /// Path to the ublog database.
    #[structopt(short, long)]
    database: PathBuf,

    /// Target Notion database ID.
    notion_database_id: String,
}
