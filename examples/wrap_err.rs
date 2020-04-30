//! An example of usage of the `WrapErr` trait with a public enum
//!
//! To use `WrapErr` with your type you have to impl from for (E: Error, String), this lets you
//! wrap arbitrary errors and add a message to them, it will then construct your type using this
//! from impl to conveniently create a new type
use errtools::{ErrTools, WrapErr};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
enum PublicEnumError {
    #[error("{msg}")]
    Variant1 {
        source: Box<dyn Error + Send + Sync + 'static>,
        msg: String,
    },
}

impl<E> From<(E, String)> for PublicEnumError
where
    E: Error + Send + Sync + 'static,
{
    fn from((source, msg): (E, String)) -> Self {
        let source = Box::new(source);
        PublicEnumError::Variant1 { source, msg }
    }
}

fn report_error(error: &(dyn Error)) {
    let mut cur_error = Some(error);
    let mut ind = 0;

    while let Some(error) = cur_error {
        println!("{}: {}", ind, error);
        ind += 1;
        cur_error = error.source();
    }
}

fn main() {
    let path = "fake_file";
    let error: PublicEnumError = std::fs::read_to_string(path)
        .wrap_err_with::<_, _, PublicEnumError>(|| {
            format!("unable to read file from path: {}", path)
        })
        .wrap_err("total failure!")
        .unwrap_err();

    report_error(&error.wrap_err::<_, PublicEnumError>("one more thing"));
}
