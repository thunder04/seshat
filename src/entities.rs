use async_sqlite::rusqlite::{Error, Row};
use time::OffsetDateTime;

#[derive(Debug)]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub sort: Option<String>,
    pub timestamp: Option<OffsetDateTime>,
    pub pubdate: Option<OffsetDateTime>,
    pub series_index: f64,
    pub author_sort: Option<String>,
    pub isbn: Option<String>,
    pub lccn: Option<String>,
    pub path: String,
    pub flags: i32,
    pub uuid: Option<String>,
    pub has_cover: Option<bool>,
    pub last_modified: OffsetDateTime,
}

impl TryFrom<&Row<'_>> for Book {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            title: row.get("title")?,
            sort: row.get("sort")?,
            timestamp: row.get("timestamp")?,
            pubdate: row.get("pubdate")?,
            series_index: row.get("series_index")?,
            author_sort: row.get("author_sort")?,
            isbn: row.get("isbn")?,
            lccn: row.get("lccn")?,
            path: row.get("path")?,
            flags: row.get("flags")?,
            uuid: row.get("uuid")?,
            has_cover: row.get("has_cover")?,
            last_modified: row.get("last_modified")?,
        })
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
    pub link: String,
}

impl TryFrom<&Row<'_>> for Language {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            lang_code: row.get("lang_code")?,
            link: row.get("link")?,
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
    pub link: String,
}

impl TryFrom<&Row<'_>> for Rating {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("id")?,
            rating: row.get("rating")?,
            link: row.get("link")?,
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
