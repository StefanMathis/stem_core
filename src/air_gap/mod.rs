use std::f64::consts::PI;

use dyn_clone::DynClone;
use planar_geo::{polysegment::Polysegment, shape::Shape};
use stem_magnet::assembly::MagnetAssembly;
use stem_slot::prelude::*;

use crate::{
    core::{CoreExt, CoreRef},
    error::Error,
    magnets::Magnets,
    winding_zones::WindingZones,
};

pub mod plain;
pub mod slotted;
pub mod straight_indents;

pub use plain::PlainAirGap;
pub use slotted::{CarterFactorModel, SlottedAirGap};
pub use straight_indents::{AirGapPolygonBuilder, StraightIndentsAirGap};

#[cfg_attr(feature = "serde", typetag::serde)]
pub trait AirGap: DynClone + Sync + Send + std::fmt::Debug + std::any::Any {
    /// Axial segments of the core
    fn num_segments(&self, core: CoreRef<'_>) -> usize;

    fn surface_magnets(
        &self,
        magnet_assembly: &MagnetAssembly,
        core: CoreRef<'_>,
        split: bool,
    ) -> Magnets;

    fn winding_zones(&self, core: CoreRef<'_>, coil_layout: &CoilLayout) -> WindingZones;

    /// Return the number of slots. If zero, slot is not windable. This number
    /// corresponds to the "slots" property of a winding, not necessarily to a
    /// physical [`Slot`]. For example, a plain air gap has no slot, but still
    /// can be windable.
    fn slots(&self, core: CoreRef<'_>) -> u16;

    /// Combines `self` with the `core` and returns the resulting cross-section
    /// shape.
    ///
    /// This method is used when creating a [`LinCore`] / [`RotCore`] out of a
    /// [`LinCoreBuilder`] / [`RotCoreBuilder`]. It therefore functions as a
    /// general hook to check the compatibility of the [`AirGap`] with the core.
    /// For example, when combining an [`SlottedAirGap`] with a [`LinCore`], the
    /// latter must be high enough to accomodate the slot (otherwise the shape
    /// creation will fail). But even if the shape creation succeeds, an
    /// [`AirGap`] might still be incompatible to a core: If for example the
    /// [`air_gap_winding_height`](PlainAirGap::air_gap_winding_height) of a
    /// [`PlainAirGap`] is larger than inner air gap radius of a [`RotCore`],
    /// the winding does not fit inside the core.
    ///
    /// Some implementors of [`AirGap`] might also be generally incompatible to
    /// either a [`LinCore`] or a [`RotCore`]. In this case, this method should
    /// return [`Error::IncompatibleToLinCore`] or
    /// [`Error::IncompatibleToRotCore`], where the `&'static str` represents
    /// the type name.
    ///
    /// TODO: Example for successfull and failing combination
    /// TODO: Slotted core image (linear core?)
    fn combine(&mut self, core: CoreRef<'_>) -> Result<Shape, Error>;

    // =========================================================================

    /**
    Return the cross section area of a winding zone. In case of a slotted air gap, this is the windable slot area.
     */
    fn zone_area(&self, _core: CoreRef<'_>) -> Area {
        Area::new::<square_meter>(0.0)
    }

    fn tooth_height(&self, _core: CoreRef<'_>) -> Length {
        return Length::new::<meter>(0.0);
    }

    fn tooth_width_at(&self, _core: CoreRef<'_>, _height: Length) -> Length {
        return Length::new::<meter>(0.0);
    }

    fn carter_factor(&self, _core: CoreRef<'_>, _air_gap_width: Length) -> f64 {
        return 1.0;
    }

    fn slot(&self, _core: CoreRef<'_>) -> Option<&dyn Slot> {
        return None;
    }

    fn current_displacement_coefficients(
        &self,
        _core: CoreRef<'_>,
    ) -> CurrentDisplacementCalculator {
        return CurrentDisplacementCalculator::from_slice_dims([].into_iter());
    }

    /// Calculate the slot opening factor for a non-slotted core according to
    /// eq. (1.2.63) in [MVP08]
    fn slot_opening_factor(&self, slots: u16, ordinal: f64, core: CoreRef<'_>) -> f64 {
        return slot_opening_factor(core.pole_pairs(), slots, ordinal);
    }
}

dyn_clone::clone_trait_object!(AirGap);

/// Calculate the slot opening factor for a non-slotted core according to eq.
/// (1.2.63) in [MVP08]
fn slot_opening_factor(pole_pairs: u16, slots: u16, ordinal: f64) -> f64 {
    let arg = ordinal * pole_pairs as f64 * PI / slots as f64;
    return (arg.sin() / arg).abs();
}

/**
Helper function to combine stator and rotor segment_chain to a shape
 */
fn combine_air_gap_and_yoke_to_shape(
    air_gap: Polysegment,
    yoke: Polysegment,
    is_outer: bool,
) -> Result<Shape, Error> {
    let (outer, inner) = if is_outer {
        (yoke, air_gap)
    } else {
        (air_gap, yoke)
    };
    return Shape::new(vec![outer.into(), inner.into()]).map_err(From::from);
}
