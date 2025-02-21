use async_sqlite::rusqlite::{Error, Row};
use compact_str::{CompactString, format_compact};
use time::OffsetDateTime;

use crate::utils::CompactStringSql;

#[derive(Debug)]
pub struct FullBook {
    pub id: i64,
    pub uuid: Option<CompactString>,
    pub title: CompactString,
    pub added_at: Option<OffsetDateTime>,
    pub published_at: Option<OffsetDateTime>,
    pub last_modified_at: OffsetDateTime,
    pub path: CompactString,
    pub has_cover: bool,
    pub authors: Vec<CompactString>,
    pub languages: Vec<CompactString>,
    pub tags: Vec<CompactString>,
    pub content: Option<CompactString>,
    pub data: Vec<Data>,
}

impl TryFrom<&Row<'_>> for FullBook {
    type Error = Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        let path = row.get::<_, CompactStringSql>("path")?.0;

        Ok(Self {
            last_modified_at: row.get("last_modified_at")?,
            has_cover: row.get::<_, bool>("has_cover")?,
            published_at: row.get("published_at")?,
            added_at: row.get("added_at")?,
            content: row
                .get::<_, Option<CompactStringSql>>("comment")?
                .map(|str| str.0),
            title: row.get::<_, CompactStringSql>("title")?.0,
            uuid: row
                .get::<_, Option<CompactStringSql>>("uuid")?
                .map(|str| str.0),
            id: row.get("id")?,
            languages: vec![],
            authors: vec![],
            data: vec![],
            tags: vec![],
            path,
        })
    }
}

impl FullBook {
    /// Returns the URI of the book.
    pub fn uri(&self) -> CompactString {
        if let Some(uuid) = &self.uuid {
            return format_compact!("urn:uuid:{uuid}");
        }

        format_compact!("urn:id:{}", self.id)
    }
}

pub struct Author {
    pub name: CompactString,
    pub book_id: i64,
}

impl TryFrom<&Row<'_>> for Author {
    type Error = Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: row.get::<_, CompactStringSql>("author_name")?.0,
            book_id: row.get("book_id")?,
        })
    }
}

pub struct Language {
    pub lang_code: CompactString,
    pub book_id: i64,
}

impl TryFrom<&Row<'_>> for Language {
    type Error = Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            lang_code: row.get::<_, CompactStringSql>("lang_code")?.0,
            book_id: row.get("book_id")?,
        })
    }
}

pub struct Tag {
    pub name: CompactString,
    pub book_id: i64,
}

impl TryFrom<&Row<'_>> for Tag {
    type Error = Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: row.get::<_, CompactStringSql>("tag_name")?.0,
            book_id: row.get("book_id")?,
        })
    }
}

#[derive(Debug)]
pub struct Data {
    pub file_name: CompactString,
    pub file_size: i64,
    pub format: CompactString,
    pub book_id: i64,
}

impl TryFrom<&Row<'_>> for Data {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let mut format = row.get::<_, CompactStringSql>("format")?.0;
        format.make_ascii_lowercase();

        Ok(Self {
            file_name: row.get::<_, CompactStringSql>("file_name")?.0,
            file_size: row.get("file_size")?,
            book_id: row.get("book_id")?,
            format,
        })
    }
}
