/*!
A module providing the the [`LinCore`] and [`RotCore`] structs, the owning and
borrowing enum wrappers [`Core`] and [`CoreRef`] and the sealed trait
[`CoreExt`] which provides a common interface for both core types.

# Overview

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
rotor contour at the air gap. That contour is defined by an
[`AirGap`](crate::air_gap::AirGap) trait object used to create the [`LinCore`] /
[`RotCore`] instance. The following image shows an example for both core types
where the air gap shape is defined by [`Slot`](stem_slot::slot::Slot)s:
*/
#![cfg_attr(feature = "doc-images", doc = "")]
#![cfg_attr(
    feature = "doc-images",
    doc = "![Slotted linear and rotary core][lin_and_rot_core_slotted.svg]"
)]
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("lin_and_rot_core_slotted.svg", "docs/img/lin_and_rot_core_slotted.svg"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/*!
_Image created with `/examples/lin_and_rot_core_plots.rs`._

The core shape can be further customized by optionally inserting a
[`FluxBarrier`](crate::flux_barrier::FluxBarrier) into the core, which can both
be used to direct the magnetic flux and to provide space for interior magnets.
The latter aspect is discussed in depth further below.

Since every core in stem is either a [`LinCore`] or a [`RotCore`], an owning
([`Core`]) and a borrowing ([`CoreRef`]) wrapper is provided as well. The
sealed [`CoreExt`] trait provides a common interface for both [`LinCore`] and
[`RotCore`] and therefore also for the [`Core`] and [`CoreRef`] enums.

# Windable cores

A core is called _windable_ if its [`AirGap`](crate::air_gap::AirGap) contour
allows for a winding to be mounted (see [`CoreExt::windable`]). The following
image shows two different examples: One with a
[`PlainAirGap`](crate::air_gap::PlainAirGap), where the winding is mounted
directly on the core / in the air gap, and one with a
[`SlottedAirGap`](crate::air_gap::SlottedAirGap), where the winding is mounted
inside the slots along the air gap contour.

*/
#![cfg_attr(feature = "doc-images", doc = "")]
#![cfg_attr(
    feature = "doc-images",
    doc = "![Winding in core][lin_core_air_gap_and_slotted_winding.svg]"
)]
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("lin_core_air_gap_and_slotted_winding.svg", "docs/img/lin_core_air_gap_and_slotted_winding.svg"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/*!
_Image created with `/examples/lin_and_rot_core_plots.rs`._

It is evident that the space available for the individual winding zones is
determined by the specific [`AirGap`](crate::air_gap::AirGap) trait object.
The [`CoreExt::winding_zones`] returns an iterator over the winding zone
contours together with the respective [`Zone`](crate::winding_zones::Zone)
index. This iterator can be used for e.g. creating a visualization of the
winding itself (as shown in the previous image) or to determine the available
area for the winding and hence its resistance and current carrying capacity.

# Surface magnets

Some [`AirGap`](crate::air_gap::AirGap)s allow for mounting magnets directly on
the core surface / in the core air gap. Similar to [`CoreExt::winding_zones`],
the [`CoreExt::surface_magnets`] method returns an iterator over the
[`PositionedMagnetShape`](crate::magnets::PositionedMagnetShape)s for a provided
[`MagnetAssembly`]. The [`CoreExt::collision_check`] method can be used to
check if the mounting results in collisions and therefore if the provided
[`MagnetAssembly`] is compatible with the core. The image below shows an example
for a linear core with an assembly consisting of [`BreadLoafMagnet`]s and for a
rotary core with an assembly consisting of [`ArcParallelMagnet`]s.

*/
#![cfg_attr(feature = "doc-images", doc = "")]
#![cfg_attr(
    feature = "doc-images",
    doc = "![Surface magnets on cores][lin_and_rot_core_surface_magnets.svg]"
)]
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("lin_and_rot_core_surface_magnets.svg", "docs/img/lin_and_rot_core_surface_magnets.svg"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/*!
_Image created with `/examples/lin_and_rot_core_plots.rs`._

# Interior magnets

Some [`FluxBarrier`](crate::flux_barrier::FluxBarrier)s provide space for
interior magnets. Similar to [`CoreExt::surface_magnets`], the
[`CoreExt::interior_magnets`] method returns an iterator over the
[`PositionedMagnetShape`](crate::magnets::PositionedMagnetShape)s of the
interior magnets. The type of those magnets is determined by the
[`FluxBarrier`](crate::flux_barrier::FluxBarrier), hence it is not necessary to
specifiy the [`MagnetAssembly`] for the interior magnets. In the image below,
a linear and a rotary core with the
[`Star1FluxBarrier`](crate::flux_barrier::Star1FluxBarrier) are shown.
*/
#![cfg_attr(feature = "doc-images", doc = "")]
#![cfg_attr(
    feature = "doc-images",
    doc = "![Interior magnets in cores][lin_and_rot_core_interior_magnets.svg]"
)]
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("lin_and_rot_core_interior_magnets.svg", "docs/img/lin_and_rot_core_interior_magnets.svg"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/*!
_Image created with `/examples/lin_and_rot_core_plots.rs`._
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
use crate::error::IncompatibleFluxBarrier;
use crate::flux_barrier::FluxBarrier;

/**
An owning enum  for a [`LinCore`] or [`RotCore`].

This enum is meant for use cases where a magnetic core is needed but the type
(linear or rotary) is unknown until runtime. This enum provides an type-erased
wrapper which "behaves like" a magnetic core by implementing [`CoreExt`]. Every
method implementation looks like this:

```ignore
fn pole_pairs(&self) -> u16 {
    match self {
        Self::Lin(c) => c.pole_pairs(),
        Self::Rot(c) => c.pole_pairs(),
    }
}
```
 */
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Core {
    /// A wrapped [`LinCore`].
    Lin(LinCore),
    /// A wrapped [`RotCore`].
    Rot(RotCore),
}

impl Core {
    /**
    Returns a reference to the underlying [`LinCore`], if `self` wraps one.
    If `self` wraps a [`RotCore`], this method returns `None` instead.
     */
    pub fn lin<'core>(&'core self) -> Option<&'core LinCore> {
        match self {
            Self::Lin(c) => Some(c),
            Self::Rot(_) => None,
        }
    }

    /**
    Returns a reference to the underlying [`RotCore`], if `self` wraps one.
    If `self` wraps a [`LinCore`], this method returns `None` instead.
     */
    pub fn rot<'core>(&'core self) -> Option<&'core RotCore> {
        match self {
            Self::Lin(_) => None,
            Self::Rot(c) => Some(c),
        }
    }

    /// Fallibly inserts a new [`FluxBarrier`] into `self` or removes an
    /// existing one.
    ///
    /// If `flux_barrier` is `Some`, it is checked whether the wrapped
    /// [`FluxBarrier`] is compatible to `self` by creating the flux barrier
    /// contours via the [`FluxBarrier::combine`] method and checking if those
    /// fit into the shape of `self`. If [`FluxBarrier::combine`] fails or if
    /// the contours don't fit, the resulting error is wrapped into
    /// [`IncompatibleFluxBarrier`] and returned together with the given
    /// [`FluxBarrier`]. Otherwise, the old flux barrier of `self` will be
    /// replaced with the new one. If `flux_barrier` is `None` and `self` has a
    /// flux barrier, it will be removed. Otherwise, this is a no-op.
    ///
    /// This method forwards to either [`LinCore::set_flux_barrier`] or
    /// [`RotCore::set_flux_barrier`], see their docstrings for examples.
    pub fn set_flux_barrier(
        &mut self,
        flux_barrier: Option<Box<dyn FluxBarrier>>,
    ) -> Result<(), IncompatibleFluxBarrier> {
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
A borrowing enum for a [`LinCore`] or [`RotCore`].

This enum is meant for use cases where a reference to a magnetic core is needed
but the type (linear or rotary) is unknown until runtime. This enum provides an
type-erased wrapper which "behaves like" a magnetic core by implementing
[`CoreExt`]. Every method implementation looks like this:

```ignore
fn pole_pairs(&self) -> u16 {
    match self {
        Self::Lin(c) => c.pole_pairs(),
        Self::Rot(c) => c.pole_pairs(),
    }
}
```

This wrapper is particularily useful for the [`AirGap`](crate::air_gap::AirGap)
and [`FluxBarrier`] traits, since these are meant to be used for creating trait
objects and therefore don't have generic methods. Therefore, the trait methods
require passing a [`CoreRef`] object, which can be created via the
[`CoreExt::as_core_ref`] method from a [`LinCore`], [`RotCore`] or [`Core`].
 */
#[derive(Debug, Clone, Copy)]
pub enum CoreRef<'a> {
    /// A reference to a [`LinCore`].
    Lin(&'a LinCore),
    /// A reference to a [`RotCore`].
    Rot(&'a RotCore),
}

impl<'a> CoreRef<'a> {
    /**
    Returns the underlying [`LinCore`] reference, if `self` wraps one.
    If `self` wraps a [`RotCore`] reference, this method returns `None` instead.
     */
    pub fn lin<'core>(&'core self) -> Option<&'core LinCore> {
        match self {
            CoreRef::Lin(c) => Some(*c),
            CoreRef::Rot(_) => None,
        }
    }

    /**
    Returns the underlying [`RotCore`] reference, if `self` wraps one.
    If `self` wraps a [`LinCore`] reference, this method returns `None` instead.
     */
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
