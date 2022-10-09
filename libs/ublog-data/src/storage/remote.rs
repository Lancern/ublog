use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::{Commit, Delta, Post, Resource};
use crate::storage::{PaginatedList, Pagination, Storage};

/// A server that exposes an inner storage object through an underlying channel to a remote storage client.
#[derive(Debug)]
pub struct RemoteStorageServer<'s, S, T>
where
    S: ?Sized,
{
    inner: &'s S,
    channel: RemoteStorageChannel<T>,
}

impl<'s, S, T> RemoteStorageServer<'s, S, T>
where
    S: ?Sized,
{
    /// Create a new `RemoteStorageServer` object.
    ///
    /// The created `RemoteStorageServer` serves storage data from the given inner storage. The communication to remote
    /// storage client is performed on the given communication channel.
    pub fn new(inner: &'s S, channel: T) -> Self {
        Self {
            inner,
            channel: RemoteStorageChannel::new(channel),
        }
    }
}

macro_rules! process_request {
    ( $self:expr, $handler:expr ) => {{
        let res = $handler.await;
        $self.finish_request(res).await?;
    }};
}

impl<'s, S, T> RemoteStorageServer<'s, S, T>
where
    S: ?Sized + Storage,
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// Start the server and serve the underlying storage data.
    pub async fn serve(&mut self) -> std::io::Result<()> {
        loop {
            let request: Request<'static> = self.channel.receive().await?;
            self.handle_request(request).await?;
        }
    }

    async fn handle_request(&mut self, request: Request<'_>) -> std::io::Result<()> {
        match request {
            Request::InsertPost {
                post,
                post_resources,
            } => {
                process_request!(self, self.inner.insert_post(&*post, &*post_resources));
            }
            Request::UpdatePost {
                post,
                post_resources,
            } => {
                process_request!(self, self.inner.update_post(&*post, &*post_resources));
            }
            Request::DeletePost { post_slug } => {
                process_request!(self, self.inner.delete_post(&*post_slug));
            }
            Request::GetPost { post_slug } => {
                process_request!(self, self.inner.get_post(&*post_slug));
            }
            Request::GetPostWithResources { post_slug } => {
                process_request!(self, self.inner.get_post_with_resources(&*post_slug));
            }
            Request::GetPosts {
                special,
                pagination,
            } => {
                process_request!(self, self.inner.get_posts(special, &*pagination));
            }
            Request::InsertResource { resource } => {
                process_request!(self, self.inner.insert_resource(&*resource));
            }
            Request::DeleteResource { resource_id } => {
                process_request!(self, self.inner.delete_resource(&resource_id));
            }
            Request::GetResource { resource_id } => {
                process_request!(self, self.inner.get_resource(&resource_id));
            }
            Request::GetResources => {
                process_request!(self, self.inner.get_resources());
            }
            Request::GetCommitsSince { since_timestamp } => {
                process_request!(self, self.inner.get_commits_since(since_timestamp));
            }
            Request::GetLatestCommit => {
                process_request!(self, self.inner.get_latest_commit());
            }
            Request::ApplyDelta { delta } => {
                process_request!(self, self.inner.apply_delta(&*delta));
            }
        }

        Ok(())
    }

    async fn finish_request<R>(&mut self, inner_res: Result<R, S::Error>) -> std::io::Result<()>
    where
        R: Serialize,
    {
        let response = inner_res.map_err(|err| format!("{}", err));
        self.channel.send(&response).await?;
        Ok(())
    }
}

/// A [`Storage`] implementation that connects to a remote storage object through a [`RemoteStorageServer`].
#[derive(Debug)]
pub struct RemoteStorageClient<T> {
    channel: Mutex<RemoteStorageChannel<T>>,
}

impl<T> RemoteStorageClient<T> {
    /// Create a new `RemoteStorageClient` object.
    ///
    /// The communication to the [`RemoteStorageServer`] object is performed through the given communication channel.
    pub fn new(channel: T) -> Self {
        Self {
            channel: Mutex::new(RemoteStorageChannel::new(channel)),
        }
    }
}

impl<T> RemoteStorageClient<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    async fn execute_request<'a, R>(&self, request: &Request<'a>) -> Result<R, RemoteStorageError>
    where
        R: DeserializeOwned,
    {
        let mut channel = self.channel.lock().await;

        channel.send(request).await?;

        let response: Result<R, String> = channel.receive().await?;
        response.map_err(RemoteStorageError::Remote)
    }
}

#[async_trait]
impl<T> Storage for RemoteStorageClient<T>
where
    T: AsyncRead + AsyncWrite + Send + Sync + Unpin,
{
    type Error = RemoteStorageError;

    async fn insert_post(
        &self,
        post: &Post,
        post_resources: &[Resource],
    ) -> Result<(), Self::Error> {
        self.execute_request(&Request::InsertPost {
            post: Cow::Borrowed(post),
            post_resources: Cow::Borrowed(post_resources),
        })
        .await
    }

    async fn update_post(
        &self,
        post: &Post,
        post_resources: &[Resource],
    ) -> Result<(), Self::Error> {
        self.execute_request(&Request::UpdatePost {
            post: Cow::Borrowed(post),
            post_resources: Cow::Borrowed(post_resources),
        })
        .await
    }

    async fn delete_post(&self, post_slug: &str) -> Result<(), Self::Error> {
        self.execute_request(&Request::DeletePost {
            post_slug: Cow::Borrowed(post_slug),
        })
        .await
    }

    async fn get_post(&self, post_slug: &str) -> Result<Option<Post>, Self::Error> {
        self.execute_request(&Request::GetPost {
            post_slug: Cow::Borrowed(post_slug),
        })
        .await
    }

    async fn get_post_with_resources(
        &self,
        post_slug: &str,
    ) -> Result<Option<(Post, Vec<Resource>)>, Self::Error> {
        self.execute_request(&Request::GetPostWithResources {
            post_slug: Cow::Borrowed(post_slug),
        })
        .await
    }

    async fn get_posts(
        &self,
        special: bool,
        pagination: &Pagination,
    ) -> Result<PaginatedList<Post>, Self::Error> {
        self.execute_request(&Request::GetPosts {
            special,
            pagination: Cow::Borrowed(pagination),
        })
        .await
    }

    async fn insert_resource(&self, resource: &Resource) -> Result<(), Self::Error> {
        self.execute_request(&Request::InsertResource {
            resource: Cow::Borrowed(resource),
        })
        .await
    }

    async fn delete_resource(&self, resource_id: &Uuid) -> Result<(), Self::Error> {
        self.execute_request(&Request::DeleteResource {
            resource_id: *resource_id,
        })
        .await
    }

    async fn get_resource(&self, resource_id: &Uuid) -> Result<Option<Resource>, Self::Error> {
        self.execute_request(&Request::GetResource {
            resource_id: *resource_id,
        })
        .await
    }

    async fn get_resources(&self) -> Result<Vec<Resource>, Self::Error> {
        self.execute_request(&Request::GetResources).await
    }

    async fn get_commits_since(&self, since_timestamp: i64) -> Result<Vec<Commit>, Self::Error> {
        self.execute_request(&Request::GetCommitsSince { since_timestamp })
            .await
    }

    async fn get_latest_commit(&self) -> Result<Option<Commit>, Self::Error> {
        self.execute_request(&Request::GetLatestCommit).await
    }

    async fn apply_delta(&self, delta: &Delta) -> Result<(), Self::Error> {
        self.execute_request(&Request::ApplyDelta {
            delta: Cow::Borrowed(delta),
        })
        .await
    }
}

/// Error type of the remote storage.
#[derive(Debug)]
pub enum RemoteStorageError {
    Io(std::io::Error),
    Remote(String),
}

impl Display for RemoteStorageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::Remote(msg) => write!(f, "remote error: {}", msg),
        }
    }
}

impl Error for RemoteStorageError {}

impl From<std::io::Error> for RemoteStorageError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum Request<'a> {
    InsertPost {
        post: Cow<'a, Post>,
        post_resources: Cow<'a, [Resource]>,
    },
    UpdatePost {
        post: Cow<'a, Post>,
        post_resources: Cow<'a, [Resource]>,
    },
    DeletePost {
        post_slug: Cow<'a, str>,
    },
    GetPost {
        post_slug: Cow<'a, str>,
    },
    GetPostWithResources {
        post_slug: Cow<'a, str>,
    },
    GetPosts {
        special: bool,
        pagination: Cow<'a, Pagination>,
    },
    InsertResource {
        resource: Cow<'a, Resource>,
    },
    DeleteResource {
        resource_id: Uuid,
    },
    GetResource {
        resource_id: Uuid,
    },
    GetResources,
    GetCommitsSince {
        since_timestamp: i64,
    },
    GetLatestCommit,
    ApplyDelta {
        delta: Cow<'a, Delta>,
    },
}

#[derive(Debug)]
struct RemoteStorageChannel<T> {
    inner: T,
}

impl<T> RemoteStorageChannel<T> {
    fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> RemoteStorageChannel<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    async fn send<U>(&mut self, value: &U) -> std::io::Result<()>
    where
        U: Serialize,
    {
        let mut value_bson_data = bson::to_vec(value).unwrap();

        let mut packet_data =
            Vec::with_capacity(value_bson_data.len() + std::mem::size_of::<u64>());
        packet_data.extend_from_slice(&value_bson_data.len().to_le_bytes());
        packet_data.append(&mut value_bson_data);

        self.inner.write_all(&packet_data).await?;

        Ok(())
    }

    async fn receive<U>(&mut self) -> std::io::Result<U>
    where
        U: DeserializeOwned,
    {
        let mut packet_size_data = [0u8; std::mem::size_of::<u64>()];
        self.inner.read_exact(&mut packet_size_data).await?;

        let packet_size = u64::from_le_bytes(packet_size_data) as usize;
        let mut value_bson_data = vec![0u8; packet_size];
        self.inner.read_exact(&mut value_bson_data).await?;

        let value = bson::from_slice(&value_bson_data).unwrap();
        Ok(value)
    }
}
