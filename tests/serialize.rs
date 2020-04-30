use errtools::deserialize;
use errtools::ErrTools;
use std::error::Error;
use std::fmt;

#[test]
fn serialize_eyre() {
    use eyre::{eyre, ErrReport};
    let err: ErrReport = eyre!("root cause")
        .wrap_err("second error")
        .wrap_err("outermost error");
    let json = serde_json::to_string_pretty(&err.serialize()).unwrap();
    println!("{}", json);
}

#[test]
fn serialize_anyhow() {
    let err = anyhow::anyhow!("root cause")
        .context("second error")
        .context("outermost error");
    let json = serde_json::to_string_pretty(&err.serialize()).unwrap();
    println!("{}", json);
}

#[derive(Debug)]
struct RootError;

impl fmt::Display for RootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "root cause")
    }
}

impl std::error::Error for RootError {}

#[derive(Debug)]
struct SecondError(RootError);

impl fmt::Display for SecondError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "second error")
    }
}

impl std::error::Error for SecondError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

#[test]
fn serialize_concrete() {
    let err = SecondError(RootError);
    let json = serde_json::to_string_pretty(&err.serialize()).unwrap();
    println!("concrete serialization:\n{}\n", json);
    let err: &dyn Error = &err;
    let json = serde_json::to_string_pretty(&err.serialize()).unwrap();
    println!("dyn serialization:\n{}", json);
}

#[test]
fn deserialize_concrete() {
    let err = SecondError(RootError);
    let json = serde_json::to_string_pretty(&err.serialize()).unwrap();
    println!("concrete serialization:\n{}\n", json);

    let err_out: deserialize::Error = serde_json::from_str(&json).unwrap();
    println!("{:?}", err_out);
}
