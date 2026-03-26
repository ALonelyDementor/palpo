mod bucket_config;
mod file_system_config;

pub use bucket_config::*;
pub use file_system_config::*;
use serde::{Deserialize, Serialize};

use super::ConfigValidator;
use crate::AppResult;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoreConfig {
    FileSystem(FileSystemConfig),
    Bucket(BucketConfig),
}

pub enum StoreEngine {
    FileSystem,
    Bucket,
}

impl StoreConfig {
    pub fn engine(&self) -> StoreEngine {
        match self {
            StoreConfig::Bucket(_) => StoreEngine::Bucket,
            StoreConfig::FileSystem(_) => StoreEngine::FileSystem,
        }
    }

    pub fn max_upload_size(&self) -> u64 {
        match self {
            StoreConfig::Bucket(_) => 100000000 as u64,
            StoreConfig::FileSystem(f) => f.max_upload_size as u64
        }
    }

    pub fn save_location(&self) -> String {
        match self {
            StoreConfig::Bucket(b) => b.location.clone(),
            StoreConfig::FileSystem(f) => f.location.clone()
        }
    }

    pub fn bucket_config(&self) -> Option<BucketConfig> {
        match self {
            StoreConfig::Bucket(b) => Some(b.clone()),
            StoreConfig::FileSystem(_) => None
        }
    }
}

impl ConfigValidator for StoreConfig {
    fn check(&self) -> AppResult<()> {
        match self {
            StoreConfig::FileSystem(config) => config.check(),
            StoreConfig::Bucket(config) => config.check(),
        }
    }
}



