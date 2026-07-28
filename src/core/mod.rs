/*!
A module providing the the [`LinCore`] and [`RotCore`] structs, the owning and
borrowing enum wrappers [`Core`] and [`CoreRef`] and the sealed trait
[`CoreExt`].

Both the stator and the rotor of a radial flux machine are typically built
around a magnetic core, which serves as a scaffolding for mounting magnets and
windings and guides the magnetic flux produced by them. Radial flux machines can
be either _linear_ (meaning that the rotor performs a linear movement relative
to the stator) or _rotary_ (meaning that the rotor rotates around its center).
For a linear machine, both stator and rotor are usually cuboids, whereas in the
latter case, they are annular (hollow) cylinders which share a common rotation
axis. In stem_core, a linear core is modeled by the [`LinCore`] type, a
rotary core by the [`RotCore`] type.

The magnetic force moving a motor / powering a generator is created by magnetic
flux passing through the air gap between stator and rotor. The local
distribution of that flux is heavily influenced by the shape of the stator /
rotor contour at the air gap. That contour is defined by the
[`AirGap`](crate::air_gap::AirGap) implementor used to create the [`LinCore`] /
[`RotCore`] instance. The following image shows an example for both core types
where the air gap shape is defined by [`Slot`](stem_slot::slot::Slot)s:
*/
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("lin_and_rot_core.svg", "docs/img/lin_and_rot_core.svg"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/*!
In addition to the air gap, the magnetic flux can be further directed by
introducing [`FluxBarrier`](crate::flux_barrier::FluxBarrier)s (holes) in
the core. See the trait docstring for more.

Since every core in stem is either a [`LinCore`] or a [`RotCore`], an owning
([`Core`]) and a borrowing ([`CoreRef`]) wrapper is provided as well. The
sealed [`CoreExt`] trait provides a common interface for both [`LinCore`] and
[`RotCore`] and therefore also for the [`Core`] and [`CoreRef`] enums. See its
docstring for more.
 */

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

impl TryFrom<Core> for LinCore {
    type Error = Core;

    fn try_from(value: Core) -> Result<Self, Self::Error> {
        match value {
            Core::Lin(c) => Ok(c),
            Core::Rot(_) => Err(value),
        }
    }
}

impl TryFrom<Core> for RotCore {
    type Error = Core;

    fn try_from(value: Core) -> Result<Self, Self::Error> {
        match value {
            Core::Lin(_) => Err(value),
            Core::Rot(c) => Ok(c),
        }
    }
}

// =============================================================================

/**
TODO

This wrapper is particularily useful for the [`AirGap`](crate::air_gap::AirGap)
and [`FluxBarrier`](crate::flux_barrier::FluxBarrier) traits, since these are
meant to be used as trait objects in [`LinCore`] or [`RotCore`] and therefore
must not have generic methods. Therefore, the trait methods require passing a
[`CoreRef`] object, which can be created via the [`CoreExt::as_core_ref`]
method from a [`LinCore`], [`RotCore`] or [`Core`].
 */
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
