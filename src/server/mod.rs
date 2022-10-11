pub(crate) mod config;
mod feed;
mod router;

use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::sync::Arc;

use axum::Server;
use hyper::server::conn::AddrIncoming;
use rss::Channel as RssChannel;
use ublog_data::db::Database;
use ublog_data::storage::sqlite::SqliteStorage;

use crate::server::config::SiteConfig;
use crate::utils::cache::Cache;
use crate::{fallible_step, ServerArgs};

pub(crate) async fn serve(args: &ServerArgs) -> Result<(), Box<dyn Error>> {
    fallible_step!(
        "initialize logger",
        crate::utils::logging::init_server_logger(args.debug, &args.logs_dir)
    );

    let site = fallible_step!("load site config", load_site_config(&args.site).await);

    let storage = fallible_step!(
        "initialize database storage",
        SqliteStorage::new_file(&args.database)
    );

    let ctx = ServerContext {
        site,
        db: Database::new(storage),
        rss_cache: Cache::new(RSS_CACHE_EXPIRE),
    };
    let router = crate::server::router::create_router(Arc::new(ctx));

    let addr: IpAddr = fallible_step!("parse server address", args.addr.parse());
    let server_addr = SocketAddr::new(addr, args.port);

    spdlog::info!("Starting HTTP server at {}", server_addr);
    let acceptor = fallible_step!("initialize TCP acceptor", AddrIncoming::bind(&server_addr));
    fallible_step!(
        "run server",
        Server::builder(acceptor)
            .serve(router.into_make_service())
            .await
    );

    Ok(())
}

async fn load_site_config<P>(path: P) -> Result<SiteConfig, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let config_json = fallible_step!("read site config", tokio::fs::read_to_string(path).await);
    let config = fallible_step!("parse site config", serde_json::from_str(&config_json));
    Ok(config)
}

#[derive(Debug)]
struct ServerContext {
    site: SiteConfig,
    db: Database<SqliteStorage>,
    rss_cache: Cache<RssChannel>,
}

// RSS cache expire time is 10 minutes.
const RSS_CACHE_EXPIRE: u64 = 600;
