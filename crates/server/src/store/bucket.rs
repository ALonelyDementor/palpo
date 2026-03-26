use std::{path::PathBuf, str::FromStr};
use bytes::Bytes;
use palpo_data::media::DbMetadata;
use aws_credential_types::Credentials;
use palpo_core::MatrixError;
use mime::Mime;
use aws_sdk_s3::{Client as S3Client, Config as S3Config};
use aws_config::Region;
use tokio::{
    fs::File,
    io::AsyncWriteExt
};
use salvo::fs::{NamedFile, NamedFileBuilder};

use crate::{AppResult, AppError, config};

#[derive(Debug)]
pub struct Bucket{
    client: S3Client,
    bucket: String,
}

impl Bucket {
    pub fn init() -> AppResult<Self> {
        if let Some(bucket_config) = config::get().storage.bucket_config() {

            let credential_provider =
                Credentials::from_keys(&bucket_config.access_key_id, &bucket_config.secret_access_key, None);

            // This works for now as I'm just using exo, but this should be changed to a callback that builds this endpoint based on config
            let bucket_endpoint = format!("https://sos-{}.exo.io", &bucket_config.region);
            let region = Region::new(bucket_config.region);

            let s3_config = S3Config::builder()
                .force_path_style(true)
                .credentials_provider(credential_provider)
                .endpoint_url(bucket_endpoint)
                .set_region(Some(region))
                .clone()
                .build();

            let client = S3Client::from_conf(s3_config);

            Ok(Bucket {
                client,
                bucket: bucket_config.bucket_name.clone()
            })
        } else {
            Err(AppError::internal("Bucket not configured"))
        }
    }

    pub async fn put(&self, filename: &str, payload: &Bytes) -> AppResult<()> {
        let mut path = self.get_dest_path();

        path.push(filename);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(path.to_str().expect("Could not build path"))
            .body(payload.to_vec().into())
            .send()
            .await
            .map_or_else(|_| { Err(AppError::internal("Could not save the file")) },  |_| { Ok(()) })
    }

    pub async fn builder(&self, media_id: &str) -> AppResult<NamedFileBuilder> {
        // This get will fetch the ByteStream from cloud and
        // create a temp file to send the user.
        // Ideally, the file will be locked, since multiple users can request the same file
        // And to avoid removing the file while another process is trying to access it.
        // (We might want to add a reference counter to check the amount of processes requesting this file)

        let server_name = config::server_name();
        if let Some(metadata) = crate::data::media::get_metadata(&server_name, media_id)? {
            let file_location = self.temp_file(media_id, &metadata).await?;

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
                // File was just created. Let's pass the existence validation and create the named file:
            let named_file_builder =
                self
                .to_named_file(file_location, &metadata)
                .content_type(content_type);

            Ok(named_file_builder)
        } else {
            Err(MatrixError::not_yet_uploaded("Media has not been uploaded yet").into())
        }
    }

    pub async fn temp_file(&self, file_media_id: &str, metadata: &DbMetadata) -> AppResult<PathBuf> {

        let mut dest_path = self.get_dest_path();

        dest_path.push(file_media_id);

        let file_object =
            self.client
            .get_object()
            .bucket(&self.bucket)
            .key(dest_path.to_str().expect("Could not build path"))
            .send()
            .await
            .map_err(|e| { error!("Got error {e:?}"); AppError::public("Could not fetch the file")})?;

        let mut location: PathBuf = std::env::temp_dir();

        if let Some(filename) = &metadata.file_name {
            location.push(filename.clone());
        } else {
            location.push(file_media_id);
        }

        let body =
            file_object
            .body
            .collect()
            .await
            .map_err(|_| { AppError::public("Found empty file")})?
            .into_bytes();


        if let Ok(mut file) = File::create_new(&location).await {
            // In any other case, file already exists and we can just return the location of the file.
            file
            .write_all(&body)
            .await
            .map_err(|e| { error!("Write all {e:?}"); AppError::internal("Failed to save downloaded content into file") })?;
        }

        Ok(location)
    }

    fn to_named_file(&self, path: PathBuf, metadata: &DbMetadata) -> NamedFileBuilder {
        if let Some(file_name) = &metadata.file_name {
            NamedFile::builder(path).attached_name(file_name)
        } else {
            NamedFile::builder(path)
        }
    }

    fn get_dest_path(&self) -> PathBuf {
        let config = config::get();
        let mut path = PathBuf::from("media");
        path.push(&config.server_name.as_str());

        path
    }
}