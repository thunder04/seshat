use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use async_sqlite::{Pool, PoolBuilder, rusqlite::OpenFlags};
use eyre::bail;
use tokio::fs;

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
}

struct Library {
    root_path: PathBuf,
    metadata_db: Pool,
    name: String,
}

impl Library {
    async fn new(name: String, path: PathBuf) -> eyre::Result<Self> {
        let root_path = fs::canonicalize(path).await?;
        let metadata_db = PoolBuilder::new()
            .flags(OpenFlags::SQLITE_OPEN_READ_ONLY)
            .path(root_path.join("metadata.db"))
            .open()
            .await?;

        Ok(Self {
            metadata_db,
            root_path,
            name,
        })
    }
}
