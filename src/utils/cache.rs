use std::future::Future;
use std::ops::Deref;
use std::time::SystemTime;

use tokio::sync::{Mutex, MutexGuard};

#[derive(Debug)]
pub(crate) struct Cache<T> {
    expire_secs: u64,
    value: Mutex<Option<(SystemTime, T)>>,
}

impl<T> Cache<T> {
    pub(crate) fn new(expire_secs: u64) -> Self {
        Self {
            expire_secs,
            value: Mutex::new(None),
        }
    }

    pub(crate) async fn get<F, R, E>(&self, value_factory: F) -> Result<CachedValue<T>, E>
    where
        F: FnOnce() -> R,
        R: Future<Output = Result<T, E>>,
    {
        let mut lock = self.value.lock().await;
        if let Some((t, _)) = &*lock {
            if t.elapsed().unwrap().as_secs() < self.expire_secs {
                return Ok(CachedValue { guard: lock });
            }
        }

        let value = value_factory().await?;
        *lock = Some((SystemTime::now(), value));

        Ok(CachedValue { guard: lock })
    }
}

#[derive(Debug)]
pub(crate) struct CachedValue<'a, T> {
    guard: MutexGuard<'a, Option<(SystemTime, T)>>,
}

impl<'a, T> CachedValue<'a, T> {
    fn cache_pair(&self) -> &(SystemTime, T) {
        (*self.guard).as_ref().unwrap()
    }
}

impl<'a, T> Deref for CachedValue<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.cache_pair().1
    }
}
