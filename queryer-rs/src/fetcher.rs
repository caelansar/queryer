use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::filetype;
use anyhow::{anyhow, Result};
use tokio::fs;

pub trait Fetch {
    type Error;
    async fn fetch(&self) -> Result<(filetype::Filetype, String), Self::Error>;
}

pub async fn retrieve_data(source: impl AsRef<str>) -> Result<(filetype::Filetype, String)> {
    let source = source.as_ref();

    let (protocol, name) = source
        .split_once("://")
        .ok_or(anyhow!("protocol is not specified in source"))?;

    match protocol {
        "file" => FileFetcher(PathBuf::from(name).as_path()).fetch().await,
        "http" | "https" => HttpFetcher(source).fetch().await,
        _ => todo!(),
    }
}

struct HttpFetcher<'a>(&'a str);
struct FileFetcher<'a>(&'a Path);

impl<'a> Fetch for HttpFetcher<'a> {
    type Error = anyhow::Error;

    async fn fetch(&self) -> Result<(filetype::Filetype, String), Self::Error> {
        let resp = reqwest::get(self.0).await?;
        // 1. try to get filetype from content-type header
        let content_type = resp
            .headers()
            .get("Content-Type")
            .and_then(|x| x.to_str().ok().map(|s| s.split('/').last()))
            .flatten();
        let file_type = filetype::get_data_filetype(content_type);
        if file_type != filetype::Filetype::Unknown {
            return Ok((file_type, resp.text().await?));
        }

        // 2. try to get filetype from url
        let last_part = self.0.split('/').last();

        let file_type = filetype::get_data_filetype(last_part.and_then(|x| x.split('.').last()));

        Ok((file_type, resp.text().await?))
    }
}

impl<'a> Fetch for FileFetcher<'a> {
    type Error = anyhow::Error;

    async fn fetch(&self) -> Result<(filetype::Filetype, String), Self::Error> {
        let ext = self.0.extension().and_then(OsStr::to_str);
        let file_type = filetype::get_data_filetype(ext);

        Ok((file_type, fs::read_to_string(self.0).await?))
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
