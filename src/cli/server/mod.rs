mod router;
mod tls;

use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::{Router, Server};
use hyper::server::accept::Accept;
use hyper::server::conn::AddrIncoming;
use tokio::io::{AsyncRead, AsyncWrite};
use ublog_data::db::Database;
use ublog_data::storage::sqlite::SqliteStorage;

use crate::{fallible_step, ServerArgs};

use self::tls::TlsAddrIncoming;

pub(crate) async fn serve(args: &ServerArgs) -> Result<(), Box<dyn Error>> {
    let storage = fallible_step!(
        "initialize database storage",
        SqliteStorage::new_file(&args.database)
    );
    let db = Database::new(storage);

    let ctx = ServerContext { db };
    let router = crate::cli::server::router::create_router(Arc::new(ctx));

    let addr: IpAddr = fallible_step!("parse server address", args.addr.parse());
    let server_addr = SocketAddr::new(addr, args.port);

    if let Some(cert) = &args.cert {
        let acceptor = fallible_step!(
            "initialize TLS acceptor",
            TlsAddrIncoming::new(cert, &server_addr)
        );
        spdlog::info!("Starting HTTPS server at {}", server_addr);
        serve_with(acceptor, router).await?;
    } else {
        let acceptor = fallible_step!("initialize TCP acceptor", AddrIncoming::bind(&server_addr));
        spdlog::info!("Starting HTTP server at {}", server_addr);
        serve_with(acceptor, router).await?;
    }

    Ok(())
}

async fn serve_with<A>(acceptor: A, router: Router) -> Result<(), Box<dyn Error>>
where
    A: Accept,
    A::Conn: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    A::Error: Into<Box<dyn Error + Send + Sync>>,
{
    fallible_step!(
        "run server",
        Server::builder(acceptor)
            .serve(router.into_make_service())
            .await
    );

    Ok(())
}

#[derive(Debug)]
struct ServerContext {
    db: Database<SqliteStorage>,
}
