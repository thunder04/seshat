#![allow(clippy::literal_string_with_formatting_args, reason = "False positive")]

use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::{Responder, get, web};
use serde::Deserialize;
use tokio::fs;

use crate::{errors::AppError, library::Libraries};

pub const COMMON_ROUTE: &str = "/lib-content";

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(file_handler);
}

#[derive(Deserialize)]
struct FileHandlerPath {
    file_path: PathBuf,
    lib_name: String,
}

#[get("/{lib_name}/{file_path:.*}")]
async fn file_handler(
    path: web::Path<FileHandlerPath>,
    libraries: web::Data<Libraries>,
) -> crate::Result<impl Responder> {
    let Some(lib) = libraries.get(&path.lib_name) else {
        return Err(AppError::LibraryNotFound);
    };

    let file_path = lib.root_path().join(&path.file_path);
    let real_path = fs::canonicalize(file_path).await?;

    let Ok(relative_real_path) = real_path.strip_prefix(lib.root_path()) else {
        // Client attempted to access a file outside of the library's root path.
        return Err(AppError::file_not_found());
    };

    if relative_real_path.components().nth(1).is_none() {
        // Calibre library books are organized in folders.
        // The root path contains metadata which we don't want to share.
        return Err(AppError::file_not_found());
    }

    if fs::metadata(&real_path).await?.is_dir() {
        // Directories aren't supported.
        return Err(AppError::file_not_found());
    }

    Ok(tokio::task::spawn_blocking(move || NamedFile::open(real_path)).await??)
}
