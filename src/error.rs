use crate::{domeseeing::DomeSeeingError, pressure::PressureError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error in the `pressure` module")]
    Pressure(#[from] PressureError),
    #[error("Error in the `domeseeing` module")]
    DomeSeeing(#[from] DomeSeeingError),
}

/*
fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

impl std::fmt::Debug for CfdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
*/
