use crate::{
    cfd::CfdError, domeseeing::DomeSeeingError, monitors::MonitorsError, pressure::PressureError,
};

#[derive(thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Monitors(#[from] MonitorsError),
    #[error(transparent)]
    Pressure(#[from] PressureError),
    #[error(transparent)]
    DomeSeeing(#[from] DomeSeeingError),
    #[error(transparent)]
    CFD(#[from] CfdError),
    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error>),
}

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

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
