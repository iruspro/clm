use std::error::Error;
use std::fmt;

/// An error from constructing a [`Magnitude`](super::Magnitude).
#[derive(Debug)]
pub enum MagnitudeError {
    /// The amount was zero or negative; a magnitude must be strictly positive.
    NonPositive,
}

impl fmt::Display for MagnitudeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MagnitudeError::NonPositive => write!(f, "magnitude must be positive"),
        }
    }
}

impl Error for MagnitudeError {}
