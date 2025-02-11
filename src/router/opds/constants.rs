use std::borrow::Cow;

pub const XMLNS_ATOM: &str = "http://www.w3.org/2005/Atom";
pub const FEED_TITLE: Cow<'static, str> = Cow::Borrowed("Simple Calibre Catalog");
pub const FEED_AUTHOR: super::types::Author = super::types::Author {
    uri: Cow::Borrowed("https://github.com/thunder04"),
    name: Cow::Borrowed("Thunder04"),
};

pub enum LinkType {
    AcquisitionFeed,
    Navigation,
    Search,
}

impl LinkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AcquisitionFeed => "application/atom+xml;profile=opds-catalog;kind=acquisition",
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
