use actix_multipart::Multipart;
use actix_web::{
    web::{self, Path},
    HttpResponse, Responder,
};
use anyhow::Result;
use futures::TryStreamExt;
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

use crate::Global;

// #[get("/list")]
pub async fn list_api(data: web::Data<Global>) -> impl Responder {
    let mut res = String::new();
    data.conf.apis.iter().for_each(|api| {
        res.push_str(&format!("{} /mock{}\n", api.method, api.path));
    });
    res
}

#[derive(Deserialize)]
pub struct FileUpload {
    // file: web::Payload,
    subpath: String,
}

// #[put("/upload/{subpath:.+}")]
pub async fn upload_file(
    data: web::Data<Global>,
    arg: Path<FileUpload>,
    content: Multipart,
) -> HttpResponse {
    let subpath = arg.into_inner().subpath;
    let path = format!("{}/{}", data.work_dir, subpath);
    match save_file(content, path.as_str()).await {
        Ok(_) => HttpResponse::Ok().body("ok"),
        Err(e) => HttpResponse::InternalServerError().body(format!("save file failed: {}", e)),
    }
}

async fn save_file(mut content: Multipart, path: &str) -> Result<()> {
    let path = std::path::Path::new(path);
    let dir = path.parent().unwrap();
    tokio::fs::create_dir_all(dir).await?;
    let mut file = tokio::fs::File::create(path).await?;
    while let Ok(Some(mut field)) = content.try_next().await {
        while let Some(chunk) = field.try_next().await? {
            file.write_all(&chunk).await?;
        }
    }
    Ok(())
}
