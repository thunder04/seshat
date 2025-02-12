mod types;

use std::borrow::Cow;

use actix_web::{HttpResponse, Responder, get, http::header, web};
use quick_xml::se::to_string;
use serde::Deserialize;
use time::OffsetDateTime;
use types as t;

use crate::library::Libraries;

pub const COMMON_ROUTE: &str = "/opds";

const XMLNS_ATOM: &str = "http://www.w3.org/2005/Atom";
const FEED_TITLE: Cow<'static, str> = Cow::Borrowed("Seshat – OPDS Catalog");
const FEED_AUTHOR: t::Author = t::Author {
    uri: Cow::Borrowed("https://github.com/thunder04"),
    name: Cow::Borrowed("Thunder04"),
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(root)
        .service(library_root)
        .service(explore_catalog);
}

#[get("/")]
async fn root(libraries: web::Data<Libraries>) -> impl Responder {
    let feed = t::AcquisitionFeed {
        xmlns: XMLNS_ATOM,

        // TODO: What should I change this to? Perhaps to a hash of modified at dates of all
        // libraries?
        id: "urn:uuid:2853dacf-ed79-42f5-8e8a-a7bb3d1ae6a2".into(),
        title: FEED_TITLE,
        subtitle: Some("Explore all available libraries".into()),
        author: FEED_AUTHOR,
        links: vec![t::Link::start()],

        // updated_at is set as the library with the most recent change.
        updated: libraries
            .all_libraries()
            .map(|lib| lib.modified_at())
            .max()
            .unwrap_or(OffsetDateTime::UNIX_EPOCH),

        entries: libraries
            .all_libraries()
            .map(|lib| t::Entry {
                title: lib.name().to_string().into(),
                link: t::Link {
                    href: format!("{COMMON_ROUTE}/{}", lib.name()).into(),
                    kind: t::LinkType::Acquisition.as_str(),
                    rel: None,
                },
                content: Some(t::Content {
                    value: format!("Explore the \"{}\" library", lib.name()).into(),
                    kind: t::ContentKind::Text,
                }),
            })
            .collect(),
    };

    HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_XML))
        .body(to_string(&feed).expect("serialization failed"))
}

#[get("/{lib_name}/")]
async fn library_root(
    lib_name: web::Path<String>, libraries: web::Data<Libraries>,
) -> impl Responder {
    let Some(lib) = libraries.get_library(&lib_name) else {
        return HttpResponse::NotFound().body("The library doesn't exist");
    };

    let mut entries = vec![t::Entry {
        title: "View Books".into(),
        content: None,
        link: t::Link {
            href: format!("{COMMON_ROUTE}/{lib_name}/explore").into(),
            kind: t::LinkType::Acquisition.as_str(),
            rel: None,
        },
    }];

    entries.extend(
        [
            ("By Date", "the date they were added", "date_added"),
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
        .map(|(title, sorted_by, sort)| t::Entry {
            title: title.to_string().into(),
            link: t::Link {
                href: format!("{COMMON_ROUTE}/{lib_name}/explore?sort={sort}").into(),
                kind: t::LinkType::Acquisition.as_str(),
                rel: None,
            },

            content: Some(t::Content {
                value: format!("View books sorted by {sorted_by}").into(),
                kind: t::ContentKind::Text,
            }),
        }),
    );

    let feed = t::AcquisitionFeed {
        xmlns: XMLNS_ATOM,

        // TODO: What should I change this to? Perhaps to a hash of modified at date of metadata.db?
        id: "urn:uuid:2853dacf-ed79-42f5-8e8a-a7bb3d1ae6a2".into(),
        title: FEED_TITLE,
        subtitle: Some(format!("Exploring the \"{lib_name}\" library").into()),
        author: FEED_AUTHOR,
        updated: lib.modified_at(),
        links: vec![t::Link::start()],
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

#[get("/{lib_name}/explore/")]
async fn explore_catalog(
    lib_name: web::Path<String>, query: web::Query<ExploreCatalogQuery>,
    libraries: web::Data<Libraries>,
) -> impl Responder {
    let Some(lib) = libraries.get_library(&lib_name) else {
        return HttpResponse::NotFound().body("The library doesn't exist");
    };

    let ExploreCatalogQuery {
        sort,
        offset,
        limit,
    } = query.into_inner();
    let limit = limit.unwrap_or(25).clamp(1, 50);
    let offset = offset.unwrap_or(0);

    let feed = t::AcquisitionFeed {
        xmlns: XMLNS_ATOM,

        // TODO: What should I change this to? Perhaps to a hash of modified at date of metadata.db?
        id: "urn:uuid:2853dacf-ed79-42f5-8e8a-a7bb3d1ae6a2".into(),
        title: FEED_TITLE,
        subtitle: None,
        author: FEED_AUTHOR,
        updated: lib.modified_at(),
        links: vec![t::Link::start()],

        entries: vec![],
    };

    HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_XML))
        .body(to_string(&feed).expect("serialization failed"))
}
