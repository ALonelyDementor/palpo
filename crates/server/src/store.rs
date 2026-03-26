mod bucket;
mod filesystem;

use std::sync::OnceLock;

pub use bucket::Bucket;
pub use filesystem::FileSystem;

use bytes::Bytes;
use salvo::fs::NamedFileBuilder;

use crate::config::StoreEngine;

use crate::{AppResult, config};
use core::future::Future;

// Personally, I don't like this solution.
// A proper way to access store, should be via Palpo state, HOWEVER this would incur in a lot of changes
// Hence for now, let's move with the approach of a static variable.
pub static STORE: OnceLock<Store> = OnceLock::new();

#[derive(Debug)]
pub enum Store {
    FileSystem(FileSystem),
    Bucket(Bucket),
}

pub trait StoreActions: Send + 'static {
    fn get(&self, server: &str, key: &str) -> impl Future<Output = AppResult<NamedFileBuilder>>;
    fn create(&self, server:&str, key: &str, payload: &Bytes) -> impl Future<Output = AppResult<()>>;
}

pub fn init_store() {
    let cfg = config::get();
    let store_config = &cfg.storage;

    match store_config.engine() {
        StoreEngine::Bucket =>
            Store::new_bucket_engine(),
        StoreEngine::FileSystem =>
            Store::new_filesystem_engine(),
    }

}

pub fn get() -> &'static Store {
    STORE.get().unwrap()
}

impl StoreActions for Store{
    async fn get(&self, _server_name: &str, key: &str) -> AppResult<NamedFileBuilder> {
        self.builder(key).await
    }

    async fn create(&self, _server_name: &str, key: &str, value: &Bytes) -> AppResult<()> {
        self.request_put(key, value).await
    }
}

impl Store {

    pub fn new_filesystem_engine() {
        STORE.set(Store::FileSystem(FileSystem{})).expect("Store is already set");
    }

    pub fn new_bucket_engine() {
        let bucket = Bucket::init().expect("Config not set");

        STORE.set(Store::Bucket(bucket)).expect("Store is already set");
    }

    async fn builder(&self, key: &str) -> AppResult<NamedFileBuilder> {
        // TODO: fix these returns
        match self {
            Store::Bucket(b) => b.builder(key).await,
            Store::FileSystem(f) => f.builder(key).await
        }
    }

    async fn request_put(&self, key: &str, value: &Bytes) -> AppResult<()> {
        match self {
            Store::Bucket(b) => b.put(key, value).await,
            Store::FileSystem(f) => f.create(key, value).await
        }
    }

}