use std::borrow::Cow;

use serde::Serialize;
use time::{OffsetDateTime, serde::rfc3339};

use crate::library::Library;

pub struct LibraryRootEntry {
    pub title: &'static str,
    pub description: &'static str,
    pub sort_by: Option<&'static str>,
    pub link_rel: Option<LinkRel>,
}

impl From<(&Library, LibraryRootEntry)> for Entry {
    fn from((lib, e): (&Library, LibraryRootEntry)) -> Self {
        let mut href = format!("{}/{}/explore", super::COMMON_ROUTE, lib.name());

        if let Some(sort_by) = e.sort_by {
            href += "?sort=";
            href += sort_by;
        }

        Self {
            id: lib.acquisition_feed_id().to_string(),
            title: e.title.to_string(),
            updated: lib.updated_at(),
            authors: vec![],
            categories: vec![],
            content: Some(Content {
                value: e.description.to_string(),
                kind: ContentKind::Text,
            }),
            links: vec![Link {
                kind: LinkType::Acquisition.as_str(),
                rel: e.link_rel.map(|x| x.as_str()),
                href: Cow::Owned(href),
            }],
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename = "feed", rename_all = "kebab-case")]
pub struct Feed {
    #[serde(rename = "@xlmns")]
    pub xmlns: &'static str,

    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(with = "rfc3339")]
    pub updated: OffsetDateTime,
    /// Specification says there must be at least one author.
    #[serde(rename = "author", skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<Author>,
    #[serde(rename = "link", skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
    #[serde(rename = "entry", skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Entry {
    pub id: String,
    pub title: String,
    #[serde(with = "rfc3339")]
    pub updated: OffsetDateTime,
    /// Specification says there must be at least one author.
    #[serde(rename = "author", skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<Author>,
    #[serde(rename = "category", skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<Category>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
    #[serde(rename = "link", skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Author {
    pub name: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<Cow<'static, str>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Category {
    #[serde(rename = "@term")]
    pub term: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Link {
    #[serde(rename = "@href")]
    pub href: Cow<'static, str>,
    #[serde(rename = "@rel", skip_serializing_if = "Option::is_none")]
    pub rel: Option<&'static str>,
    #[serde(rename = "@type")]
    pub kind: &'static str,
}

impl Link {
    pub fn start() -> Self {
        Self {
            href: Cow::Borrowed(super::COMMON_ROUTE),
            kind: LinkType::Navigation.as_str(),
            rel: Some(LinkRel::Start.as_str()),
        }
    }
}

pub enum LinkType {
    Acquisition,
    Navigation,
    Search,
}

impl LinkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Acquisition => "application/atom+xml;profile=opds-catalog;kind=acquisition",
            Self::Navigation => "application/atom+xml;profile=opds-catalog;kind=navigation",
            Self::Search => "application/opensearchdescription+xml",
        }
    }
}

pub enum LinkRel {
    Acquisition,
    SortNew,
    Image,
    Start,
}

impl LinkRel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Acquisition => "http://opds-spec.org/acquisition",
            Self::SortNew => "http://opds-spec.org/sort/new",
            Self::Image => "http://opds-spec.org/image",
            Self::Start => "start",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Content {
    #[serde(rename = "@type")]
    pub kind: ContentKind,
    #[serde(rename = "$text")]
    pub value: String,
}

#[non_exhaustive]
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentKind {
    Text,
    Html,
}
