use std::error::Error;
use std::fmt::{Display, Formatter};

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
    todo!()
}

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
