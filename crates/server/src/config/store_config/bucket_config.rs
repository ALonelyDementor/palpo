use super::ConfigValidator;
use crate::{AppResult, AppError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BucketConfig {
    pub bucket_name: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    #[serde(default = "bucket_default_location")]
    pub location: String,
}

impl ConfigValidator for BucketConfig {
    fn check(&self) -> AppResult<()> {
        if self.bucket_name.is_empty() {
            return Err(AppError::internal("Bucket name cannot be empty."));
        }

        if self.region.is_empty() {
            return Err(AppError::internal("Region cannot be empty."));
        }

        if self.access_key_id.is_empty() {
            return Err(AppError::internal("Access key ID cannot be empty."));
        }

        if self.secret_access_key.is_empty() {
            return Err(AppError::internal("Secret access key cannot be empty."));
        }

        // Add validation logic for BucketConfig here
        Ok(())
    }
}

fn bucket_default_location() -> String {
    "media".to_string()
}