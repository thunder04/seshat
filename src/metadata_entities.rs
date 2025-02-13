use std::path::PathBuf;

use async_sqlite::rusqlite::{Error, Row};
use time::OffsetDateTime;

pub struct FullBook {
    pub id: i64,
    pub uuid: Option<String>,
    pub title: String,
    pub added_at: Option<OffsetDateTime>,
    pub published_at: Option<OffsetDateTime>,
    pub last_modified_at: OffsetDateTime,
    pub path: PathBuf,
    pub cover_path: Option<PathBuf>,
    pub authors: Vec<Author>,
    pub languages: Vec<Language>,
    pub ratings: Option<Rating>,
    pub series: Option<Series>,
    pub tags: Vec<Tag>,
    pub comments: Option<Comment>,
    pub data: Vec<Data>,
}

#[derive(Debug)]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub added_at: Option<OffsetDateTime>,
    pub published_at: Option<OffsetDateTime>,
    pub last_modified_at: OffsetDateTime,
    pub series_index: f64,
    pub isbn: Option<String>,
    pub lccn: Option<String>,
    pub path: String,
    pub flags: i32,
    pub uuid: Option<String>,
    pub has_cover: Option<bool>,
}

impl TryFrom<&Row<'_>> for Book {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            title: row.get("title")?,
            added_at: row.get("timestamp")?,
            published_at: row.get("pubdate")?,
            series_index: row.get("series_index")?,
            isbn: row.get("isbn")?,
            lccn: row.get("lccn")?,
            path: row.get("path")?,
            flags: row.get("flags")?,
            uuid: row.get("uuid")?,
            has_cover: row.get("has_cover")?,
            last_modified_at: row.get("last_modified")?,
        })
    }
}

impl Book {
    /// Returns the URI of the book.
    pub fn uri(&self) -> String {
        if let Some(uuid) = &self.uuid {
            return format!("urn:uuid:{uuid}");
        }

        format!("urn:id:{}", self.id)
    }
}

#[derive(Debug)]
pub struct BooksAuthorsLink {
    pub id: i64,
    pub book: i64,
    pub author: i64,
}

impl TryFrom<&Row<'_>> for BooksAuthorsLink {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            book: row.get("book")?,
            author: row.get("author")?,
        })
    }
}

#[derive(Debug)]
pub struct BooksLanguagesLink {
    pub id: i64,
    pub book: i64,
    pub lang_code: i64,
    pub item_order: i64,
}

impl TryFrom<&Row<'_>> for BooksLanguagesLink {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            book: row.get("book")?,
            lang_code: row.get("lang_code")?,
            item_order: row.get("item_order")?,
        })
    }
}

#[derive(Debug)]
pub struct BooksPublishersLink {
    pub id: i64,
    pub book: i64,
    pub publisher: i64,
}

impl TryFrom<&Row<'_>> for BooksPublishersLink {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            book: row.get("book")?,
            publisher: row.get("publisher")?,
        })
    }
}

#[derive(Debug)]
pub struct BooksRatingsLink {
    pub id: i64,
    pub book: i64,
    pub rating: i64,
}

impl TryFrom<&Row<'_>> for BooksRatingsLink {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            book: row.get("book")?,
            rating: row.get("rating")?,
        })
    }
}

#[derive(Debug)]
pub struct BooksSeriesLink {
    pub id: i64,
    pub book: i64,
    pub series: i64,
}

impl TryFrom<&Row<'_>> for BooksSeriesLink {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            book: row.get("book")?,
            series: row.get("series")?,
        })
    }
}

#[derive(Debug)]
pub struct BooksTagsLink {
    pub id: i64,
    pub book: i64,
    pub tag: i64,
}

impl TryFrom<&Row<'_>> for BooksTagsLink {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            book: row.get("book")?,
            tag: row.get("tag")?,
        })
    }
}

#[derive(Debug)]
pub struct Comment {
    pub id: i64,
    pub book: i64,
    pub text: String,
}

impl TryFrom<&Row<'_>> for Comment {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            book: row.get("book")?,
            text: row.get("text")?,
        })
    }
}

#[derive(Debug)]
pub struct Author {
    pub id: i64,
    pub name: String,
    pub sort: Option<String>,
    pub link: String,
}

impl TryFrom<&Row<'_>> for Author {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            sort: row.get("sort")?,
            link: row.get("link")?,
        })
    }
}

#[derive(Debug)]
pub struct Language {
    pub id: i64,
    pub lang_code: String,
}

impl TryFrom<&Row<'_>> for Language {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            lang_code: row.get("lang_code")?,
            id: row.get("id")?,
        })
    }
}

#[derive(Debug)]
pub struct Publisher {
    pub id: i64,
    pub name: String,
    pub sort: Option<String>,
    pub link: String,
}

impl TryFrom<&Row<'_>> for Publisher {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            sort: row.get("sort")?,
            link: row.get("link")?,
        })
    }
}

#[derive(Debug)]
pub struct Rating {
    pub id: i64,
    pub rating: i64,
}

impl TryFrom<&Row<'_>> for Rating {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            rating: row.get("rating")?,
            id: row.get("id")?,
        })
    }
}

#[derive(Debug)]
pub struct Series {
    pub id: i64,
    pub name: String,
    pub sort: Option<String>,
    pub link: String,
}

impl TryFrom<&Row<'_>> for Series {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            sort: row.get("sort")?,
            link: row.get("link")?,
        })
    }
}

#[derive(Debug)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub link: String,
}

impl TryFrom<&Row<'_>> for Tag {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
            link: row.get("link")?,
        })
    }
}

#[derive(Debug)]
pub struct Data {
    pub id: i64,
    pub book: i64,
    pub format: String,
    pub uncompressed_size: i64,
    pub name: String,
}

impl TryFrom<&Row<'_>> for Data {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            book: row.get("book")?,
            format: row.get("format")?,
            uncompressed_size: row.get("uncompressed_size")?,
            name: row.get("name")?,
        })
    }
}
