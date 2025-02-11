mod types;

use std::borrow::Cow;

use actix_web::{HttpResponse, Responder, get, http::header, web};
use quick_xml::se::to_string;
use serde::Deserialize;
use time::OffsetDateTime;
use types::LinkType;

use crate::utils::determine_possesive;

pub const XMLNS_ATOM: &str = "http://www.w3.org/2005/Atom";

pub const FEED_TITLE: Cow<'static, str> = Cow::Borrowed("Seshat");
pub const FEED_AUTHOR: types::Author = types::Author {
    uri: Cow::Borrowed("https://github.com/thunder04"),
    name: Cow::Borrowed("Thunder04"),
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(root)
        .service(library_root)
        .service(explore_catalog);
}

#[get("/")]
async fn root() -> impl Responder {
    // Future-proof the possibility to load multiple libraries.
    let libraries = &[("Default Library", "_")];

    let entries = libraries
        .iter()
        .map(|(name, path)| types::Entry {
            title: Cow::Borrowed(name),
            link: types::Link {
                href: Cow::Owned(format!("/opds/{path}/")),
                kind: LinkType::Acquisition.as_str(),
                rel: None,
            },

            content: Some(types::Content {
                kind: types::ContentKind::Text,
                value: Cow::Owned(format!(
                    "Explore \"{name}\"{s} Catalog",
                    s = determine_possesive(name)
                )),
            }),
        })
        .collect::<Vec<_>>();
    let feed = types::AcquisitionFeed {
        xmlns: XMLNS_ATOM,

        // TODO: What should I change this to? Perhaps to a hash of modified at dates of all
        // libraries?
        id: "urn:uuid:2853dacf-ed79-42f5-8e8a-a7bb3d1ae6a2".into(),
        subtitle: Some("Explore available libraries".into()),
        links: vec![types::Link::start()],
        author: FEED_AUTHOR,
        title: FEED_TITLE,

        // TODO: Set to Modified at time of the newest metadata.db
        updated: OffsetDateTime::now_utc(),

        entries,
    };

    HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_XML))
        .body(to_string(&feed).expect("serialization failed"))
}

#[get("/{lib_path}/")]
async fn library_root(lib_path: web::Path<String>) -> impl Responder {
    // let lib_name = &**lib_path; // Currently they are the same...
    let lib_name = "Default Library"; // ...but I don't support multiple libraries.
    let mut entries = [
        ("By Date", "the date they were added", "date_added"),
        ("By Title", "title", "title"),
        ("By Author", "author", "author"),
        ("By Language", "language", "lang"),
        ("By Publisher", "publisher", "publisher"),
        ("By Rating", "rating", "rating"),
        ("By Series", "series", "series"),
        ("By Tags", "tags", "tags"),
    ]
    .iter()
    .map(|(title, sorted_by, sort_param)| types::Entry {
        title: Cow::Owned(title.to_string()),
        link: types::Link {
            href: Cow::Owned(format!("/opds/{lib_path}/explore?sort={sort_param}")),
            kind: LinkType::Acquisition.as_str(),
            rel: None,
        },

        content: Some(types::Content {
            value: Cow::Owned(format!("View books sorted by {sorted_by}")),
            kind: types::ContentKind::Text,
        }),
    })
    .collect::<Vec<_>>();

    entries.push(types::Entry {
        title: "View Books".into(),
        content: None,
        link: types::Link {
            href: Cow::Owned(format!("/opds/{lib_path}/explore")),
            kind: LinkType::Acquisition.as_str(),
            rel: None,
        },
    });

    let subtitle = format!(
        "Explore \"{lib_name}\"{s} catalog",
        s = determine_possesive(lib_name)
    );
    let feed = types::AcquisitionFeed {
        xmlns: XMLNS_ATOM,

        // TODO: What should I change this to? Perhaps to a hash of modified at date of metadata.db?
        id: "urn:uuid:2853dacf-ed79-42f5-8e8a-a7bb3d1ae6a2".into(),
        subtitle: Some(Cow::Owned(subtitle)),
        links: vec![types::Link::start()],
        author: FEED_AUTHOR,
        title: FEED_TITLE,
        // TODO: Set to Modified at time of metadata.db.
        updated: OffsetDateTime::now_utc(),

        entries,
    };

    HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_XML))
        .body(to_string(&feed).expect("serialization failed"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ExploreCatalogSortType {
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

#[derive(Deserialize)]
struct ExploreCatalogQuery {
    sort: Option<ExploreCatalogSortType>,
    offset: Option<u32>,
    limit: Option<u32>,
}

#[get("/{lib_path}/explore/")]
async fn explore_catalog(
    lib_path: web::Path<String>, query: web::Query<ExploreCatalogQuery>,
) -> impl Responder {
    let ExploreCatalogQuery {
        sort,
        offset,
        limit,
    } = query.into_inner();
    let limit = limit.unwrap_or(25).clamp(1, 50);
    let offset = offset.unwrap_or(0);

    let feed = types::AcquisitionFeed {
        xmlns: XMLNS_ATOM,

        // TODO: What should I change this to? Perhaps to a hash of modified at date of metadata.db?
        id: "urn:uuid:2853dacf-ed79-42f5-8e8a-a7bb3d1ae6a2".into(),
        links: vec![types::Link::start()],
        title: FEED_TITLE,
        author: FEED_AUTHOR,
        subtitle: None,
        // TODO: Set to Modified at time of metadata.db.
        updated: OffsetDateTime::now_utc(),

        entries: vec![],
    };

    HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_XML))
        .body(to_string(&feed).expect("serialization failed"))
}
