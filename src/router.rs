use actix_web::web;

mod opds;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope(opds::COMMON_ROUTE).configure(opds::configure));
}
