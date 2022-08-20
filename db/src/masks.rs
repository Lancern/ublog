use bitflags::bitflags;

bitflags! {
    /// A bit mask that selects the fields of [`Post`] that will be updated to the database.
    pub struct PostUpdateMask : u64 {
        const TITLE    = 0x01;
        const SLUG     = 0x02;
        const AUTHOR   = 0x04;
        const CATEGORY = 0x08;
        const CONTENT  = 0x10;
        const TAGS     = 0x20;
    }
}
