pub mod star1;
pub mod v1r;
pub mod v2r;

pub use star1::{Star1HeightSplit, Star1FluxBarrier};
pub use v1r::V1rFluxBarrier;
pub use v2r::V2rFluxBarrier;

use std::{any::Any, f64::consts::FRAC_PI_2};

use dyn_clone::DynClone;
use planar_geo::contour::Contour;
use stem_magnet::assembly::MagnetAssembly;
use stem_slot::prelude::*;

use crate::{core::CoreRef, error::Error, magnets::Magnets};

/// The `FluxBarrier` trait allows the usage of structs as flux barriers.
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait FluxBarrier: DynClone + Any + Sync + Send + std::fmt::Debug + 'static {
    /// TODO
    fn magnets(&self, core: CoreRef<'_>) -> &[MagnetAssembly];

    /// TODO
    fn starts_in_d_axis(&self, core: CoreRef<'_>) -> bool;

    /**
    Returns the pole coverage of the flux barrier. The pole coverage is the air gap area of the d-axis flux over the total air gap area
    and is therefore a value between 0 and 1. This is usually the area between the flux leakage paths.
     */
    fn pole_coverage(&self, core: CoreRef<'_>) -> f64;

    fn interior_magnets(&self, core: CoreRef<'_>, split: bool) -> Magnets;

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
    fn combine(&mut self, core: CoreRef<'_>) -> Result<Vec<Contour>, Error>;

    fn magnet_assemblies(&self, _core: CoreRef<'_>) -> &[MagnetAssembly] {
        return &[];
    }
}

dyn_clone::clone_trait_object!(FluxBarrier);

// =============================================================================

/**
Returns the slot bottom middle and the slot index which is closest to the point
`pt`.
*/
pub(crate) fn closest_slot_bottom_middle_rot(
    pt: [f64; 2],
    core: &crate::core::RotCore,
    air_gap: &crate::air_gap::SlottedAirGap,
) -> ([f64; 2], u16) {
    use crate::core::CoreExt;
    use stem_magnet::planar_geo::Transformation;
    use uom::si::length::meter;

    let offset_slot = if air_gap.starts_in_slot_middle() {
        0.0
    } else {
        0.5
    };

    let m = if core.is_outer() { 1.0 } else { -1.0 };
    let air_gap_radius = core.air_gap_radius().get::<meter>();

    let slots = core.slots();
    let slot_height = core.tooth_height().get::<meter>();
    let mut closest_slot_bottom_middle_dist = std::f64::INFINITY;
    let mut closest_slot_bottom_middle_idx = 0;
    let mut closest_slot_bottom_middle_pt = [0.0, 0.0]; // Dummy value, will be overwritten in the loop
    let slot_bottom_middle = [air_gap_radius + m * slot_height, 0.0];
    for slot in 0..slots {
        let slot_angle = std::f64::consts::TAU / slots as f64 * (slot as f64 + offset_slot);
        let mut slot_bottom_middle = slot_bottom_middle.clone();
        slot_bottom_middle.rotate([0.0, 0.0], slot_angle);
        let dist_sqr =
            (slot_bottom_middle[0] - pt[0]).powi(2) + (slot_bottom_middle[1] - pt[1]).powi(2);
        if dist_sqr < closest_slot_bottom_middle_dist {
            closest_slot_bottom_middle_dist = dist_sqr;
            closest_slot_bottom_middle_idx = slot;
            closest_slot_bottom_middle_pt = slot_bottom_middle;
        }
    }
    return (
        closest_slot_bottom_middle_pt,
        closest_slot_bottom_middle_idx,
    );
}

/// Returns the distance from the given point to the q-axis
pub(crate) fn dist_to_q_axis(pt: [f64; 2], pole_pairs: u16) -> f64 {
    let x = pt[0];
    let y = pt[1];

    // Define the q-axis line from the origin to the air gap as an equation ax + by
    // + c
    let angle = FRAC_PI_2 * (1.0 - 1.0 / (pole_pairs as f64));
    let x1 = angle.cos();
    let y1 = angle.sin();
    let x2 = 0.0;
    let y2 = 0.0;

    // Formula for the distance of a point (x,y) from a line defined by two points
    // (x1,y1) and (x2,y2). Point (x2,y2) is the origin.
    return ((y2 - y1) * x - (x2 - x1) * y + x2 * y1 - y2 * x1).abs()
        / ((y2 - y1).powi(2) + (x2 - x1).powi(2)).sqrt();
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::prelude::*;
    use stem_slot::semi_trapezoid::SemiTrapezoidWidthsAndHeightsBuilder;

    fn create_core() -> RotCore {
        let slot: SemiTrapezoidSlot = SemiTrapezoidWidthsAndHeightsBuilder {
            bottom_width: Length::new::<millimeter>(6.76),
            bottom_side_width: Length::new::<millimeter>(6.76),
            top_side_width: Length::new::<millimeter>(8.0),
            top_width: Length::new::<millimeter>(1.5),
            opening_width: Length::new::<millimeter>(1.5),
            bottom_height: Length::new::<millimeter>(0.0),
            side_height: Length::new::<millimeter>(6.79 - 0.75 - 0.5),
            top_height: Length::new::<millimeter>(0.5),
            opening_height: Length::new::<millimeter>(0.75),
            bottom_radius: Length::new::<millimeter>(0.0),
            bottom_side_radius: Length::new::<millimeter>(0.0),
            top_radius: Length::new::<millimeter>(0.0),
            top_side_radius: Length::new::<millimeter>(0.0),
            opening_radius: Length::new::<millimeter>(0.0),
            consider_tooth_tip_leakage: true,
        }
        .try_into()
        .unwrap();
        let air_gap = SlottedAirGap::new(28, false, CarterFactorModel::Bin12, Box::new(slot));
        let core = RotCoreBuilder {
            air_gap_radius: Length::new::<millimeter>(54.4),
            yoke_radius: Length::new::<millimeter>(19.0),
            axial_length: Length::new::<millimeter>(165.0),
            axial_coil_overhang: Length::new::<millimeter>(0.0),
            iron_fill_factor: 1.0,
            material: Arc::new(Material::default()),
            pole_pairs: 2,
            skew_angle: 0.0,
            air_gap: Box::new(air_gap),
            flux_barrier: None,
        };

        return core.try_into().unwrap();
    }

    #[test]
    fn test_closest_slot_bottom_middle() {
        let core = create_core();
        let (closest_pt, slot) = closest_slot_bottom_middle_rot(
            [0.04, 0.01],
            &core,
            (core.air_gap() as &dyn std::any::Any)
                .downcast_ref::<SlottedAirGap>()
                .unwrap(),
        );
        assert_eq!(slot, 1);
        approx::assert_abs_diff_eq!(closest_pt[0], 0.0449, epsilon = 1e-3);
        approx::assert_abs_diff_eq!(closest_pt[1], 0.0157, epsilon = 1e-3);
    }
}
