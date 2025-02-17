use std::{
    collections::HashMap,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

use async_sqlite::{Pool, PoolBuilder, rusqlite};
use eyre::bail;
use serde::Deserialize;
use time::OffsetDateTime;
use tokio::fs;

use crate::{
    metadata_entities::{Author, Data, FullBook, Language, Tag},
    utils::hash_str,
};

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

// TODO: Use full text search database too if it's available?
pub struct Library {
    modified_at: OffsetDateTime,
    root_path: PathBuf,
    metadata_db: Pool,
    name: String,
    acquisition_feed_id: String,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum BooksSortType {
    DateAdded,
    Title,
    Author,
    #[serde(rename = "lang")]
    Language,
    Publisher,
    Rating,
    Series,
    Tags,
}

impl BooksSortType {
    pub fn as_sql_column(&self) -> &'static str {
        match self {
            Self::DateAdded => "timestamp",
            Self::Title => "title",
            Self::Author => "author_sort",
            Self::Language => "lang",
            Self::Publisher => "publisher",
            Self::Rating => "rating",
            Self::Series => "series",
            Self::Tags => "tags",
        }
    }
}

impl Library {
    pub const MIN_PAGE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };
    pub const MAX_PAGE_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(50) };

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
    pub async fn fetch_books<F>(
        &self,
        limit: NonZeroUsize,
        offset: usize,
        sort_by: BooksSortType,
        mut f: F,
    ) -> crate::Result<bool>
    where
        F: FnMut(FullBook) -> rusqlite::Result<()> + Send + Sync + 'static,
    {
        const BASE_QUERY: &str = r#"
            SELECT
               	b.id AS id,
               	b.uuid AS uuid,
               	b.title AS title,
               	b.timestamp AS added_at,
               	b.pubdate AS published_at,
               	b.has_cover AS has_cover,
               	b.last_modified AS last_modified_at,
               	b.path AS path,
               	c.text AS comment
            FROM books as b
          		LEFT JOIN comments as c ON c.book = b.id
           	ORDER BY author_sort
           	LIMIT ?1 OFFSET ?2"#;

        const RETRIEVE_BOOK_AUTHORS: &str = r#"
            SELECT
                a.name AS author_name,
                link.book AS book_id
            FROM books_authors_link as link
               	INNER JOIN (
              		SELECT id AS b_id FROM books
              		ORDER BY author_sort
              		LIMIT ?1 OFFSET ?2
               	) ON book_id = b_id
               	INNER JOIN authors AS a ON link.author = a.id;"#;

        const RETRIEVE_BOOK_LANGUAGES: &str = r#"
            SELECT
                l.lang_code AS lang_code,
                link.book AS book_id
            FROM books_languages_link as link
           	INNER JOIN (
          		SELECT id AS b_id FROM books
          		ORDER BY author_sort
          		LIMIT ?1 OFFSET ?2
           	) ON book_id = b_id
           	INNER JOIN languages AS l ON link.lang_code = l.id;"#;

        const RETRIEVE_BOOK_TAGS: &str = r#"
            SELECT
                link.book AS book_id,
                t.name AS tag_name
            FROM books_tags_link as link
           	INNER JOIN (
          		SELECT id AS b_id FROM books
          		ORDER BY author_sort
          		LIMIT ?1 OFFSET ?2
           	) ON book_id = b_id
           	INNER JOIN tags AS t ON link.tag = t.id;"#;

        const RETRIEVE_BOOK_DATA: &str = r#"
            SELECT
                d.uncompressed_size AS file_size,
                d.name AS file_name,
                d.format AS format,
                d.book AS book_id
            FROM data AS d
           	WHERE book_id IN (
          		SELECT id AS b_id FROM books
          		ORDER BY author_sort
          		LIMIT ?1 OFFSET ?2);"#;

        Ok(self
            .metadata_db
            .conn(move |conn| {
                let mut authors = conn
                    .prepare_cached(RETRIEVE_BOOK_AUTHORS)?
                    .query_map([limit.get(), offset], |row| Author::try_from(row))?
                    .try_fold(HashMap::new(), |mut acc, row| -> rusqlite::Result<_> {
                        let row = row?;

                        acc.entry(row.book_id).or_insert(vec![]).push(row.name);
                        Ok(acc)
                    })?;

                let mut languages = conn
                    .prepare_cached(RETRIEVE_BOOK_LANGUAGES)?
                    .query_map([limit.get(), offset], |row| Language::try_from(row))?
                    .try_fold(HashMap::new(), |mut acc, row| -> rusqlite::Result<_> {
                        let row = row?;

                        acc.entry(row.book_id).or_insert(vec![]).push(row.lang_code);
                        Ok(acc)
                    })?;

                let mut tags = conn
                    .prepare_cached(RETRIEVE_BOOK_TAGS)?
                    .query_map([limit.get(), offset], |row| Tag::try_from(row))?
                    .try_fold(HashMap::new(), |mut acc, row| -> rusqlite::Result<_> {
                        let row = row?;

                        acc.entry(row.book_id).or_insert(vec![]).push(row.name);
                        Ok(acc)
                    })?;

                let mut data = conn
                    .prepare_cached(RETRIEVE_BOOK_DATA)?
                    .query_map([limit.get(), offset], |row| Data::try_from(row))?
                    .try_fold(HashMap::new(), |mut acc, row| -> rusqlite::Result<_> {
                        let row = row?;

                        acc.entry(row.book_id).or_insert(vec![]).push(row);
                        Ok(acc)
                    })?;

                let mut stmt = conn.prepare_cached(BASE_QUERY)?;
                let mut books =
                    stmt.query_map([limit.get() + 1, offset], |row| FullBook::try_from(row))?;

                for _ in 0..limit.get() {
                    let Some(book) = books.next().transpose()? else {
                        return Ok(false);
                    };

                    f(FullBook {
                        languages: languages.remove(&book.id).unwrap_or_default(),
                        authors: authors.remove(&book.id).unwrap_or_default(),
                        data: data.remove(&book.id).unwrap_or_default(),
                        tags: tags.remove(&book.id).unwrap_or_default(),
                        ..book
                    })?;
                }

                Ok(books.next().is_some())
            })
            .await?)
    }
}
