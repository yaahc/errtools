use errtools::WrapErr;
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
enum PrivateKind {
    #[error("{msg}")]
    Variant1 {
        source: Box<dyn Error + Send + Sync + 'static>,
        msg: String,
    },
}

#[derive(Error, Debug)]
#[error(transparent)]
struct PublicErrorStruct {
    #[from]
    source: PrivateKind,
}

impl<E> From<(E, String)> for PrivateKind
where
    E: Error + Send + Sync + 'static,
{
    fn from((source, msg): (E, String)) -> Self {
        let source = Box::new(source);
        PrivateKind::Variant1 { source, msg }
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

fn do_thing(path: &str) -> Result<String, PublicErrorStruct> {
    let s = std::fs::read_to_string(path)
        .wrap_err_with(|| format!("unable to read file from path: {}", path))?;

    Ok(s)
}

fn main() {
    let path = "fake_file";
    let error = do_thing(path).unwrap_err();
    report_error(&error);
}
