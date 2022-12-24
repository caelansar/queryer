use std::{collections::HashMap, ffi::OsStr, path::Path};

use anyhow::{anyhow, Result};
use tokio::fs;

use crate::filetype;

pub trait Fetch {
    type Error;
    async fn fetch(&self) -> Result<(filetype::Filetype, String), Self::Error>;
}

pub async fn retrieve_data(source: impl AsRef<str>) -> Result<(filetype::Filetype, String)> {
    let name = source.as_ref();
    match &name[..4] {
        "http" => HttpFetcher(name).fetch().await,
        "file" => FileFetcher(name).fetch().await,
        _ => Err(anyhow!("only support http/https/file")),
    }
}

struct HttpFetcher<'a>(pub(crate) &'a str);
struct FileFetcher<'a>(pub(crate) &'a str);

impl<'a> Fetch for HttpFetcher<'a> {
    type Error = anyhow::Error;

    async fn fetch(&self) -> Result<(filetype::Filetype, String), Self::Error> {
        let resp = reqwest::get(self.0).await?;
        let content_type = resp
            .headers()
            .get("Content-Type")
            .and_then(|x| x.to_str().ok().map(|s| s.split("/").last()))
            .flatten();
        let file_type = filetype::get_data_filetype(content_type);
        if file_type != filetype::Filetype::Unknown {
            return Ok((file_type, resp.text().await?));
        }

        let parts: Vec<&str> = self.0.split("/").collect();
        let last_part = parts[parts.len() - 1];

        let file_type = filetype::get_data_filetype(
            last_part
                .find(".")
                .and_then(|idx| Some(&last_part[idx + 1..])),
        );

        Ok((file_type, resp.text().await?))
    }
}

impl<'a> Fetch for FileFetcher<'a> {
    type Error = anyhow::Error;

    async fn fetch(&self) -> Result<(filetype::Filetype, String), Self::Error> {
        let ext = Path::new(&self.0[7..]).extension().and_then(OsStr::to_str);
        let file_type = filetype::get_data_filetype(ext);

        Ok((file_type, fs::read_to_string(&self.0[7..]).await?))
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
        println!("type {:?}, data {}", data.0, data.1);

        let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";
        let data = retrieve_data(url).await.unwrap();
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
        println!("{:?}", content_type);
    }
}
