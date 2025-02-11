use std::borrow::Cow;

use serde::Serialize;
use time::{OffsetDateTime, serde::rfc3339};

// The ordering of elements is specified as listed [in the specification](https://specs.opds.io/opds-1.2#511-relationship-between-atom-and-dublin-core-metadata).
#[derive(Debug, Serialize)]
#[serde(rename = "feed", rename_all = "kebab-case")]
pub struct AcquisitionFeed {
    pub id: Cow<'static, str>,
    #[serde(with = "rfc3339")]
    pub updated: OffsetDateTime,
    pub title: Cow<'static, str>,
    pub subtitle: Option<Cow<'static, str>>,
    pub author: Author,

    #[serde(rename = "link")]
    pub links: Vec<Link>,
    #[serde(rename = "entry")]
    pub entries: Vec<Entry>,

    #[serde(rename = "@xlmns")]
    pub xmlns: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Author {
    pub name: Cow<'static, str>,
    pub uri: Cow<'static, str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Entry {
    pub title: Cow<'static, str>,
    pub link: Link,
    pub content: Option<Content>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Link {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@rel")]
    pub rel: Option<&'static str>,
    #[serde(rename = "@href")]
    pub href: Cow<'static, str>,
    #[serde(rename = "@type")]
    pub kind: &'static str,
}

impl Link {
    pub fn start() -> Self {
        Self {
            kind: LinkType::Navigation.as_str(),
            rel: Some(LinkRel::Start.as_str()),
            href: Cow::Borrowed("/opds/"),
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
    Start,
}

impl LinkRel {
    pub fn as_str(&self) -> &'static str {
        match self {
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
    pub value: Cow<'static, str>,
}

#[non_exhaustive]
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentKind {
    Text,
}
