mod constants;
mod types;

use std::borrow::Cow;

use actix_web::{HttpResponse, Responder, get, http::header, web};
use constants::{FEED_AUTHOR, FEED_TITLE, LinkRel, LinkType, XMLNS_ATOM};
use quick_xml::se::to_string;
use serde::Deserialize;
use time::OffsetDateTime;

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
                kind: LinkType::AcquisitionFeed.as_str(),
                rel: None,
            },

            content: Some(types::Content {
                value: Cow::Owned(format!("Explore \"{name}\"")),
                kind: types::ContentKind::Text,
            }),
        })
        .collect::<Vec<_>>();
    let feed = types::AcquisitionFeed {
        xmlns: XMLNS_ATOM,

        // TODO: What should I change this to? Perhaps to a hash of modified at dates of all
        // libraries?
        id: "urn:uuid:2853dacf-ed79-42f5-8e8a-a7bb3d1ae6a2".into(),

        subtitle: Some("Explore available libraries".into()),
        author: FEED_AUTHOR,
        title: FEED_TITLE,

        // TODO: Set to Modified at time of the newest metadata.db
        updated: OffsetDateTime::now_utc(),

        links: vec![types::Link {
            kind: LinkType::AcquisitionFeed.as_str(),
            rel: Some(LinkRel::Start.as_str()),
            href: "/opds/".into(),
        }],

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

    let feed = types::AcquisitionFeed {
        xmlns: XMLNS_ATOM,

        // TODO: What should I change this to? Perhaps to a hash of modified at date of metadata.db?
        id: "urn:uuid:2853dacf-ed79-42f5-8e8a-a7bb3d1ae6a2".into(),

        title: FEED_TITLE,
        subtitle: Some(Cow::Owned(format!(
            "Explore \"{lib_name}\"'{possesive} catalog",
            possesive = if lib_name.ends_with(['s', 'S']) {
                ""
            } else {
                "s"
            }
        ))),
        author: FEED_AUTHOR,

        // TODO: Set to Modified at time of metadata.db.
        updated: OffsetDateTime::now_utc(),

        links: vec![types::Link {
            href: Cow::Owned(format!("/opds/{lib_path}/")),
            kind: LinkType::AcquisitionFeed.as_str(),
            rel: Some(LinkRel::Start.as_str()),
        }],

        entries: vec![
            types::Entry {
                title: "By Title".into(),
                link: types::Link {
                    href: Cow::Owned(format!("/opds/{lib_path}/explore?sort=title")),
                    kind: LinkType::AcquisitionFeed.as_str(),
                    rel: None,
                },

                content: Some(types::Content {
                    kind: types::ContentKind::Text,
                    value: "View books sorted by title".into(),
                }),
            },
            types::Entry {
                title: "View Books".into(),
                content: None,
                link: types::Link {
                    href: Cow::Owned(format!("/opds/{lib_path}/explore")),
                    kind: LinkType::AcquisitionFeed.as_str(),
                    rel: None,
                },
            },
        ],
    };

    HttpResponse::Ok()
        .insert_header(header::ContentType(mime::TEXT_XML))
        .body(to_string(&feed).expect("serialization failed"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ExploreCatalogSortType {
    Title,
}

#[derive(Deserialize)]
struct ExploreCatalogParams {
    sort: Option<ExploreCatalogSortType>,
}

#[get("/{lib_path}/explore/")]
async fn explore_catalog(
    lib_path: web::Path<String>, query: web::Query<ExploreCatalogParams>,
) -> impl Responder {
    let ExploreCatalogParams { sort } = query.into_inner();

    info!("{sort:?}");

    HttpResponse::NoContent()
}
