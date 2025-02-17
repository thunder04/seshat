use std::io;

use actix_web::{
    HttpResponse, ResponseError,
    http::{StatusCode, header::ContentType},
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("The library could not be found")]
    LibraryNotFound,

    #[cfg_attr(not(debug_assertions), error("Failed to serialize XML response"))]
    #[cfg_attr(debug_assertions, error("Failed to serialize XML response: {0}"))]
    XmlSerialization(#[from] quick_xml::SeError),

    #[cfg_attr(not(debug_assertions), error("Internal server error"))]
    #[cfg_attr(debug_assertions, error("rusqilite error: {0}"))]
    Db(#[from] async_sqlite::Error),

    #[cfg_attr(not(debug_assertions), error("{}", match .0.kind() {
        io::ErrorKind::NotFound | io::ErrorKind::NotADirectory => "File does not exist",
        _ => "Internal server error",
    }))]
    #[cfg_attr(debug_assertions, error("IO error: {0}"))]
    Io(#[from] io::Error),

    #[error("task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        use AppError::*;

        match self {
            LibraryNotFound => StatusCode::NOT_FOUND,
            Io(cause) => match cause.kind() {
                io::ErrorKind::NotFound | io::ErrorKind::NotADirectory => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },

            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::plaintext())
            .body(self.to_string())
    }
}

impl AppError {
    /// Creates a new IO NOT_FOUND error
    pub fn file_not_found() -> Self {
        Self::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"))
    }
}
