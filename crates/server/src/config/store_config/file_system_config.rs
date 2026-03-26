use super::ConfigValidator;
use crate::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileSystemConfig {
    #[serde(default = "default_space_path")]
    pub location: String,

    /// Max request size for file uploads in bytes. Defaults to 20MB.
    ///
    /// default: 20971520
    #[serde(default = "default_max_upload_size")]
    pub max_upload_size: u32,
}

impl ConfigValidator for FileSystemConfig {
    fn check(&self) -> AppResult<()> {
        if self.max_upload_size < 10_000_000 {
            tracing::warn!(
                "max request size is less than 100MB. Please increase it as this is too low for operable federation"
            );
        }
        Ok(())
    }
}

// Internal Functions
fn default_space_path() -> String {
    "./space".into()
}

fn default_max_upload_size() -> u32 {
    100 * 1024 * 1024 // Default to 100 MB
}