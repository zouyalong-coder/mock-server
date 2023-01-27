use actix_web::HttpResponse;
use anyhow::Result;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use std::collections::HashMap;

use reqwest::{header::HeaderValue, Method, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Deserialize_enum_str, Serialize_enum_str, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    /// default.
    JSON,
    FORM,
    // MULTIPART,
    RAW,
}

impl Default for ContentType {
    fn default() -> Self {
        Self::JSON
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct API {
    pub path: String,
    #[serde(skip, default)]
    path_pattern: Option<regex::Regex>,
    #[serde(with = "http_serde::method", default)]
    pub method: Method,
    #[serde(default)]
    pub content_type: ContentType,
    #[serde(default = "default_status")]
    pub status: u16,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

fn default_status() -> u16 {
    200
}

impl API {
    pub fn match_path(&self, path: &str) -> bool {
        self.path_pattern.as_ref().unwrap().is_match(path)
    }

    fn amend(&mut self) -> Result<()> {
        self.path_pattern = Some(regex::Regex::new(&self.path)?);
        Ok(())
    }
}

impl Into<HttpResponse> for &API {
    fn into(self) -> HttpResponse {
        let mut builder = HttpResponse::build(StatusCode::from_u16(self.status).unwrap());
        for (k, v) in self.headers.iter() {
            builder.append_header((k.as_str(), v.as_str()));
        }
        if let Some(body) = self.body.as_ref() {
            match self.content_type {
                ContentType::RAW => builder.body(body.to_string()),
                ContentType::FORM => {
                    let body = serde_qs::to_string(body).unwrap();
                    builder.content_type(
                        HeaderValue::from_str("application/x-www-form-urlencoded").unwrap(),
                    );
                    builder.body(body)
                }
                _ => builder.json(body),
            }
        } else {
            builder.finish()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub apis: Vec<API>,
}

impl Config {
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let mut conf: Self = serde_yaml::from_str(yaml).map_err(|e| {
            let e: anyhow::Error = e.into();
            e
        })?;
        for api in conf.apis.iter_mut() {
            api.amend()?;
        }
        conf.validate()?;
        Ok(conf)
    }

    pub fn load_from_file(path: &str) -> Result<Self> {
        let yaml = std::fs::read_to_string(path)?;
        Self::from_yaml(&yaml)
    }

    pub fn empty() -> Self {
        Self { apis: vec![] }
    }

    fn validate(&self) -> Result<()> {
        for api in &self.apis {
            if api.path.is_empty() {
                return Err(anyhow::anyhow!("path is empty"));
            }
            // if api.method == Method::GET && api.body.is_some() {
            //     return Err(anyhow::anyhow!("GET method can't have body"));
            // }
        }
        Ok(())
    }

    pub fn find_api(&self, path: &str, method: &Method) -> Option<&API> {
        self.apis
            .iter()
            .find(|api| api.method == *method && api.match_path(path))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::config::ContentType;

    use super::{Config, API};

    #[test]
    fn match_reg() {
        let mut api = API {
            path: r"/api/test/\d+$".to_string(),
            content_type: crate::config::ContentType::FORM,
            path_pattern: None,
            method: "GET".parse().unwrap(),
            status: 200,
            headers: HashMap::new(),
            body: None,
        };
        api.amend().unwrap();
        assert!(api.match_path("/api/test/123"));
        println!("{}", serde_json::to_string_pretty(&api).unwrap());

        assert!(false);
    }

    #[test]
    fn from_yaml() {
        let yaml = r#"
        apis:
            -   path: /api/test/\d+$`                       
                method: POST
        "#;
        let conf = Config::from_yaml(yaml).unwrap();
        assert_eq!(conf.apis.len(), 1);
        assert_eq!(conf.apis[0].content_type, ContentType::JSON);
    }
}
