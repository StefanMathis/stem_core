/*!
This module contains the [`Error`] enum, which represents the different ways
building a magnetic core can fail due to invalid input data. The
[`Error::Other`] variants supports arbitrary errors resulting from user-created
core types. The [`IncompatibleFluxBarrier`] covers failure when trying to
insert a flux barrier in an existing core.
*/

use std::num::TryFromIntError;

use crate::{flux_barrier::FluxBarrier, planar_geo};
use compare_variables::Comparison;
use planar_geo::{contour::Contour, error::ShapeConstructorError};
use stem_magnet::prelude::stem_material::si::Length;

/// An enum representing errors resulting from attempting to build a
/// [`LinCore`](crate::core::LinCore) or [`RotCore`](crate::core::RotCore) from
/// invalid inputs.
#[derive(Debug)]
pub enum Error {
    /**
    A given physical [`Length`] is not within its allowed value range (as
    specified inside the [`Comparison`], usually a length needs to be
    positive).
     */
    InvalidLength(Comparison<Length>),
    /// A given [`usize`] is not within its allowed value range.
    InvalidUsize(Comparison<usize>),
    /// A given [`f64`] is not within its allowed value range.
    InvalidF64(Comparison<f64>),
    /// A given unsized integer such as [`usize`] is zero, but shouldn't be.
    ZeroUInt,
    /// Failed to create a core geometry due to the contained error.
    GeometryError(planar_geo::error::Error),
    /// A [`AirGap`](crate::air_gap::AirGap) or
    /// [`FluxBarrier`] is not compatible to a linear core. The string holds the
    /// type name (e.g. "PlainAirGap").
    IncompatibleToLinCore(&'static str),
    /// A [`AirGap`](crate::air_gap::AirGap) or
    /// [`FluxBarrier`] is not compatible to a rotary core. The string holds the
    /// type name (e.g. "PlainAirGap").
    IncompatibleToRotCore(&'static str),
    /// Fallback variant for arbitrary other errors (e.g. from custom
    /// [`AirGap`](crate::air_gap::AirGap) or [`FluxBarrier`] implementations).
    Other(Box<dyn std::error::Error>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLength(comparison_error) => comparison_error.fmt(f),
            Error::InvalidUsize(comparison_error) => comparison_error.fmt(f),
            Error::InvalidF64(comparison_error) => comparison_error.fmt(f),
            Error::ZeroUInt => write!(f, "value is zero, but shouldn't be"),
            Error::GeometryError(err) => err.fmt(f),
            Error::IncompatibleToLinCore(type_name) => {
                write!(f, "{} is not compatible to a linear core", type_name)
            }
            Error::IncompatibleToRotCore(type_name) => {
                write!(f, "{} is not compatible to a rotary core", type_name)
            }
            Error::Other(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<Comparison<Length>> for Error {
    fn from(value: Comparison<Length>) -> Self {
        return Error::InvalidLength(value);
    }
}

impl From<Comparison<usize>> for Error {
    fn from(value: Comparison<usize>) -> Self {
        return Error::InvalidUsize(value);
    }
}

impl From<Comparison<f64>> for Error {
    fn from(value: Comparison<f64>) -> Self {
        return Error::InvalidF64(value);
    }
}

impl From<ShapeConstructorError<Vec<Contour>>> for Error {
    fn from(value: ShapeConstructorError<Vec<Contour>>) -> Self {
        return planar_geo::error::Error::from(value).into();
    }
}

impl From<ShapeConstructorError<Contour>> for Error {
    fn from(value: ShapeConstructorError<Contour>) -> Self {
        return planar_geo::error::Error::from(value).into();
    }
}

impl From<planar_geo::error::Error> for Error {
    fn from(value: planar_geo::error::Error) -> Self {
        return Error::GeometryError(value);
    }
}

impl From<TryFromIntError> for Error {
    fn from(_: TryFromIntError) -> Self {
        return Error::ZeroUInt;
    }
}

/**
An error representing an incompatibility between a [`FluxBarrier`] and a core.

The [`Core`](crate::core::Core), [`RotCore`](crate::core::RotCore) and
[`LinCore`](crate::core::LinCore) types offer a method `set_flux_barrier` to
change the flux barrier after creation of the core. Should the barrier not be
compatible with the core, this error is returned, containing both the provided
flux barrier and the reason why setting a new flux barrier failed. This error
can only be created if the argument to `set_flux_barrier` was `Some`, since
removing a flux barrier with `core.set_flux_barrier(None)` cannot fail.
*/
#[derive(Debug)]
pub struct IncompatibleFluxBarrier {
    /// The incompatible [`FluxBarrier`] which
    /// was used as an argument to the set method.
    pub flux_barrier: Box<dyn FluxBarrier>,
    /// The underlying root cause why the flux barrier is incompatible to the
    /// core.
    pub cause: Error,
}

impl std::fmt::Display for IncompatibleFluxBarrier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Flux barrier incompatible to core: ")?;
        self.cause.fmt(f)
    }
}

impl std::error::Error for IncompatibleFluxBarrier {}
