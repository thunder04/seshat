mod models;

use std::{borrow::Cow, num::NonZeroUsize};

use actix_web::{HttpResponse, Responder, get, http::header, web};
use quick_xml::se::to_string;
use serde::Deserialize;
use time::OffsetDateTime;

use crate::{
    errors::AppError,
    library::{Libraries, Library, OrderBooksBy},
};

pub const COMMON_ROUTE: &str = "/opds";

const XMLNS_ATOM: &str = "http://www.w3.org/2005/Atom";
const FEED_TITLE: &str = "Seshat – OPDS Catalog";
const FEED_AUTHOR: models::Author = models::Author {
    uri: Some(Cow::Borrowed("https://github.com/thunder04")),
    name: Cow::Borrowed("Thunder04"),
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(root)
        .service(library_root)
        .service(explore_catalog);
}

#[get("")]
async fn root(libraries: web::Data<Libraries>) -> crate::Result<impl Responder> {
    let mut updated_at = OffsetDateTime::UNIX_EPOCH;
    let entries = libraries
        .get_all()
        .map(|lib| {
            // Calculate the feed's "updated" field while we're at it.
            if lib.updated_at() > updated_at {
                updated_at = lib.updated_at();
            }

            models::Entry {
                id: lib.acquisition_feed_id().to_string(),
                title: lib.name().to_string(),
                updated: lib.updated_at(),
                authors: vec![],
                categories: vec![],
                content: Some(models::Content {
                    value: format!("Explore the \"{}\" library", lib.name()),
                    kind: models::ContentKind::Text,
                }),
                links: vec![models::Link {
                    href: Cow::Owned(format!("{COMMON_ROUTE}/{}", lib.name())),
                    kind: models::LinkType::Acquisition.as_str(),
                    rel: None,
                }],
            }
        })
        .collect();

    Ok(HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_XML))
        .body(to_string(&models::Feed {
            xmlns: XMLNS_ATOM,
            id: "urn:seshat:root".to_string(),
            title: FEED_TITLE.to_string(),
            subtitle: Some("Explore all available libraries".to_string()),
            updated: updated_at,
            authors: vec![FEED_AUTHOR],
            links: vec![models::Link::start()],
            entries,
        })?))
}

#[get("/{lib_name}")]
async fn library_root(
    libraries: web::Data<Libraries>,
    lib_name: web::Path<String>,
) -> crate::Result<impl Responder> {
    let Some(lib) = libraries.get(&lib_name) else {
        return Err(AppError::LibraryNotFound);
    };

    let mut entries = vec![models::Entry {
        id: lib.acquisition_feed_id().to_string(),
        title: "View Books".into(),
        updated: lib.updated_at(),
        authors: vec![],
        categories: vec![],
        content: None,
        links: vec![models::Link {
            href: Cow::Owned(format!("{COMMON_ROUTE}/{lib_name}/explore")),
            kind: models::LinkType::Acquisition.as_str(),
            rel: None,
        }],
    }];

    entries.extend(
        [
            ("Sorted by Newest", "the date they were added", "date_added"),
            ("Sorted by Title", "title", "title"),
            ("Sorted by Author", "author", "author"),
        ]
        .into_iter()
        .map(|(title, sorted_by, sort)| models::Entry {
            id: lib.acquisition_feed_id().to_string(),
            title: title.to_string(),
            updated: lib.updated_at(),
            authors: vec![],
            categories: vec![],
            content: Some(models::Content {
                value: format!("View books sorted by {sorted_by}"),
                kind: models::ContentKind::Text,
            }),
            links: vec![models::Link {
                href: Cow::Owned(format!("{COMMON_ROUTE}/{lib_name}/explore?sort={sort}")),
                kind: models::LinkType::Acquisition.as_str(),
                rel: None,
            }],
        }),
    );

    Ok(HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_XML))
        .body(to_string(&models::Feed {
            xmlns: XMLNS_ATOM,
            id: lib.acquisition_feed_id().to_string(),
            title: FEED_TITLE.to_string(),
            subtitle: Some(format!("Exploring the \"{lib_name}\" library").to_string()),
            updated: lib.updated_at(),
            authors: vec![FEED_AUTHOR],
            links: vec![models::Link::start()],
            entries,
        })?))
}

#[derive(Deserialize)]
struct ExploreCatalogQuery {
    #[serde(rename = "sort")]
    order_by: Option<OrderBooksBy>,
    offset: Option<usize>,
    limit: Option<NonZeroUsize>,
}

#[get("/{lib_name}/explore")]
async fn explore_catalog(
    query: web::Query<ExploreCatalogQuery>,
    libraries: web::Data<Libraries>,
    lib_name: web::Path<String>,
) -> crate::Result<impl Responder> {
    let Some(lib) = libraries.get(&lib_name) else {
        return Err(AppError::LibraryNotFound);
    };

    let order_by = query.order_by.unwrap_or(OrderBooksBy::DateAdded);
    let offset = query.offset.unwrap_or(0);
    let limit = query
        .limit
        .unwrap_or(Library::DEFAULT_PAGE_SIZE)
        .clamp(Library::MIN_PAGE_SIZE, Library::MAX_PAGE_SIZE);

    let ((entries, lib_name), has_next_page) = lib
        .fetch_books(
            limit,
            offset,
            order_by,
            (vec![], lib_name.into_inner()),
            move |(mut acc, lib_name), book| {
                acc.push(models::Entry {
                    id: book.uri(),
                    title: book.title,
                    updated: book.last_modified_at,
                    authors: book
                        .authors
                        .into_iter()
                        .map(|author| models::Author {
                            name: Cow::Owned(author),
                            uri: None,
                        })
                        .collect(),
                    categories: book
                        .tags
                        .into_iter()
                        .map(|tag| models::Category { term: tag })
                        .collect(),
                    content: book.content.map(|content| models::Content {
                        kind: models::ContentKind::Html,
                        value: content,
                    }),
                    links: book
                        .data
                        .into_iter()
                        .map(|data| models::Link {
                            rel: Some(models::LinkRel::Acquisition.as_str()),
                            href: Cow::Owned(format!(
                                "{}/{lib_name}/{}/{}.{}",
                                super::lib_content::COMMON_ROUTE,
                                book.path,
                                data.file_name,
                                data.format
                            )),
                            kind: mime_guess::from_ext(&data.format)
                                .first_raw()
                                .unwrap_or("*/*"),
                        })
                        .chain(book.has_cover.then(|| models::Link {
                            rel: Some(models::LinkRel::Image.as_str()),
                            kind: mime::JPEG.as_str(),
                            href: Cow::Owned(format!(
                                "{}/{lib_name}/{}/cover.jpg",
                                super::lib_content::COMMON_ROUTE,
                                book.path,
                            )),
                        }))
                        .collect(),
                });

                (acc, lib_name)
            },
        )
        .await?;

    Ok(HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_XML))
        .body(to_string(&models::Feed {
            xmlns: XMLNS_ATOM,
            id: lib.acquisition_feed_id().to_string(),
            title: FEED_TITLE.to_string(),
            subtitle: Some(format!("Exploring the \"{lib_name}\" library").to_string()),
            updated: lib.updated_at(),
            authors: vec![FEED_AUTHOR],
            links: vec![models::Link::start()],
            entries,
        })?))
}
