use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{ready, Context as TaskContext, Poll};

use futures::Future;
use hyper::server::accept::Accept;
use hyper::server::conn::{AddrIncoming, AddrStream};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

#[derive(Debug)]
pub(super) enum TlsError {
    Io(std::io::Error),
    Hyper(hyper::Error),
    Tls(tokio_rustls::rustls::Error),
}

impl Display for TlsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::Hyper(err) => write!(f, "hyper error: {}", err),
            Self::Tls(err) => write!(f, "TLS error: {}", err),
        }
    }
}

impl Error for TlsError {}

impl From<std::io::Error> for TlsError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<hyper::Error> for TlsError {
    fn from(err: hyper::Error) -> Self {
        Self::Hyper(err)
    }
}

impl From<tokio_rustls::rustls::Error> for TlsError {
    fn from(err: tokio_rustls::rustls::Error) -> Self {
        Self::Tls(err)
    }
}

#[derive(Debug)]
pub(super) struct TlsAddrStream {
    state: TlsAddrStreamState,
}

impl TlsAddrStream {
    fn new(accept_fut: tokio_rustls::Accept<AddrStream>) -> Self {
        Self {
            state: TlsAddrStreamState::Handshaking(accept_fut),
        }
    }

    fn poll_handshake(&mut self, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
        match &mut self.state {
            TlsAddrStreamState::Handshaking(accept_fut) => {
                match ready!(Pin::new(accept_fut).poll(cx)) {
                    Ok(stream) => {
                        self.state = TlsAddrStreamState::Streaming(stream);
                        Poll::Ready(Ok(()))
                    }
                    Err(err) => Poll::Ready(Err(err)),
                }
            }
            TlsAddrStreamState::Streaming(_) => Poll::Ready(Ok(())),
        }
    }

    fn poll_stream<F, R>(
        &mut self,
        cx: &mut TaskContext<'_>,
        poll_stream: F,
    ) -> Poll<std::io::Result<R>>
    where
        F: FnOnce(
            Pin<&mut TlsStream<AddrStream>>,
            &mut TaskContext<'_>,
        ) -> Poll<std::io::Result<R>>,
    {
        match ready!(self.poll_handshake(cx)) {
            Ok(()) => {}
            Err(err) => {
                return Poll::Ready(Err(err));
            }
        }

        let stream = Pin::new(self.state.stream_mut());
        poll_stream(stream, cx)
    }
}

impl AsyncRead for TlsAddrStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.get_mut()
            .poll_stream(cx, move |stream, cx| stream.poll_read(cx, buf))
    }
}

impl AsyncWrite for TlsAddrStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        self.get_mut()
            .poll_stream(cx, move |stream, cx| stream.poll_write(cx, buf))
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.get_mut()
            .poll_stream(cx, |stream, cx| stream.poll_flush(cx))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.get_mut()
            .poll_stream(cx, |stream, cx| stream.poll_shutdown(cx))
    }
}

pub(super) struct TlsAddrIncoming {
    tls_acceptor: TlsAcceptor,
    addr_incoming: AddrIncoming,
}

impl TlsAddrIncoming {
    pub(super) fn new<P>(cert_path: P, server_addr: &SocketAddr) -> Result<Self, TlsError>
    where
        P: AsRef<Path>,
    {
        let (cert, priv_key) = load_certificate(cert_path)?;
        let tls_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert, priv_key)?;

        let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));
        let addr_incoming = AddrIncoming::bind(server_addr)?;

        Ok(Self {
            tls_acceptor,
            addr_incoming,
        })
    }
}

impl Accept for TlsAddrIncoming {
    type Conn = TlsAddrStream;
    type Error = std::io::Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match ready!(Pin::new(&mut self.addr_incoming).poll_accept(cx)) {
            Some(Ok(conn)) => {
                let accept_fut = Pin::new(&mut self.tls_acceptor).accept(conn);
                Poll::Ready(Some(Ok(TlsAddrStream::new(accept_fut))))
            }
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => Poll::Ready(None),
        }
    }
}

impl Debug for TlsAddrIncoming {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsAddrIncoming").finish()
    }
}

enum TlsAddrStreamState {
    Handshaking(tokio_rustls::Accept<AddrStream>),
    Streaming(TlsStream<AddrStream>),
}

impl TlsAddrStreamState {
    fn stream_mut(&mut self) -> &mut TlsStream<AddrStream> {
        match self {
            Self::Streaming(stream) => stream,
            _ => panic!("trying to get TlsStream before TLS handshake completes"),
        }
    }
}

impl Debug for TlsAddrStreamState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Handshaking(_) => f.debug_tuple("Handshaking").finish(),
            Self::Streaming(stream) => f.debug_tuple("Streaming").field(stream).finish(),
        }
    }
}

fn load_certificate<P>(path: P) -> std::io::Result<(Vec<Certificate>, PrivateKey)>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut certificates = Vec::new();
    let mut priv_key = None;

    for item in std::iter::from_fn(|| rustls_pemfile::read_one(&mut reader).transpose()) {
        match item? {
            rustls_pemfile::Item::X509Certificate(data) => {
                certificates.push(Certificate(data));
            }
            rustls_pemfile::Item::RSAKey(data) => {
                if priv_key.is_none() {
                    priv_key = Some(PrivateKey(data));
                }
            }
            _ => {}
        }
    }

    if priv_key.is_none() {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
    }

    Ok((certificates, priv_key.unwrap()))
}
