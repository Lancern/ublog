use serde::{Deserialize, Serialize};

/// Provide information about the served site.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SiteConfig {
    pub(crate) title: String,
    pub(crate) owner: String,
    pub(crate) owner_email: String,
    pub(crate) url: String,
    pub(crate) copyright: String,
    pub(crate) post_url_template: String,
}
