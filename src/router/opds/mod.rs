mod opds_models;

use std::{borrow::Cow, num::NonZeroUsize};

use actix_web::{HttpResponse, Responder, get, http::header, web};
use opds_models as opds;
use quick_xml::se::to_string;
use serde::Deserialize;
use time::OffsetDateTime;

use crate::library::{BooksSortType, Libraries, Library};

pub const COMMON_ROUTE: &str = "/opds";

const XMLNS_ATOM: &str = "http://www.w3.org/2005/Atom";
const FEED_TITLE: &str = "Seshat – OPDS Catalog";
const FEED_AUTHOR: opds::Author = opds::Author {
    uri: Some(Cow::Borrowed("https://github.com/thunder04")),
    name: Cow::Borrowed("Thunder04"),
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(root)
        .service(library_root)
        .service(explore_catalog);
}

#[get("/")]
async fn root(libraries: web::Data<Libraries>) -> impl Responder {
    let mut updated_at = OffsetDateTime::UNIX_EPOCH;
    let entries = libraries
        .get_all()
        .map(|lib| {
            // Calculate the feed's "updated" field while we're at it.
            if lib.updated_at() > updated_at {
                updated_at = lib.updated_at();
            }

            opds::Entry {
                id: lib.acquisition_feed_id().to_string(),
                title: lib.name().to_string(),
                updated: lib.updated_at(),
                authors: vec![],
                categories: vec![],
                content: Some(opds::Content {
                    value: format!("Explore the \"{}\" library", lib.name()),
                    kind: opds::ContentKind::Text,
                }),
                links: vec![opds::Link {
                    href: Cow::Owned(format!("{COMMON_ROUTE}/{}", lib.name())),
                    kind: opds::LinkType::Acquisition.as_str(),
                    rel: None,
                }],
            }
        })
        .collect();

    HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_XML))
        .body(
            to_string(&opds::Feed {
                xmlns: XMLNS_ATOM,

                id: "urn:seshat:root".to_string(),
                title: FEED_TITLE.to_string(),
                subtitle: Some("Explore all available libraries".to_string()),
                updated: updated_at,
                authors: vec![FEED_AUTHOR],
                links: vec![opds::Link::start()],
                entries,
            })
            .expect("serialization failed"),
        )
}

#[get("/{lib_name}/")]
async fn library_root(
    libraries: web::Data<Libraries>,
    lib_name: web::Path<String>,
) -> impl Responder {
    let Some(lib) = libraries.get(&lib_name) else {
        return HttpResponse::NotFound().body("The library doesn't exist");
    };

    let mut entries = vec![opds::Entry {
        id: lib.acquisition_feed_id().to_string(),
        title: "View Books".into(),
        updated: lib.updated_at(),
        authors: vec![],
        categories: vec![],
        content: None,
        links: vec![opds::Link {
            href: Cow::Owned(format!("{COMMON_ROUTE}/{lib_name}/explore")),
            kind: opds::LinkType::Acquisition.as_str(),
            rel: None,
        }],
    }];

    entries.extend(
        [
            ("By Newest", "the date they were added", "date_added"),
            ("By Title", "title", "title"),
            ("By Author", "author", "author"),
            // TODO: Viewing books sorted by language? That's dumb. Group them instead.
            ("By Language", "language", "lang"),
            ("By Publisher", "publisher", "publisher"),
            ("By Rating", "rating", "rating"),
            ("By Series", "series", "series"),
            ("By Tags", "tags", "tags"),
        ]
        .into_iter()
        .map(|(title, sorted_by, sort)| opds::Entry {
            id: lib.acquisition_feed_id().to_string(),
            title: title.to_string(),
            updated: lib.updated_at(),
            authors: vec![],
            categories: vec![],
            content: Some(opds::Content {
                value: format!("View books sorted by {sorted_by}"),
                kind: opds::ContentKind::Text,
            }),
            links: vec![opds::Link {
                href: Cow::Owned(format!("{COMMON_ROUTE}/{lib_name}/explore?sort={sort}")),
                kind: opds::LinkType::Acquisition.as_str(),
                rel: None,
            }],
        }),
    );

    HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_XML))
        .body(
            to_string(&opds::Feed {
                xmlns: XMLNS_ATOM,

                id: lib.acquisition_feed_id().to_string(),
                title: FEED_TITLE.to_string(),
                subtitle: Some(format!("Exploring the \"{lib_name}\" library").to_string()),
                updated: lib.updated_at(),
                authors: vec![FEED_AUTHOR],
                links: vec![opds::Link::start()],
                entries,
            })
            .expect("serialization failed"),
        )
}

#[derive(Deserialize)]
struct ExploreCatalogQuery {
    sort: Option<BooksSortType>,
    offset: Option<usize>,
    limit: Option<NonZeroUsize>,
}

#[get("/{lib_name}/explore/")]
async fn explore_catalog(
    query: web::Query<ExploreCatalogQuery>,
    libraries: web::Data<Libraries>,
    lib_name: web::Path<String>,
) -> impl Responder {
    let Some(lib) = libraries.get(&lib_name) else {
        return HttpResponse::NotFound().body("The library doesn't exist");
    };

    let sort = query.sort.unwrap_or(BooksSortType::DateAdded);
    let offset = query.offset.unwrap_or(0);
    let limit = query
        .limit
        .unwrap_or(unsafe { NonZeroUsize::new_unchecked(25) })
        .clamp(Library::MIN_PAGE_SIZE, Library::MAX_PAGE_SIZE);

    // TODO: Construct FullBook
    // Idea: SELECT DISTINCT other entities, based on fetched books

    let (entries, next_page) = lib
        .fetch_books(limit, offset, sort, move |book| {
            Ok(opds::Entry {
                id: book.uri(),
                title: book.title.to_string(),
                updated: book.last_modified_at,
                authors: vec![],
                categories: vec![],
                content: Some(opds::Content {
                    kind: opds::ContentKind::Text,
                    value: "Hi".into(),
                }),
                links: vec![opds::Link {
                    kind: opds::LinkType::Acquisition.as_str(),
                    href: "https://example.com".into(),
                    rel: None,
                }],
            })
        })
        .await
        .unwrap(); // TODO: Better error handling

    HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_XML))
        .body(
            to_string(&opds::Feed {
                xmlns: XMLNS_ATOM,

                id: lib.acquisition_feed_id().to_string(),
                title: FEED_TITLE.to_string(),
                subtitle: Some(format!("Exploring the \"{lib_name}\" library").to_string()),
                updated: lib.updated_at(),
                authors: vec![FEED_AUTHOR],
                links: vec![opds::Link::start()],
                entries,
            })
            .expect("serialization failed"),
        )
}
