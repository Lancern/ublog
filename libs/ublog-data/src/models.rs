use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use ublog_doc::DocumentNode;
use uuid::Uuid;

/// A blog post.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    /// The post's title.
    pub title: String,

    /// The post's slug.
    pub slug: String,

    /// The post's author.
    pub author: String,

    /// Unix timestamp of the post's creation time, in UTC time zone.
    pub create_timestamp: i64,

    /// Unix timestamp of the post's last update time, in UTC time zone.
    pub update_timestamp: i64,

    /// The post's category.
    pub category: String,

    /// The post's tags.
    pub tags: Vec<String>,

    /// Is this post a special post?
    #[serde(rename = "isSpecial")]
    pub is_special: bool,

    /// Content of the post.
    pub content: DocumentNode,
}

impl Post {
    /// Get the post's creation time.
    pub fn create_time(&self) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(self.create_timestamp).unwrap()
    }

    /// Get the post's last update time.
    pub fn update_time(&self) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(self.update_timestamp).unwrap()
    }
}

/// A static resource.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Resource {
    /// UUID of the resource.
    pub id: Uuid,

    /// Name of the resource.
    pub name: String,

    /// The MIME type of the resource.
    pub ty: String,

    /// Raw data of the resource.
    pub data: Vec<u8>,
}

/// A commit object.
///
/// A commit object represents a unit of change to the blog content.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Commit {
    /// ID of the commit.
    ///
    /// A commit's ID is the SHA256 digest of the commit's metadata and data.
    pub id: Vec<u8>,

    /// The creation timestamp of the commit.
    pub timestamp: i64,

    /// The previous commit ID.
    pub prev_commit_id: Vec<u8>,

    /// The commit's payload.
    pub payload: CommitPayload,
}

impl Commit {
    /// Create a new commit object that contains the specified payload and points the specified commit as its parent
    /// commit.
    pub fn new<T>(prev_commit_id: T, payload: CommitPayload) -> Self
    where
        T: Into<Vec<u8>>,
    {
        let prev_commit_id = prev_commit_id.into();
        let timestamp = OffsetDateTime::now_utc().unix_timestamp();

        let mut commit = Self {
            id: Vec::new(),
            timestamp,
            prev_commit_id,
            payload,
        };
        let commit_digest_data = bson::to_vec(&commit).unwrap();
        let commit_digest = {
            let mut hasher = Sha256::new();
            hasher.update(&commit_digest_data);
            hasher.finalize()
        };

        commit.id = Vec::from(commit_digest.as_slice());

        commit
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CommitPayload {
    CreatePost(CreatePostCommitPayload),
    DeletePost(DeletePostCommitPayload),
    CreateResource(CreateResourceCommitPayload),
    DeleteResource(DeleteResourceCommitPayload),
}

impl CommitPayload {
    /// Create a new `CreatePost` commit payload.
    pub fn create_post<T>(slug: T) -> Self
    where
        T: Into<String>,
    {
        Self::CreatePost(CreatePostCommitPayload { slug: slug.into() })
    }

    /// Create a new `DeletePost` commit payload.
    pub fn delete_post<T>(slug: T) -> Self
    where
        T: Into<String>,
    {
        Self::DeletePost(DeletePostCommitPayload { slug: slug.into() })
    }

    /// Create a new `CreateResource` commit payload.
    pub fn create_resource(id: Uuid) -> Self {
        Self::CreateResource(CreateResourceCommitPayload { id })
    }

    /// Create a new `DeleteResource` commit payload.
    pub fn delete_resource(id: Uuid) -> Self {
        Self::DeleteResource(DeleteResourceCommitPayload { id })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreatePostCommitPayload {
    pub slug: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeletePostCommitPayload {
    pub slug: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateResourceCommitPayload {
    pub id: Uuid,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeleteResourceCommitPayload {
    pub id: Uuid,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Delta {
    pub added_posts: Vec<(Post, Vec<Resource>)>,
    pub deleted_post_slugs: Vec<String>,
    pub added_resources: Vec<Resource>,
    pub deleted_resource_ids: Vec<Uuid>,
    pub commits: Vec<Commit>,
}

impl Delta {
    /// Create a new, empty `Delta`.
    pub fn new() -> Self {
        Self::default()
    }
}
