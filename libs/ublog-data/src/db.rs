use crate::models::{Post, PostResource, Resource};
use crate::storage::{Pagination, Storage};

/// A database instance that loads data from an underlying storage.
#[derive(Debug)]
pub struct Database<S> {
    storage: S,
}

impl<S> Database<S> {
    /// Create a new database instance from the given storage.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }
}

impl<S> Database<S>
where
    S: Storage,
{
    /// Get the post object with the given slug.
    pub async fn get_post<T>(&self, slug: T) -> Result<Option<Post>, S::Error>
    where
        T: AsRef<str>,
    {
        let slug = slug.as_ref();
        self.storage.get_post(slug).await
    }

    /// Get a view of post objects within the specified page.
    pub async fn get_posts(&self, pagination: &Pagination) -> Result<Vec<Post>, S::Error> {
        self.storage.get_posts(pagination).await
    }

    /// Insert the given post into the database.
    pub async fn insert_post(
        &self,
        post: &Post,
        resources: &[PostResource],
    ) -> Result<(), S::Error> {
        self.storage.insert_post(post, resources).await
    }

    /// Delete the post object with the given slug.
    pub async fn delete_post<T>(&self, slug: T) -> Result<(), S::Error>
    where
        T: AsRef<str>,
    {
        let slug = slug.as_ref();
        self.storage.delete_post(slug).await
    }

    /// Get the static resource object with the given name.
    pub async fn get_resource<N>(&self, name: N) -> Result<Option<Resource>, S::Error>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        self.storage.get_resource(name).await
    }

    /// Get a list of resources within the specified page.
    pub async fn get_resources(&self) -> Result<Vec<Resource>, S::Error> {
        self.storage.get_resources().await
    }

    /// Insert the given resource object into the database.
    pub async fn insert_resource(&self, res: &Resource) -> Result<(), S::Error> {
        self.storage.insert_resource(res).await
    }

    /// Delete the resource object with the given name.
    pub async fn delete_resource<N>(&self, name: N) -> Result<(), S::Error>
    where
        N: AsRef<str>,
    {
        let name = name.as_ref();
        self.storage.delete_resource(name).await
    }
}
