use std::{collections::HashMap, ffi::OsStr, path::Path, sync::Mutex};

use crate::filetype;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use lazy_static::lazy_static;
use tokio::fs;

#[async_trait]
pub trait Fetch {
    type Error;
    async fn fetch(&self, source: &str) -> Result<(filetype::Filetype, String), Self::Error>;
}

lazy_static! {
    static ref FETCHER_REGISTRY: Mutex<FetcherRegistry> = Mutex::new(FetcherRegistry::new());
}

pub struct FetcherRegistry {
    registry: HashMap<String, Box<dyn Fetch<Error = anyhow::Error> + Sync + Send>>,
}

impl FetcherRegistry {
    pub fn new() -> Self {
        let mut registry: HashMap<String, Box<dyn Fetch<Error = anyhow::Error> + Sync + Send>> =
            HashMap::new();
        registry.insert("http".to_string(), Box::new(HttpFetcher()));
        registry.insert("https".to_string(), Box::new(HttpFetcher()));
        registry.insert("file".to_string(), Box::new(FileFetcher()));
        Self { registry }
    }
}

pub fn register_fetcher<T: Fetch<Error = anyhow::Error> + Sync + Send + 'static>(
    protocol: impl AsRef<str>,
    fetcher: T,
) {
    FETCHER_REGISTRY
        .lock()
        .unwrap()
        .registry
        .insert(protocol.as_ref().into(), Box::new(fetcher));
}

pub async fn retrieve_data(source: impl AsRef<str>) -> Result<(filetype::Filetype, String)> {
    let name = source.as_ref();

    let protocol = name.split("://").next();

    if let Some(p) = protocol {
        let r = FETCHER_REGISTRY.lock().unwrap();
        let fetcher = r
            .registry
            .get(p)
            .ok_or(anyhow!("fetcher not found in for given protocol"))?;
        fetcher.fetch(source.as_ref()).await
    } else {
        Err(anyhow!("protocol is not specified in source"))
    }
}

struct HttpFetcher();
struct FileFetcher();

#[async_trait]
impl Fetch for HttpFetcher {
    type Error = anyhow::Error;

    async fn fetch(&self, source: &str) -> Result<(filetype::Filetype, String), Self::Error> {
        let resp = reqwest::get(source).await?;
        // 1. try to get filetype from cntent-type header
        let content_type = resp
            .headers()
            .get("Content-Type")
            .and_then(|x| x.to_str().ok().map(|s| s.split("/").last()))
            .flatten();
        let file_type = filetype::get_data_filetype(content_type);
        if file_type != filetype::Filetype::Unknown {
            return Ok((file_type, resp.text().await?));
        }

        // 2. try to get filetype from url
        let last_part = source.split("/").last();

        let file_type = filetype::get_data_filetype(last_part.and_then(|x| x.split(".").last()));

        Ok((file_type, resp.text().await?))
    }
}

#[async_trait]
impl Fetch for FileFetcher {
    type Error = anyhow::Error;

    async fn fetch(&self, source: &str) -> Result<(filetype::Filetype, String), Self::Error> {
        let ext = Path::new(&source[7..]).extension().and_then(OsStr::to_str);
        let file_type = filetype::get_data_filetype(ext);

        Ok((file_type, fs::read_to_string(&source[7..]).await?))
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::HeaderMap;

    use super::*;

    #[tokio::test]
    async fn retrieve_data_should_work() {
        let url = "file://./examples/data.json";
        let data = retrieve_data(url).await.unwrap();
        assert_eq!(filetype::Filetype::Json, data.0);
        println!("type {:?}, data {}", data.0, data.1);

        let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";
        let data = retrieve_data(url).await.unwrap();
        assert_eq!(filetype::Filetype::Csv, data.0);
        println!("type {:?}, data {}", data.0, data.1);
    }

    #[test]
    fn test_get_header_content_type() {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let content_type = headers
            .get("Content-Type")
            .and_then(|x| x.to_str().ok().map(|s| s.split("/").last()))
            .flatten();
        assert_eq!(Some("json"), content_type);
    }
}
