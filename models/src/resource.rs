#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A static resource.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Resource {
    /// Name of the resource.
    pub name: String,

    /// The MIME type of the resource.
    pub ty: String,

    /// Raw data of the resource.
    pub data: Vec<u8>,
}
