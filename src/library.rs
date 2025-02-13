use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use async_sqlite::{Pool, PoolBuilder, rusqlite::OpenFlags};
use eyre::bail;
use time::OffsetDateTime;
use tokio::fs;

use crate::utils::hash_str;

/// Handles all Calibre libraries. It's responsible for reading the metadata.db file and
/// performing search operation of books.
pub struct Libraries {
    entries: HashMap<String, Library>,
}

impl Libraries {
    pub async fn from_cli(cli: &mut crate::Cli) -> eyre::Result<Self> {
        let mut paths = std::mem::take(&mut cli.lib_path).into_iter();
        let mut names = std::mem::take(&mut cli.lib_name).into_iter();
        let mut entries = HashMap::new();

        loop {
            match (names.next(), paths.next()) {
                (Some(name), Some(path)) => {
                    if entries
                        .insert(name.clone(), Library::new(name, path).await?)
                        .is_some()
                    {
                        bail!("library names must be unique");
                    }
                }

                (None, None) => break,
                _ => bail!("each --lib:name must have a corresponding --lib:path"),
            }
        }

        Ok(Self { entries })
    }

    pub fn get_library(&self, name: &str) -> Option<&Library> {
        self.entries.get(name)
    }

    pub fn all_libraries(&self) -> impl Iterator<Item = &Library> {
        self.entries.values()
    }
}

// TODO: Use full text search database too if it's available?
pub struct Library {
    modified_at: OffsetDateTime,
    root_path: PathBuf,
    metadata_db: Pool,
    name: String,
    acquisition_feed_id: String,
}

impl Library {
    async fn new(name: String, lib_path: PathBuf) -> eyre::Result<Self> {
        let root_path = fs::canonicalize(&lib_path).await?;
        debug!(lib_name = %name, "Canonicalized path {lib_path:?} => {root_path:?}");

        let metadata_db_path = root_path.join("metadata.db");
        let metadata_db_file_metadata = fs::metadata(&metadata_db_path).await?;
        let modified_at = metadata_db_file_metadata
            .modified()
            .or_else(|_| metadata_db_file_metadata.created())
            .map(OffsetDateTime::from)
            .expect("neither modified_at and created_at dates are supported in this platform");
        let metadata_db = PoolBuilder::new()
            .flags(OpenFlags::SQLITE_OPEN_READ_ONLY)
            .path(metadata_db_path)
            .open()
            .await?;
        debug!(mtime = ?modified_at, lib_name = %name, "Opened \"metadata.db\"");

        Ok(Self {
            acquisition_feed_id: format!("urn:seshat:lib-{}", hash_str(&name)),
            metadata_db,
            modified_at,
            root_path,
            name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn modified_at(&self) -> OffsetDateTime {
        self.modified_at
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn acquisition_feed_id(&self) -> &str {
        &self.acquisition_feed_id
    }
}
