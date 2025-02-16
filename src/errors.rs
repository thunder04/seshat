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
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        use AppError::*;

        match self {
            XmlSerialization(_) | Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
            LibraryNotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::plaintext())
            .body(self.to_string())
    }
}
