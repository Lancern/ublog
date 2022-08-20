#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "time")]
use time::OffsetDateTime;

/// A blog post.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Post {
    /// A globally unique ID that identifies the post.
    pub id: i64,

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

    /// Number of views of the post.
    pub views: u64,

    /// Content of the post.
    pub content: String,
}

impl Post {
    /// Get the post's creation time.
    #[cfg(feature = "time")]
    pub fn create_time(&self) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(self.create_timestamp).unwrap()
    }

    /// Get the post's last update time.
    #[cfg(feature = "time")]
    pub fn update_time(&self) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(self.update_timestamp).unwrap()
    }
}

/// A resource object that is attached to a post.
#[derive(Clone, Debug)]
pub struct PostResource {
    /// ID of the associated post.
    pub post_id: i64,

    /// Name of the resource.
    pub name: String,

    /// MIME type of the resource.
    pub ty: String,

    /// Data of the resource.
    pub data: Vec<u8>,
}
