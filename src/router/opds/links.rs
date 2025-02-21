use compact_str::{CompactString, format_compact};
use percent_encoding::{NON_ALPHANUMERIC, percent_encode};

use super::{
    super::lib_content::COMMON_ROUTE as LIB_CONTENT_ROOT, COMMON_ROUTE as OPDS_ROOT,
    ExploreCatalogQuery,
};
use crate::library::{Data, FullBook, Library};

#[inline(always)]
fn enc(s: &str) -> percent_encoding::PercentEncode<'_> {
    percent_encode(s.as_bytes(), NON_ALPHANUMERIC)
}

pub fn lib_root(lib: &Library) -> CompactString {
    format_compact!("{OPDS_ROOT}/{}", enc(lib.name()))
}

pub fn explore_lib(lib: &Library, order_by: Option<&str>) -> CompactString {
    let mut link = format_compact!("{OPDS_ROOT}/{}/explore", enc(lib.name()));

    if let Some(order_by) = order_by {
        link += "?sort=";
        link += order_by;
    }

    link
}

pub fn explore_lib_with_query(query: ExploreCatalogQuery, lib: &Library) -> CompactString {
    let mut link = format_compact!("{OPDS_ROOT}/{}/explore", enc(lib.name()));

    if query.limit.is_some() || query.offset.is_some() || query.order_by.is_some() {
        let query = serde_urlencoded::ser::to_string(&query).expect("failed to serialize query");

        link.push('?');
        link.push_str(&query);
    }

    link
}

pub fn download_book(lib_name: &str, book: &FullBook, data: &Data) -> CompactString {
    format_compact!(
        "{LIB_CONTENT_ROOT}/{lib_name}/{path}/{file_name}.{file_format}",
        file_name = enc(&data.file_name),
        file_format = enc(&data.format),
        lib_name = enc(lib_name),
        path = enc(&book.path),
    )
}

pub fn book_cover(lib_name: &str, book: &FullBook) -> CompactString {
    format_compact!(
        "{LIB_CONTENT_ROOT}/{lib_name}/{path}/cover.jpg",
        lib_name = enc(lib_name),
        path = enc(&book.path),
    )
}
