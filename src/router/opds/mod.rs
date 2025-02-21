mod links;
mod models;

use std::{borrow::Cow, num::NonZeroUsize};

use actix_web::{HttpResponse, Responder, get, web};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    errors::AppError,
    library::{Libraries, Library, OrderBooksBy},
    utils::HttpResponseBuilderExt as _,
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
                    kind: models::LinkType::Acquisition.as_str(),
                    href: links::lib_root(lib),
                    rel: None,
                }],
            }
        })
        .collect();

    HttpResponse::Ok().xml(&models::Feed {
        xmlns: XMLNS_ATOM,
        id: "urn:seshat:root".to_string(),
        title: FEED_TITLE.to_string(),
        subtitle: Some("Explore all available libraries".to_string()),
        updated: updated_at,
        authors: vec![FEED_AUTHOR],
        links: vec![models::Link::start()],
        entries,
    })
}

#[get("/{lib_name}")]
async fn library_root(
    libraries: web::Data<Libraries>,
    lib_name: web::Path<String>,
) -> crate::Result<impl Responder> {
    let Some(lib) = libraries.get(&lib_name) else {
        return Err(AppError::LibraryNotFound);
    };

    HttpResponse::Ok().xml(&models::Feed {
        xmlns: XMLNS_ATOM,
        id: lib.acquisition_feed_id().to_string(),
        title: FEED_TITLE.to_string(),
        subtitle: Some(format!("Exploring the \"{lib_name}\" library").to_string()),
        updated: lib.updated_at(),
        authors: vec![FEED_AUTHOR],
        links: vec![models::Link::start()],
        entries: [
            models::LibraryRootEntry {
                description: "View books",
                title: "View Books",
                link_rel: None,
                sort_by: None,
            },
            models::LibraryRootEntry {
                link_rel: Some(models::LinkRel::SortNew),
                description: "View new books",
                sort_by: Some("date_added"),
                title: "View New Books",
            },
            models::LibraryRootEntry {
                description: "View books sorted by title",
                title: "View Books by Title",
                sort_by: Some("title"),
                link_rel: None,
            },
            models::LibraryRootEntry {
                description: "View books sorted by author",
                title: "View Books by Author",
                sort_by: Some("author"),
                link_rel: None,
            },
        ]
        .into_iter()
        .map(|e| (lib, e).into())
        .collect(),
    })
}

#[derive(Serialize, Deserialize)]
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
    let lib_len = lib.len().await?;

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
                let id = book.uri();
                let links = book
                    .data
                    .iter()
                    .map(|data| models::Link {
                        rel: Some(models::LinkRel::Acquisition.as_str()),
                        href: links::download_book(&lib_name, &book, data),
                        kind: mime_guess::from_ext(&data.format)
                            .first_raw()
                            .unwrap_or("*/*"),
                    })
                    .chain(book.has_cover.then(|| models::Link {
                        rel: Some(models::LinkRel::Image.as_str()),
                        href: links::book_cover(&lib_name, &book),
                        kind: mime::JPEG.as_str(),
                    }))
                    .collect();
                let authors = book
                    .authors
                    .into_iter()
                    .map(|author| models::Author {
                        name: Cow::Owned(author),
                        uri: None,
                    })
                    .collect();
                let categories = book
                    .tags
                    .into_iter()
                    .map(|tag| models::Category { term: tag })
                    .collect();

                acc.push(models::Entry {
                    title: book.title,
                    updated: book.last_modified_at,
                    content: book.content.map(|content| models::Content {
                        kind: models::ContentKind::Html,
                        value: content,
                    }),
                    categories,
                    authors,
                    links,
                    id,
                });

                (acc, lib_name)
            },
        )
        .await?;
    let mut links = vec![
        models::Link::start(),
        models::Link {
            kind: models::LinkType::Navigation.as_str(),
            rel: Some(models::LinkRel::First.as_str()),
            href: links::explore_lib_with_query(
                ExploreCatalogQuery {
                    order_by: Some(order_by),
                    limit: Some(limit),
                    offset: Some(0),
                },
                lib,
            ),
        },
    ];

    if lib_len > limit.get() {
        links.push(models::Link {
            kind: models::LinkType::Navigation.as_str(),
            rel: Some(models::LinkRel::Last.as_str()),
            href: links::explore_lib_with_query(
                ExploreCatalogQuery {
                    offset: Some(lib_len.saturating_sub(limit.get())),
                    order_by: Some(order_by),
                    limit: Some(limit),
                },
                lib,
            ),
        });
    }

    if offset > 0 {
        links.push(models::Link {
            kind: models::LinkType::Navigation.as_str(),
            rel: Some(models::LinkRel::Previous.as_str()),
            href: links::explore_lib_with_query(
                ExploreCatalogQuery {
                    offset: Some(offset.saturating_sub(limit.get())),
                    order_by: Some(order_by),
                    limit: Some(limit),
                },
                lib,
            ),
        });
    }

    if has_next_page {
        links.push(models::Link {
            kind: models::LinkType::Navigation.as_str(),
            rel: Some(models::LinkRel::Next.as_str()),
            href: links::explore_lib_with_query(
                ExploreCatalogQuery {
                    offset: Some(offset + limit.get()),
                    order_by: Some(order_by),
                    limit: Some(limit),
                },
                lib,
            ),
        });
    }

    HttpResponse::Ok().xml(&models::Feed {
        xmlns: XMLNS_ATOM,
        id: lib.acquisition_feed_id().to_string(),
        title: FEED_TITLE.to_string(),
        subtitle: Some(format!("Exploring the \"{lib_name}\" library")),
        updated: lib.updated_at(),
        authors: vec![FEED_AUTHOR],
        entries,
        links,
    })
}
