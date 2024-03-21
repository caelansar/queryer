use anyhow::{anyhow, Context, Result};
use polars::prelude::*;
use sqlparser::parser::Parser;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};
use tracing::info;

use crate::{
    ast_convert::Sql, dialect::SqlDialect, fetcher::retrieve_data, transformer::detect_content,
};

mod ast_convert;
mod dialect;
pub mod fetcher;
pub mod filetype;
mod transformer;

#[derive(Debug)]
pub struct DataSet(DataFrame);

impl Display for DataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for DataSet {
    type Target = DataFrame;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DataSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DataSet {
    pub fn to_csv(&mut self) -> Result<String> {
        let mut buf = Vec::new();
        let mut writer = CsvWriter::new(&mut buf);
        writer.finish(&mut self.0)?;
        Ok(String::from_utf8(buf)?)
    }

    pub fn to_json(&mut self) -> Result<String> {
        let mut buf = Vec::new();
        let writer = JsonWriter::new(&mut buf);
        writer
            .with_json_format(JsonFormat::JsonLines)
            .finish(&mut self.0)?;
        let s = String::from_utf8(buf)?;

        let tmp: Vec<&str> = s.lines().collect();
        let mut s = String::new();
        s.push('[');
        s.push_str(tmp.join(",").as_str());
        s.push(']');
        Ok(s)
    }
}

pub async fn query(sql: impl AsRef<str>) -> Result<DataSet> {
    let ast = Parser::parse_sql(&SqlDialect, sql.as_ref())?;

    if ast.len() != 1 {
        return Err(anyhow!("only support single sql"));
    }

    let sql = &ast[0];

    let Sql {
        source,
        condition,
        selection,
        offset,
        limit,
        order_by,
    } = sql.try_into()?;

    info!("retrieving data from source: {}", source);

    let ds = detect_content(
        retrieve_data(source)
            .await
            .context("failed to retrieve data")?,
    )?
    .transform()?;

    let mut filtered = match condition {
        Some(expr) => ds.0.lazy().filter(expr),
        None => ds.0.lazy(),
    };

    filtered = order_by.into_iter().fold(filtered, |acc, (col, desc)| {
        acc.sort(
            &col,
            SortOptions {
                descending: desc,
                nulls_last: false,
            },
        )
    });

    if offset.is_some() || limit.is_some() {
        filtered = filtered.slice(offset.unwrap_or(0), limit.unwrap_or(usize::MAX) as u32);
    }

    Ok(DataSet(filtered.select(selection).collect()?))
}
