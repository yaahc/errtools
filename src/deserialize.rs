use serde::Deserialize;

#[derive(Deserialize, Debug)]
///
pub struct Error {
    type_name: Option<String>,
    #[serde(flatten)]
    error: Box<InnerError>,
}

#[derive(Deserialize, Debug)]
struct InnerError {
    msg: String,
    source: Option<Box<InnerError>>,
}
