mod entities;
mod sql;

use std::{
    collections::HashMap,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

use async_sqlite::{Pool, PoolBuilder, rusqlite};
use entities::{Author, Language, Tag};
pub use entities::{Data, FullBook};
use eyre::bail;
use serde::Deserialize;
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

    pub fn get(&self, name: &str) -> Option<&Library> {
        self.entries.get(name)
    }

    pub fn get_all(&self) -> impl Iterator<Item = &Library> {
        self.entries.values()
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum OrderBooksBy {
    DateAdded,
    Title,
    Author,
    // TODO: Group by the following options instead.
    // Language,
    // Tags,
    // Publisher,
    // Series,
}

impl OrderBooksBy {
    fn as_sql_query(&self) -> &'static dyn sql::SqlQueries {
        match self {
            Self::DateAdded => &sql::OrderedByDateAdded,
            Self::Author => &sql::OrderedByAuthor,
            Self::Title => &sql::OrderedByTitle,
        }
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
    pub const DEFAULT_PAGE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(25) };
    pub const MAX_PAGE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(50) };
    pub const MIN_PAGE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };

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
            .flags(rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
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

    pub fn acquisition_feed_id(&self) -> &str {
        &self.acquisition_feed_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn updated_at(&self) -> OffsetDateTime {
        self.modified_at
    }

    /// Fetches a page of books from the library. Returns `true` if there is a next page.
    #[allow(
        clippy::unwrap_or_default,
        reason = "Default impl takes away type info from the compiler"
    )]
    pub async fn fetch_books<A, F>(
        &self,
        limit: NonZeroUsize,
        offset: usize,
        order_by: OrderBooksBy,
        mut acc: A,
        mut f: F,
    ) -> crate::Result<(A, bool)>
    where
        F: FnMut(A, FullBook) -> A + Send + 'static,
        A: Send + 'static,
    {
        Ok(self
            .metadata_db
            .conn(move |conn| {
                let sql_query = order_by.as_sql_query();

                let mut authors = conn
                    .prepare_cached(sql_query.retrieve_book_authors())?
                    .query_map([limit.get(), offset], |row| Author::try_from(row))?
                    .try_fold(HashMap::new(), |mut acc, row| -> rusqlite::Result<_> {
                        let row = row?;

                        acc.entry(row.book_id).or_insert(vec![]).push(row.name);
                        Ok(acc)
                    })?;

                let mut languages = conn
                    .prepare_cached(sql_query.retrieve_book_languages())?
                    .query_map([limit.get(), offset], |row| Language::try_from(row))?
                    .try_fold(HashMap::new(), |mut acc, row| -> rusqlite::Result<_> {
                        let row = row?;

                        acc.entry(row.book_id).or_insert(vec![]).push(row.lang_code);
                        Ok(acc)
                    })?;

                let mut tags = conn
                    .prepare_cached(sql_query.retrieve_book_tags())?
                    .query_map([limit.get(), offset], |row| Tag::try_from(row))?
                    .try_fold(HashMap::new(), |mut acc, row| -> rusqlite::Result<_> {
                        let row = row?;

                        acc.entry(row.book_id).or_insert(vec![]).push(row.name);
                        Ok(acc)
                    })?;

                let mut data = conn
                    .prepare_cached(sql_query.retrieve_book_data())?
                    .query_map([limit.get(), offset], |row| Data::try_from(row))?
                    .try_fold(HashMap::new(), |mut acc, row| -> rusqlite::Result<_> {
                        let row = row?;

                        acc.entry(row.book_id).or_insert(vec![]).push(row);
                        Ok(acc)
                    })?;

                let mut stmt = conn.prepare_cached(sql_query.retrieve_books())?;
                let mut books =
                    stmt.query_map([limit.get() + 1, offset], |row| FullBook::try_from(row))?;

                for _ in 0..limit.get() {
                    let Some(book) = books.next().transpose()? else {
                        return Ok((acc, false));
                    };

                    acc = f(acc, FullBook {
                        languages: languages.remove(&book.id).unwrap_or_default(),
                        authors: authors.remove(&book.id).unwrap_or_default(),
                        data: data.remove(&book.id).unwrap_or_default(),
                        tags: tags.remove(&book.id).unwrap_or_default(),
                        ..book
                    });
                }

                Ok((acc, books.next().is_some()))
            })
            .await?)
    }
}
