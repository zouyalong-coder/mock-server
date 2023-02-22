use std::env;

use actix_files::Files;
use actix_web::{
    web::{self, Bytes, Data},
    App, HttpRequest, HttpResponse, HttpServer,
};
use anyhow::Result;
use clap::Parser;
use cli::Cli;
use colored::Colorize;
use log::{info, warn};

use crate::tty::highlight_text;

// #[feature(async_closure)]
// #[macro_use]
// extern crate log;
extern crate pretty_env_logger;
// extern crate log;

mod cli;
mod config;
mod error;
mod handlers;
mod tty;

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    pretty_env_logger::init();
    let args = Cli::parse();
    let wd = args.work_dir.clone();
    let static_prefix = args.static_prefix.clone();
    let config_file = format!("{}/config.yaml", wd);
    let upload_file = args.up_file;
    let conf = match config::Config::load_from_file(&config_file) {
        Ok(conf) => conf,
        Err(e) => {
            warn!("load config file failed: {}", e);
            config::Config::empty()
        }
    };
    HttpServer::new(move || {
        let app = App::new()
            .app_data(Data::new(Global {
                conf: conf.clone(),
                verbose: args.verbose,
                work_dir: wd.clone(),
            }))
            .route("/mock{subpath:/.+}", actix_web::web::get().to(mock))
            .route("/mock{subpath:/.+}", actix_web::web::post().to(mock))
            .route("/mock{subpath:/.+}", actix_web::web::delete().to(mock))
            .route("/mock{subpath:/.+}", actix_web::web::put().to(mock))
            .route("/list_api", actix_web::web::get().to(handlers::list_api))
            .service(
                Files::new(format!("/{}", static_prefix.as_str()).as_str(), wd.as_str())
                    .show_files_listing(),
            );
        if upload_file {
            app.route(
                "/upload/{subpath:.+}",
                actix_web::web::put().to(handlers::upload_file),
            )
        } else {
            app
        }
    })
    .bind(("0.0.0.0", args.port))?
    .run()
    .await
    .unwrap();
    Ok(())
}

pub struct Global {
    pub conf: config::Config,
    pub verbose: bool,
    pub work_dir: String,
}

async fn mock(raw_body: Bytes, request: HttpRequest, data: web::Data<Global>) -> HttpResponse {
    let path = request.match_info().get("subpath").unwrap_or("/");
    if data.verbose {
        print_request(&request, raw_body.clone());
        info!("");
    }
    let hit_api = data.conf.find_api(path, &request.method());
    match hit_api {
        Some(api) => api.into(),
        None => HttpResponse::NotFound()
            .content_type("text/plain; charset=utf-8")
            .body(format!("{} not found, pls check it in config.yaml", path)),
    }
}

fn content_type(req: &HttpRequest) -> Option<&str> {
    req.headers()
        .get("content-type")
        .map(|ct| ct.to_str().unwrap())
}

fn print_request(req: &HttpRequest, raw_body: Bytes) {
    let method = req.method();
    let path = req.path();
    let query = req.query_string();
    info!("{} {}?{}", method.to_string().yellow(), path, query);
    req.headers().iter().for_each(|(key, value)| {
        info!(
            "{}: {}",
            key.to_string().green(),
            value.to_str().unwrap().white()
        );
    });
    match content_type(req) {
        Some("application/json") => {
            let body = String::from_utf8(raw_body.to_vec()).unwrap();
            highlight_text(body.as_str(), "json", None).unwrap();
        }
        Some("application/x-www-form-urlencoded") => {
            let body = String::from_utf8(raw_body.to_vec()).unwrap();
            info!("{}", body.white())
        }
        _ => {}
    }
}
