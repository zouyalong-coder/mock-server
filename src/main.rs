use std::{future::Future, pin::Pin};

use actix_files::Files;
use actix_web::{
    http::header::HeaderMap,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer, Responder,
};
use anyhow::Result;
use clap::Parser;
use cli::Cli;
use log::info;
use reqwest::{Method, StatusCode};

// #[feature(async_closure)]
// #[macro_use]
// extern crate log;
extern crate pretty_env_logger;
// extern crate log;

mod cli;
mod config;
mod handlers;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let args = Cli::parse();
    let wd = args.work_dir.clone();
    let config_file = format!("{}/config.yaml", wd);
    info!("load config file: {}", config_file);
    let conf = match config::Config::load_from_file(&config_file) {
        Ok(conf) => conf,
        Err(e) => {
            info!("load config file failed: {}", e);
            config::Config::empty()
        }
    };
    info!("conf: {:?}", conf);
    info!("work_dir: {}", wd);
    info!("run server on port: {}", args.port);
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(Global { conf: conf.clone() }))
            .service(Files::new("/static", wd.as_str()).show_files_listing())
            .route("/mock{subpath:/.+}", actix_web::web::get().to(mock))
            .route("/mock{subpath:/.+}", actix_web::web::post().to(mock))
            .route("/mock{subpath:/.+}", actix_web::web::delete().to(mock))
            .route("/mock{subpath:/.+}", actix_web::web::put().to(mock))
            .route("/list_api", actix_web::web::get().to(handlers::list_api))
    })
    .bind(("0.0.0.0", args.port))?
    .run()
    .await
    .unwrap();
    Ok(())
}

struct Global {
    pub conf: config::Config,
}

async fn mock(request: HttpRequest, data: web::Data<Global>) -> HttpResponse {
    let path = request.match_info().get("subpath").unwrap_or("/");
    println!("path: {}", path);
    let hit_api = data.conf.find_api(path, &request.method());
    match hit_api {
        Some(api) => api.into(),
        None => HttpResponse::NotFound().body("not found"),
    }
}
