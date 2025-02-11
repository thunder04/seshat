use actix_web::web;

mod opds;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/opds").configure(opds::config));
}
