use anyhow::Result;
use queryer_rs::{fetcher::Fetch, filetype, query};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // https fetcher
    let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";
    let sql = format!(
        "SELECT location name, total_cases, new_cases, total_deaths, new_deaths \
        FROM {} where new_deaths >= 600 ORDER BY new_cases DESC",
        url
    );
    let df = query(sql).await?;
    println!("{}", df);

    // file fetcher
    let url = "file://./queryer-rs/examples/data.json";
    let sql = format!("SELECT name FROM {} where age >= 18", url);

    let mut df = query(sql).await?;
    assert_eq!(
        "[{\"name\":\"bb\"},{\"name\":\"cc\"}]",
        df.to_json().unwrap()
    );

    Ok(())
}

struct CustomFetcher();

impl Fetch for CustomFetcher {
    type Error = anyhow::Error;

    async fn fetch(&self) -> Result<(filetype::Filetype, String), Self::Error> {
        Ok((
            filetype::Filetype::Json,
            r#"
                    [
                        {"name": "aa", "score": 88},
                        {"name": "bb", "score": 91}
                    ]
               "#
            .to_string(),
        ))
    }
}
