use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("ZIP error")]
    Zip(#[from] zip::result::ZipError),
    #[error("RocksDb error")]
    Db(#[from] rocksdb::Error),
}
