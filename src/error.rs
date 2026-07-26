use crate::planar_geo;
use compare_variables::Comparison;
use planar_geo::{contour::Contour, error::ShapeConstructorError};
use stem_magnet::prelude::stem_material::si::Length;

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
    GeometryError(planar_geo::error::Error),
    /// A [`AirGap`](crate::air_gap::AirGap) or
    /// [`FluxBarrier`](crate::flux_barrier::FluxBarrier) is not compatible to
    /// a linear core. The string holds the type name (e.g. "PlainAirGap").
    IncompatibleToLinCore(&'static str),
    /// A [`AirGap`](crate::air_gap::AirGap) or
    /// [`FluxBarrier`](crate::flux_barrier::FluxBarrier) is not compatible to
    /// a rotary core. The string holds the type name (e.g. "PlainAirGap").
    IncompatibleToRotCore(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
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

#[derive(Debug)]
pub struct IncompatibleFluxBarrier {
    pub flux_barrier: Box<dyn crate::flux_barrier::FluxBarrier>,
    pub cause: Error,
}

impl std::fmt::Display for IncompatibleFluxBarrier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Flux barrier incompatible to core: ")?;
        self.cause.fmt(f)
    }
}

impl std::error::Error for IncompatibleFluxBarrier {}
