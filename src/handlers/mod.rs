use actix_web::{web, Responder};

use crate::Global;

// #[get("/list")]
pub async fn list_api(data: web::Data<Global>) -> impl Responder {
    let mut res = String::new();
    data.conf.apis.iter().for_each(|api| {
        res.push_str(&format!("{} /mock{}\n", api.method, api.path));
    });
    res
}
