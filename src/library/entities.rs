use async_sqlite::rusqlite::{Error, Row};
use time::OffsetDateTime;

#[derive(Debug)]
pub struct FullBook {
    pub id: i64,
    pub uuid: Option<String>,
    pub title: String,
    pub added_at: Option<OffsetDateTime>,
    pub published_at: Option<OffsetDateTime>,
    pub last_modified_at: OffsetDateTime,
    pub path: String,
    pub has_cover: bool,
    pub authors: Vec<String>,
    pub languages: Vec<String>,
    pub tags: Vec<String>,
    pub content: Option<String>,
    pub data: Vec<Data>,
}

impl TryFrom<&Row<'_>> for FullBook {
    type Error = Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        let path = row.get::<_, String>("path")?;

        Ok(Self {
            last_modified_at: row.get("last_modified_at")?,
            has_cover: row.get::<_, bool>("has_cover")?,
            published_at: row.get("published_at")?,
            added_at: row.get("added_at")?,
            content: row.get("comment")?,
            title: row.get("title")?,
            uuid: row.get("uuid")?,
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
    pub fn uri(&self) -> String {
        if let Some(uuid) = &self.uuid {
            return format!("urn:uuid:{uuid}");
        }

        format!("urn:id:{}", self.id)
    }
}

pub struct Author {
    pub name: String,
    pub book_id: i64,
}

impl TryFrom<&Row<'_>> for Author {
    type Error = Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: row.get("author_name")?,
            book_id: row.get("book_id")?,
        })
    }
}

pub struct Language {
    pub lang_code: String,
    pub book_id: i64,
}

impl TryFrom<&Row<'_>> for Language {
    type Error = Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            lang_code: row.get("lang_code")?,
            book_id: row.get("book_id")?,
        })
    }
}

pub struct Tag {
    pub name: String,
    pub book_id: i64,
}

impl TryFrom<&Row<'_>> for Tag {
    type Error = Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: row.get("tag_name")?,
            book_id: row.get("book_id")?,
        })
    }
}

#[derive(Debug)]
pub struct Data {
    pub file_name: String,
    pub file_size: i64,
    pub format: String,
    pub book_id: i64,
}

impl TryFrom<&Row<'_>> for Data {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let mut format: String = row.get("format")?;

        format.make_ascii_lowercase();

        Ok(Self {
            file_size: row.get("file_size")?,
            file_name: row.get("file_name")?,
            book_id: row.get("book_id")?,
            format,
        })
    }
}
