use std::sync::Arc;

use planar_geo::prelude::Shape;
use stem_magnet::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod ext;
pub mod lin;
pub mod rot;

pub use ext::CoreExt;
pub use lin::*;
pub use rot::*;

use crate::LinOrRot;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Core {
    Lin(LinCore),
    Rot(RotCore),
}

impl Core {
    pub fn lin<'core>(&'core self) -> Option<&'core LinCore> {
        match self {
            Self::Lin(c) => Some(c),
            Self::Rot(_) => None,
        }
    }

    pub fn rot<'core>(&'core self) -> Option<&'core RotCore> {
        match self {
            Self::Lin(_) => None,
            Self::Rot(c) => Some(c),
        }
    }

    pub fn set_flux_barrier(
        &mut self,
        flux_barrier: Option<Box<dyn crate::prelude::FluxBarrier>>,
    ) -> Result<(), crate::error::IncompatibleFluxBarrier> {
        match self {
            Self::Lin(c) => c.set_flux_barrier(flux_barrier),
            Self::Rot(c) => c.set_flux_barrier(flux_barrier),
        }
    }
}

impl ext::private::Sealed for Core {}

impl CoreExt for Core {
    fn air_gap(&self) -> &dyn crate::air_gap::AirGap {
        match self {
            Self::Lin(c) => c.air_gap(),
            Self::Rot(c) => c.air_gap(),
        }
    }

    fn axial_length(&self) -> Length {
        match self {
            Self::Lin(c) => c.axial_length(),
            Self::Rot(c) => c.axial_length(),
        }
    }

    fn iron_fill_factor(&self) -> f64 {
        match self {
            Self::Lin(c) => c.iron_fill_factor(),
            Self::Rot(c) => c.iron_fill_factor(),
        }
    }

    fn pole_pairs(&self) -> u16 {
        match self {
            Self::Lin(c) => c.pole_pairs(),
            Self::Rot(c) => c.pole_pairs(),
        }
    }

    fn material(&self) -> &Arc<Material> {
        match self {
            Self::Lin(c) => c.material(),
            Self::Rot(c) => c.material(),
        }
    }

    fn yoke_height(&self) -> Length {
        match self {
            Self::Lin(c) => c.yoke_height(),
            Self::Rot(c) => c.yoke_height(),
        }
    }

    fn skew_angle(&self) -> f64 {
        match self {
            Self::Lin(c) => c.skew_angle(),
            Self::Rot(c) => c.skew_angle(),
        }
    }

    fn air_gap_width(&self) -> Length {
        match self {
            Self::Lin(c) => c.air_gap_width(),
            Self::Rot(c) => c.air_gap_width(),
        }
    }

    fn lin_or_rot(&self) -> LinOrRot {
        match self {
            Self::Lin(c) => c.lin_or_rot(),
            Self::Rot(c) => c.lin_or_rot(),
        }
    }

    fn axial_coil_overhang(&self) -> Length {
        match self {
            Self::Lin(c) => c.axial_coil_overhang(),
            Self::Rot(c) => c.axial_coil_overhang(),
        }
    }

    fn flux_barrier(&self) -> Option<&dyn crate::flux_barrier::FluxBarrier> {
        match self {
            Self::Lin(c) => c.flux_barrier(),
            Self::Rot(c) => c.flux_barrier(),
        }
    }

    fn shape<'a>(&'a self) -> &'a Shape {
        match self {
            Self::Lin(c) => c.shape(),
            Self::Rot(c) => c.shape(),
        }
    }

    fn as_core_ref(&self) -> CoreRef<'_> {
        match self {
            Self::Lin(c) => c.as_core_ref(),
            Self::Rot(c) => c.as_core_ref(),
        }
    }

    fn pole_coverage(&self, surface_magnet_assembly: Option<&MagnetAssembly>) -> f64 {
        match self {
            Self::Lin(c) => c.pole_coverage(surface_magnet_assembly),
            Self::Rot(c) => c.pole_coverage(surface_magnet_assembly),
        }
    }
}

impl From<LinCore> for Core {
    fn from(value: LinCore) -> Self {
        Self::Lin(value)
    }
}

impl From<RotCore> for Core {
    fn from(value: RotCore) -> Self {
        Self::Rot(value)
    }
}

// =============================================================================

#[derive(Debug, Clone, Copy)]
pub enum CoreRef<'a> {
    Lin(&'a LinCore),
    Rot(&'a RotCore),
}

impl<'a> CoreRef<'a> {
    pub fn lin<'core>(&'core self) -> Option<&'core LinCore> {
        match self {
            CoreRef::Lin(c) => Some(*c),
            CoreRef::Rot(_) => None,
        }
    }

    pub fn rot<'core>(&'core self) -> Option<&'core RotCore> {
        match self {
            CoreRef::Lin(_) => None,
            CoreRef::Rot(c) => Some(*c),
        }
    }
}

impl<'a> ext::private::Sealed for CoreRef<'a> {}

impl<'a> CoreExt for CoreRef<'a> {
    fn air_gap(&self) -> &dyn crate::air_gap::AirGap {
        match self {
            Self::Lin(c) => c.air_gap(),
            Self::Rot(c) => c.air_gap(),
        }
    }

    fn axial_length(&self) -> Length {
        match self {
            Self::Lin(c) => c.axial_length(),
            Self::Rot(c) => c.axial_length(),
        }
    }

    fn iron_fill_factor(&self) -> f64 {
        match self {
            Self::Lin(c) => c.iron_fill_factor(),
            Self::Rot(c) => c.iron_fill_factor(),
        }
    }

    fn pole_pairs(&self) -> u16 {
        match self {
            Self::Lin(c) => c.pole_pairs(),
            Self::Rot(c) => c.pole_pairs(),
        }
    }

    fn material(&self) -> &Arc<Material> {
        match self {
            Self::Lin(c) => c.material(),
            Self::Rot(c) => c.material(),
        }
    }

    fn yoke_height(&self) -> Length {
        match self {
            Self::Lin(c) => c.yoke_height(),
            Self::Rot(c) => c.yoke_height(),
        }
    }

    fn skew_angle(&self) -> f64 {
        match self {
            Self::Lin(c) => c.skew_angle(),
            Self::Rot(c) => c.skew_angle(),
        }
    }

    fn air_gap_width(&self) -> Length {
        match self {
            Self::Lin(c) => c.air_gap_width(),
            Self::Rot(c) => c.air_gap_width(),
        }
    }

    fn lin_or_rot(&self) -> LinOrRot {
        match self {
            Self::Lin(c) => c.lin_or_rot(),
            Self::Rot(c) => c.lin_or_rot(),
        }
    }

    fn axial_coil_overhang(&self) -> Length {
        match self {
            Self::Lin(c) => c.axial_coil_overhang(),
            Self::Rot(c) => c.axial_coil_overhang(),
        }
    }

    fn flux_barrier(&self) -> Option<&dyn crate::flux_barrier::FluxBarrier> {
        match self {
            Self::Lin(c) => c.flux_barrier(),
            Self::Rot(c) => c.flux_barrier(),
        }
    }

    fn shape(&self) -> &Shape {
        match self {
            Self::Lin(c) => c.shape(),
            Self::Rot(c) => c.shape(),
        }
    }

    fn as_core_ref(&self) -> CoreRef<'_> {
        return self.clone();
    }

    fn pole_coverage(&self, surface_magnet_assembly: Option<&MagnetAssembly>) -> f64 {
        match self {
            Self::Lin(c) => c.pole_coverage(surface_magnet_assembly),
            Self::Rot(c) => c.pole_coverage(surface_magnet_assembly),
        }
    }
}

impl<'a> From<&'a Core> for CoreRef<'a> {
    fn from(value: &'a Core) -> Self {
        match value {
            Core::Lin(c) => Self::Lin(c),
            Core::Rot(c) => Self::Rot(c),
        }
    }
}

impl<'a> From<&'a LinCore> for CoreRef<'a> {
    fn from(value: &'a LinCore) -> Self {
        Self::Lin(value)
    }
}

impl<'a> From<&'a RotCore> for CoreRef<'a> {
    fn from(value: &'a RotCore) -> Self {
        Self::Rot(value)
    }
}

impl<'a, 'b> From<&'b CoreRef<'a>> for CoreRef<'a> {
    fn from(value: &'b CoreRef<'a>) -> Self {
        value.clone()
    }
}
