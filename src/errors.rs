use actix_web::{
    HttpResponse, ResponseError,
    http::{StatusCode, header::ContentType},
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[cfg_attr(not(debug_assertions), error("Failed to serialize XML response"))]
    #[cfg_attr(debug_assertions, error("Failed to serialize XML response: {0}"))]
    XmlSerialization(#[from] quick_xml::SeError),
    #[error("The library could not be found")]
    LibraryNotFound,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::XmlSerialization(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::LibraryNotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::plaintext())
            .body(self.to_string())
    }
}
