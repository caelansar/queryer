use anyhow::{anyhow, Result};
use polars::prelude::*;
use std::io::Cursor;

use crate::{filetype, DataSet};

pub trait Transform {
    type Error;
    fn transform(self) -> Result<DataSet, Self::Error>;
}

#[derive(Debug)]
pub enum Transformer {
    Csv(CsvTransformer),
    Json(JsonTransformer),
}

#[derive(Default, Debug)]
pub struct CsvTransformer(pub(crate) String);

#[derive(Default, Debug)]
pub struct JsonTransformer(pub(crate) String);

impl Transformer {
    pub fn transform(self) -> Result<DataSet> {
        match self {
            Transformer::Csv(csv) => csv.transform(),
            Transformer::Json(json) => json.transform(),
        }
    }
}

pub fn detect_content(tup: (filetype::Filetype, String)) -> Result<Transformer> {
    match tup.0 {
        filetype::Filetype::Csv => Ok(Transformer::Csv(CsvTransformer(tup.1))),
        filetype::Filetype::Json => Ok(Transformer::Json(JsonTransformer(tup.1))),
        _ => Err(anyhow!("not support filetype")),
    }
}

impl Transform for CsvTransformer {
    type Error = anyhow::Error;

    fn transform(self) -> Result<DataSet, Self::Error> {
        let df = CsvReader::new(Cursor::new(self.0))
            .infer_schema(Some(16))
            .finish()?;
        Ok(DataSet(df))
    }
}

impl Transform for JsonTransformer {
    type Error = anyhow::Error;

    fn transform(self) -> Result<DataSet, Self::Error> {
        let df = JsonReader::new(Cursor::new(self.0))
            .infer_schema_len(Some(4))
            .finish()?;
        Ok(DataSet(df))
    }
}

#[cfg(test)]
mod tests {
    use crate::filetype;

    use super::detect_content;

    #[test]
    fn detect_content_should_work() {
        let json_data = r#"
        [
            {"a":1, "b":2.0, "c":false, "d":"4"},
            {"a":-10, "b":-3.5, "c":true, "d":"4"},
            {"a":2, "b":0.6, "c":false, "d":"text"},
            {"a":1, "b":2.0, "c":false, "d":"4"},
            {"a":7, "b":-3.5, "c":true, "d":"4"},
            {"a":1, "b":0.6, "c":false, "d":"text"}
        ]
        "#;

        let transform = detect_content((filetype::Filetype::Json, json_data.to_string())).unwrap();
        assert!(transform.transform().is_ok());
    }
}
