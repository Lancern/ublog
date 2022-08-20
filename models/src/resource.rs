/// A static resource.
#[derive(Clone, Debug)]
pub struct Resource {
    /// Name of the resource.
    pub name: String,

    /// The MIME type of the resource.
    pub ty: String,

    /// Raw data of the resource.
    pub data: Vec<u8>,
}
