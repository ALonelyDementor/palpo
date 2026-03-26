use std::{
    str::FromStr,
    path::{Path, PathBuf},
};
use mime::Mime;
use palpo_data::media::DbMetadata;
use salvo::fs::{NamedFile, NamedFileBuilder};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::fs;
use palpo_core::MatrixError;
use crate::{utils, config, AppResult};

use bytes::Bytes;

#[derive(Debug)]
pub struct FileSystem;

impl FileSystem {
    pub async fn create(&self, filename: &str, payload: &Bytes) -> AppResult<()> {
        let dest_path = self.get_dest_path(filename);

        if !dest_path.exists() {
            let parent_dir = utils::fs::get_parent_dir(&dest_path);
            fs::create_dir_all(&parent_dir)?;

            let mut file = File::create(dest_path).await?;
            file.write_all(payload).await?;

            Ok(())
        // TODO: thumbnail support
        } else {
            return Err(MatrixError::cannot_overwrite_media("Media ID already has content").into());
        }

    }

    pub async fn builder(&self, filename: &str) -> AppResult<NamedFileBuilder> {
        // Currently we only have 1 server. Let's use it to get the metadata
        let server_name = config::get().server_name.as_ref();

        if let Some(metadata) = crate::data::media::get_metadata(&server_name, filename)? {
            let content_type = metadata
                .content_type
                .as_deref()
                .and_then(|c| Mime::from_str(c).ok())
                .unwrap_or_else(|| {
                    metadata
                        .file_name
                        .as_ref()
                        .map(|name| mime_infer::from_path(name).first_or_octet_stream())
                        .unwrap_or(mime::APPLICATION_OCTET_STREAM)
                });

            let path = self.get_dest_path(filename);

            if Path::new(&path).exists() {
                let file =
                    self.to_named_file(path, &metadata).content_type(content_type);

                Ok(file)
            } else {
                Err(MatrixError::not_yet_uploaded("Media has not been uploaded yet").into())
            }
        } else {
            Err(MatrixError::not_yet_uploaded("Media has not been uploaded yet").into())
        }
    }

    // Internal functions

    fn to_named_file(&self, path: PathBuf, metadata: &DbMetadata) -> NamedFileBuilder {
        if let Some(file_name) = &metadata.file_name {
            NamedFile::builder(path).attached_name(file_name)
        } else {
            NamedFile::builder(path)
        }
    }

    fn get_dest_path(&self, filename: &str) -> PathBuf {
        let server_name = config::server_name();

        let mut r = PathBuf::new();
        r.push(config::space_path());
        r.push("media");
        r.push(server_name.as_str());
        r.push(filename);
        r
    }
}
