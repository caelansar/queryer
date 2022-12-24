#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Filetype {
    Unknown = 0,
    Csv = 1,
    Json = 2,
}

pub(crate) fn get_data_filetype(tp: Option<&str>) -> Filetype {
    match tp.unwrap_or("").to_lowercase().as_str() {
        "csv" => Filetype::Csv,
        "json" => Filetype::Json,
        _ => Filetype::Unknown,
    }
}
