use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::models::{Commit, CommitPayload, Delta};
use crate::storage::Storage;

/// Synchronize data in `storage_from` to `storage_to`.
pub async fn synchronize_storage<SF, ST>(
    storage_from: &SF,
    storage_to: &ST,
) -> Result<(), SynchronizeStorageError<SF::Error, ST::Error>>
where
    SF: ?Sized + Storage,
    ST: ?Sized + Storage,
{
    let delta = get_delta(storage_from, storage_to).await?;
    storage_to
        .apply_delta(&delta)
        .await
        .map_err(SynchronizeStorageError::ToStorage)?;

    Ok(())
}

/// Compute the delta required to synchronize data from `storage_from` to `storage_to`.
pub async fn get_delta<SF, ST>(
    storage_from: &SF,
    storage_to: &ST,
) -> Result<Delta, SynchronizeStorageError<SF::Error, ST::Error>>
where
    SF: ?Sized + Storage,
    ST: ?Sized + Storage,
{
    let to_latest_commit = storage_to
        .get_latest_commit()
        .await
        .map_err(SynchronizeStorageError::ToStorage)?;
    let to_latest_commit_timestamp = to_latest_commit
        .as_ref()
        .map(|commit| commit.timestamp)
        .unwrap_or(0);

    let mut from_commits = storage_from
        .get_commits_since(to_latest_commit_timestamp)
        .await
        .map_err(SynchronizeStorageError::FromStorage)?;

    if from_commits.is_empty() {
        if to_latest_commit.is_none() {
            // Both of storage_from and storage_to does not contain any commits.
            return Ok(Delta::new());
        } else {
            return Err(SynchronizeStorageError::DiverseHistory);
        }
    } else if let Some(to_latest_commit) = to_latest_commit.as_ref() {
        if to_latest_commit.id != from_commits[0].id {
            return Err(SynchronizeStorageError::DiverseHistory);
        } else {
            // The first commit within `from_commits` is actually present in the destication storage.
            from_commits.remove(0);
        }
    }

    collect_delta(storage_from, from_commits)
        .await
        .map_err(SynchronizeStorageError::FromStorage)
}

/// Errors during storage synchronization.
#[derive(Debug)]
pub enum SynchronizeStorageError<EF, ET> {
    FromStorage(EF),
    ToStorage(ET),
    DiverseHistory,
}

impl<EF, ET> Display for SynchronizeStorageError<EF, ET>
where
    EF: Display,
    ET: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FromStorage(err) => write!(f, "error from the source storage: {}", err),
            Self::ToStorage(err) => write!(f, "error from the destination storage: {}", err),
            Self::DiverseHistory => write!(f, "diverse history"),
        }
    }
}

impl<EF, ET> Error for SynchronizeStorageError<EF, ET>
where
    EF: Error,
    ET: Error,
{
}

async fn collect_delta<S>(storage: &S, commits: Vec<Commit>) -> Result<Delta, S::Error>
where
    S: ?Sized + Storage,
{
    let mut added_post_slugs = HashSet::new();
    let mut deleted_post_slugs = HashSet::new();
    let mut added_resource_names = HashSet::new();
    let mut deleted_resource_names = HashSet::new();

    for c in &commits {
        match &c.payload {
            CommitPayload::CreatePost(payload) => {
                added_post_slugs.insert(payload.slug.clone());
            }
            CommitPayload::DeletePost(payload) => {
                if !added_post_slugs.remove(&payload.slug) {
                    deleted_post_slugs.insert(payload.slug.clone());
                }
            }
            CommitPayload::CreateResource(payload) => {
                added_resource_names.insert(payload.name.clone());
            }
            CommitPayload::DeleteResource(payload) => {
                if !added_resource_names.remove(&payload.name) {
                    deleted_resource_names.insert(payload.name.clone());
                }
            }
        }
    }

    let mut delta = Delta::new();

    for slug in &added_post_slugs {
        if let Some(post_with_resources) = storage.get_post_with_resources(slug).await? {
            delta.added_posts.push(post_with_resources);
        }
    }

    for name in &added_resource_names {
        if let Some(resource) = storage.get_resource(name).await? {
            delta.added_resources.push(resource);
        }
    }

    delta.deleted_post_slugs = deleted_post_slugs.into_iter().collect();
    delta.deleted_resource_names = deleted_resource_names.into_iter().collect();

    delta.commits = commits;
    Ok(delta)
}
