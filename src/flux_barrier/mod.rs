pub mod star1;
pub mod v1r;
pub mod v2r;

pub use star1::{Star1FluxBarrier, Star1HeightSplit};
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
    /**
    Returns the pole coverage of the flux barrier. The pole coverage is the air gap area of the d-axis flux over the total air gap area
    and is therefore a value between 0 and 1. This is usually the area between the flux leakage paths.
     */
    fn pole_coverage(&self, core: CoreRef<'_>) -> f64;

    /// Returns an iterator over the interior magnet shapes for the given
    /// `core`.
    ///
    /// This method implements [`CoreExt::interior_magnets`] for the different
    /// possible flux barrier types. For example, for a [`Star1FluxBarrier`],
    /// the magnets are arranged in the center of the rectangular cutouts within
    /// the core.
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Interior magnets for a linear and a rotary motor][lin_and_rot_core_interior_magnets]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "lin_and_rot_core_interior_magnets",
            "docs/img/lin_and_rot_core_interior_magnets.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    /// _This image was produced with `examples/lin_and_rot_core_plots.rs`._
    ///
    /// The [crate::magnets::PositionedMagnetShape] elements returned by the
    /// iterator have a
    /// [`magnet_idx`](crate::magnets::PositionedMagnetShape::magnet_idx). By
    /// indexing into the slice returned by [`FluxBarrier::magnet_assemblies`],
    /// the assembly type to which the shape belongs can be found.
    ///
    /// When implementing this method for custom [`FluxBarrier`]s, the following
    /// rules should be followed:
    /// - If the flux barrier does not contain any magnets, an empty iterator
    /// should be returned (for example
    /// `Magnets::Other(Box::new([].into_iter()))`. For that particular case,
    /// [`FluxBarrier::magnet_assemblies`] should likewise return an empty
    /// slice.
    /// - The iterator should return `n`
    /// [`PositionedMagnetShape`](crate::magnets::PositionedMagnetShape)
    /// elements, where `n` is
    /// `magnets_per_pole * (1 + split) * core.poles()`. `magnets_per_pole` is
    /// the sum of the number of magnets of all magnet assemblies (as returned
    /// from [`FluxBarrier::magnet_assemblies`]).
    /// - The shapes must not overlap each other or the
    /// [`core.shape()`](CoreExt::Shape). This can be checked using
    /// [`CoreExt::assembly_check`].
    /// - The element indices
    /// ([`PositionedMagnetShape::magnet_idx`](crate::magnets::PositionedMagnetShape::magnet_idx))
    /// must indicate the magnet assembly type to which the returned shape
    /// belongs. For example, if `self.magnet_assemblies().len() == 2` and the
    /// `magnet_idx` of a returned shape is 1, that shape belongs to the second
    /// magnet assembly in the slice. Returning a `magnet_idx` which is out of
    /// bounds of the slice will result in a panic when calling some methods
    /// like e.g. [`CoreExt::mass_interior_magnets`].
    /// - If `split` is true, each magnet should be separated in its north and
    /// south shape using
    /// [`Magnet::north_south_shapes`](stem_magnet::magnet::Magnet::north_south_shapes).
    /// Otherwise, the whole magnet shape should be returned. When returning the
    /// shapes for a negative pole, the shapes need to be adjusted for polarity
    /// (see [`PositionedMagnetShape::is_north`](crate::magnets::PositionedMagnetShape::is_north)).
    ///
    /// The [`crate::magnets`] module contains some predefined iterators
    /// to simplify the implementation of this method, see e.g. the source code
    /// of [`Star1FluxBarrier::interior_magnets`] for an example.
    fn interior_magnets(&self, _core: CoreRef<'_>, _split: bool) -> Magnets {
        // Dummy implementation, to be overwritten.
        return crate::magnets::EvenlyDistributedMagnets::<true>::new(
            0,
            Length::new::<meter>(0.0),
            Vec::new(),
            0.0,
            0,
        )
        .into();
    }

    /// Combines `self` with the `core` and returns the resulting cross-section
    /// shape.
    ///
    /// This method is used when creating a [`LinCore`](crate::core::LinCore) /
    /// [`RotCore`](crate::core::RotCore) out of a
    /// [`LinCoreBuilder`](crate::core::LinCoreBuilder) /
    /// [`RotCoreBuilder`](crate::core::RotCoreBuilder). It therefore functions
    /// as a general hook to check the compatibility of the [`AirGap`] with
    /// the core. For example, when combining an [`SlottedAirGap`] with a
    /// [`LinCore`](crate::core::LinCore), the latter must be high enough to
    /// accomodate the slot (otherwise the shape creation will fail). But
    /// even if the shape creation succeeds, an [`AirGap`] might still be
    /// incompatible to a core: If for example the
    /// [`air_gap_winding_height`](PlainAirGap::air_gap_winding_height) of a
    /// [`PlainAirGap`] is larger than inner air gap radius of a
    /// [`RotCore`](crate::core::RotCore), the winding does not fit inside
    /// the core.
    ///
    /// Some implementors of [`AirGap`] might also be generally incompatible to
    /// either a [`LinCore`](crate::core::LinCore) or a
    /// [`RotCore`](crate::core::RotCore). In this case, this method should
    /// return [`Error::IncompatibleToLinCore`] or
    /// [`Error::IncompatibleToRotCore`], where the `&'static str` represents
    /// the type name.
    ///
    /// TODO: Example for successfull and failing combination
    /// TODO: Slotted core image (linear core?)
    fn combine(&mut self, core: CoreRef<'_>) -> Result<Vec<Contour>, Error>;

    /**
    Returns all magnet assemblies of a single pole placed within the flux
    barrier.

    A flux barrier may be able to contain magnets of one or even multiple types
    which are part of [`MagnetAssemblies`](MagnetAssembly). This method returns
    a slice view of all assemblies for a single pole. The total number of
    magnets per pole is therefore
    `self.magnet_assemblies(core).iter().map(|m|m.num_magnets()).sum()`. If a
    magnet of an assembly is used multiple times within the cross section,
    [`MagnetAssembly::num_tangential`] should be set to the times of occurences.
    For example, [`MagnetAssembly::num_tangential`] is 2 for a
    [`V1rFluxBarrier`], but 1 for a [`Star1FluxBarrier`]. The number of axial
    subdivisions depends entirely on the flux barrier implementation.

    The [`FluxBarrier::interior_magnets`] returns an iterator over the shapes
    of all interior magnets within `self`. By indexing into this slice with
    [`PositionedMagnetShape::magnet_idx`](crate::magnets::PositionedMagnetShape::magnet_idx),
    the magnet assembly to which a particular shape belongs can be determined.
    For example, if `self.magnet_assemblies().len() == 2` and the
    `PositionedMagnetShape::magnet_idx` of a returned shape is 1, that shape
    belongs to the second magnet assembly in the slice. This can e.g. be used to
    calculate the total mass of all interior magnets (see source code of
    [`CoreExt::mass_interior_magnets`](crate::core::CoreExt::mass_interior_magnets)
    ). When implementing [`FluxBarrier`] for an external type, this relation
    needs to be uphold, because otherwise these calculations will return wrong
    results.

    If the flux barrier does not hold magnets
    ([`FluxBarrier::interior_magnets`](crate::core::CoreExt::interior_magnets)
    returns an empty iterator), this method can be implemented by simply
    returning an empty slice.

    # Examples

    ```
    use std::f64::consts::FRAC_PI_2;
    use std::sync::Arc;

    use stem_core::prelude::*;

    let star1 = Star1FluxBarrier {
        magnet_space_width: Length::new::<millimeter>(10.0),
        glue_gap: Length::new::<millimeter>(0.2),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(5.0),
        magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<
            millimeter,
        >(2.0)),
    };
    let star1_core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(18.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        skew_angle: 0.0,
        air_gap: Box::new(PlainAirGap::default()),
        flux_barrier: Some(Box::new(star1)),
    }
    .try_into()
    .unwrap();

    let v1r = V1rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(4.5),
        relief_path_air_gap_width: Length::new::<millimeter>(3.5),
        relief_path_length: Length::new::<millimeter>(1.33),
        relief_path_width: Length::new::<millimeter>(4.0),
        opening_angle: FRAC_PI_2,
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height: Length::new::<millimeter>(20.0),
        glue_gap: Length::new::<millimeter>(0.2),
        leakage_path_width: Length::new::<millimeter>(1.0),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
    };
    let v1r_core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(18.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        skew_angle: 0.0,
        air_gap: Box::new(PlainAirGap::default()),
        flux_barrier: Some(Box::new(v1r)),
    }
    .try_into()
    .unwrap();

    let binding = star1_core.flux_barrier();
    let star1 = binding.as_ref().expect("has flux barrier");
    let sum_star1_mags: usize = star1.magnet_assemblies(star1_core.as_core_ref()).iter().map(|m|m.num_magnets()).sum();
    assert_eq!(sum_star1_mags, 1);

    let binding = v1r_core.flux_barrier();
    let v1r = binding.as_ref().expect("has flux barrier");
    let sum_v1r_mags: usize = v1r.magnet_assemblies(v1r_core.as_core_ref()).iter().map(|m|m.num_magnets()).sum();
    assert_eq!(sum_v1r_mags, 2);
    ```
     */
    fn magnet_assemblies(&self, core: CoreRef<'_>) -> &[MagnetAssembly];
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
