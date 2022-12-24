use anyhow::Result;
use queryer::query;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";

    let sql = format!(
        "SELECT location name, total_cases, new_cases, total_deaths, new_deaths \
        FROM {} where new_deaths >= 600 ORDER BY new_cases DESC",
        url
    );
    let df = query(sql).await?;
    println!("{}", df);

    let url = "file://./examples/data.json";
    let sql = format!("SELECT name FROM {} where age >= 18", url);

    let df = query(sql).await?;
    println!("{}", df);

    Ok(())
}
