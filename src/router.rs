use actix_web::web;

mod lib_content;
mod opds;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope(lib_content::COMMON_ROUTE).configure(lib_content::configure))
        .service(web::scope(opds::COMMON_ROUTE).configure(opds::configure));
}
