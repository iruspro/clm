use thiserror::Error;

/// An error from constructing a [`Magnitude`](super::Magnitude).
#[derive(Error, Debug)]
pub enum MagnitudeError {
    /// The amount was zero or negative; a magnitude must be strictly positive.
    #[error("magnitude must be positive")]
    NonPositive,
}
