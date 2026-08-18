/*!
This module defines the [`FluxBarrier`] trait used to define cutouts inside the
core yoke for influencing the magnetic flux paths and for mounting interior
magnets.

Besides the aforementioned trait, this module also reexports some implementors
of the [`FluxBarrier`] trait as well as auxiliary types and functions:
- [`Spoke1FluxBarrier`] (reexported from the [`spoke1`] module) defines a spoke
desgin flux barrier for a single block magnet (hence the "1") with an optional
flux relief path next to the flux leakage path at the air gap.
- [`V1rFluxBarrier`] (reexported from the [`v1r`] module) defines a V-shaped
flux barrier with an (optional) flux relief path in the middle of the V.
- [`V2rFluxBarrier`] (reexported from the [`v2r`] module) defines a V-shaped
flux barrier with two (optional) flux relief paths at the ends of the V (next
to the flux leakage paths).

See the [trait documentation](FluxBarrier) for details.
 */

pub mod spoke1;
pub mod v1r;
pub mod v2r;

pub use spoke1::{Spoke1FluxBarrier, Spoke1HeightSplit};
pub use v1r::V1rFluxBarrier;
pub use v2r::V2rFluxBarrier;

use std::{any::Any, f64::consts::FRAC_PI_2};

use dyn_clone::DynClone;
use planar_geo::contour::Contour;
use stem_magnet::assembly::MagnetAssembly;
use stem_slot::prelude::*;

use crate::{
    core::{CoreExt, CoreRef},
    error::Error,
    magnets::Magnets,
};

/**
A trait to define "flux barriers": Cutouts in the yoke of cores which can be
used to steer the magnetic flux and to hold interior magnets.

This trait is used to create the `flux_barrier` trait objects for the core
builder structs [`LinCoreBuilder`](crate::core::LinCoreBuilder) and
[`RotCoreBuilder`](crate::core::RotCoreBuilder). A core may or may not have a
flux barrier (in the latter case, `flux_barrier` is simply set to `None`). The
image below shows two cores which are identical except for their flux barrier
(`None` for the left core and [`V1rFluxBarrier`] for the right one).
 */
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Slotted core with and without a flux barrier][slotted_core_with_and_without_fb]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image(
        "slotted_core_with_and_without_fb",
        "docs/img/slotted_core_with_and_without_fb.svg"
    )
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

_This image was produced with `examples/flux_barrier_plots.rs`._

A flux barrier consists of one or more cutouts in the core yoke which are
usually repeated along the individual poles. Since cores are usually made from
ferromagnetic material, the magnetic flux avoids the cutouts. This can be used
to purposefully introduce a difference in the d- and q-axes inductances,
creating a reluctance force / torque. Additonally, the cutouts can also be used
to mount permanent magnets:
 */
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Interior magnets for a linear and a rotary core][lin_and_rot_core_interior_magnets]"
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
/**

_This image was produced with `examples/lin_and_rot_core_plots.rs`._

The trait methods are not meant to be called by user code. Instead, all of them
(except [`FluxBarrier::combine`]) are used to implement the
[`CoreExt`] methods of the same name. For example, the
[`CoreExt::d_axis_offset`] method is implemented like this:

```ignore
fn d_axis_offset(&self) -> u16 {
    match self.flux_barrier() {
        Some(fb) => fb
            .d_axis_offset(self.as_core_ref())
            .rem_euclid(std::f64::consts::TAU),
        None => FRAC_PI_2,
    }
}
```

Generally speaking, the documentation of the [`CoreExt`] method therefors
focuses on the _usage_ of that specific method, whereas the [`FluxBarrier`]
method docstring explains how to _implement_ it for custom flux barrier types.
If the latter have examples, they are just there to show how the method is
supposed to work, not how to use them in user code.

The [`FluxBarrier::combine`] method is used in
[`LinCore::new`](crate::core::LinCore::new) and
[`RotCore::new`](crate::core::RotCore::new) to create the flux barrier cutouts
in the core shape. See the trait method documentation for examples.
 */
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait FluxBarrier: DynClone + Any + Sync + Send + std::fmt::Debug + 'static {
    /// Combines `self` with the `core` and returns the cutout contours for the
    /// entire core.
    ///
    /// This method is used when creating a [`LinCore`](crate::core::LinCore) /
    /// [`RotCore`](crate::core::RotCore) out of a
    /// [`LinCoreBuilder`](crate::core::LinCoreBuilder) /
    /// [`RotCoreBuilder`](crate::core::RotCoreBuilder) OR when using
    /// [`LinCore::set_flux_barrier`](crate::core::LinCore::set_flux_barrier) /
    /// [`RotCore::set_flux_barrier`](crate::core::RotCore::set_flux_barrier) to
    /// modify an existing core. It therefore functions as a general hook to
    /// check the compatibility of the [`FluxBarrier`] with the core. The
    /// contours returned by this method are (fallibly) inserted into the core
    /// shape as "holes".
    ///
    /// Some implementors of [`FluxBarrier`] might also be generally
    /// incompatible to either a [`LinCore`](crate::core::LinCore) or a
    /// [`RotCore`](crate::core::RotCore). In this case, this method should
    /// return [`Error::IncompatibleToLinCore`] or
    /// [`Error::IncompatibleToRotCore`], where the `&'static str` represents
    /// the type name.
    ///
    /// It might be useful to cache data created during the combination within
    /// `self` (e.g. geometric data which is expensive to calculate but useful
    /// for other [`FluxBarrier`] methods). Therefore, this method takes a
    /// `&mut self` reference so that data can be stored within `self`.
    ///
    /// # Examples
    ///
    /// The following examples shows how [`FluxBarrier::combine`] is used within
    /// [`LinCore::new`](crate::core::LinCore::new) /
    /// [`RotCore::new`](crate::core::RotCore::new) on a principal level. The
    /// image below shows the "fb_comp" scenario on the left and the "fb_incomp"
    /// scenario on the right side.
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Comparison compatible and incompatible flux barrier][rot_core_set_fb]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image("rot_core_set_fb", "docs/img/rot_core_set_fb.svg")
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// use stem_core::prelude::*;
    /// use planar_geo::shape::Shape;
    ///
    /// fn fake_new(mut flux_barrier: Box<dyn FluxBarrier>) -> Result<RotCore, stem_core::error::Error> {
    ///     let core: RotCore = RotCoreBuilder {
    ///         air_gap_radius: Length::new::<millimeter>(40.0),
    ///         yoke_radius: Length::new::<millimeter>(15.0),
    ///         axial_length: Length::new::<millimeter>(100.0),
    ///         axial_coil_overhang: Length::new::<millimeter>(0.0),
    ///         skew_angle: 0.0,
    ///         iron_fill_factor: 1.0,
    ///         material: Arc::new(Material::default()),
    ///         pole_pairs: 3,
    ///         air_gap: Box::new(PlainAirGap::default()),
    ///         flux_barrier: None, // No flux barrier at initialization
    ///     }.try_into()?;
    ///
    ///     // In the real new method, the internally cached shape is used.
    ///     let mut shape: Shape = core.shape().to_owned();
    ///
    ///     // Try to insert the flux barriers into the core shape
    ///     let contours = flux_barrier.combine(core.as_core_ref())?;
    ///     for c in contours {
    ///         shape.add_hole(c)?;
    ///     }
    ///     
    ///     return Ok(core);
    /// }
    ///
    /// // Compatible flux barrier
    /// let fb_comp = Spoke1FluxBarrier {
    ///     air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
    ///     yoke_leakage_path_width: Length::new::<millimeter>(1.0),
    ///     relief_path_air_gap_width: Length::new::<millimeter>(4.0),
    ///     magnet_space_width: Length::new::<millimeter>(10.0),
    ///     height_split: Spoke1HeightSplit::ReliefPathWidth(Length::new::<millimeter>(2.0)),
    ///     glue_gap: Length::new::<millimeter>(0.0),
    ///     magnet_material: None,
    ///     cache: None,
    /// };
    /// assert!(fake_new(Box::new(fb_comp)).is_ok());
    ///
    /// // Incompatible flux barrier
    /// let mut fb_incomp = Spoke1FluxBarrier {
    ///     air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
    ///     yoke_leakage_path_width: Length::new::<millimeter>(1.0),
    ///     relief_path_air_gap_width: Length::new::<millimeter>(4.0),
    ///     magnet_space_width: Length::new::<millimeter>(30.0), // Too wide for the core width
    ///     height_split: Spoke1HeightSplit::ReliefPathWidth(Length::new::<
    ///         millimeter,
    ///     >(2.0)),
    ///     glue_gap: Length::new::<millimeter>(0.0),
    ///     magnet_material: None,
    ///     cache: None,
    /// };
    /// assert!(fake_new(Box::new(fb_incomp)).is_err());
    /// ```
    fn combine(&mut self, core: CoreRef<'_>) -> Result<Vec<Contour>, Error>;

    /// Returns the pole coverage of the flux barrier.
    ///
    /// This method implements [`CoreExt::pole_coverage`]. Because it is a
    /// ratio, it should return a value between 0 and 1.
    fn pole_coverage(&self, core: CoreRef<'_>) -> f64;

    /// Returns an iterator over the interior magnet shapes for the given
    /// `core`.
    ///
    /// This method implements [`CoreExt::interior_magnets`] for the different
    /// possible flux barrier types. For example, for a [`Spoke1FluxBarrier`],
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
    /// of [`Spoke1FluxBarrier::interior_magnets`] for an example.
    fn interior_magnets(&self, _core: CoreRef<'_>, _split: bool) -> Magnets {
        // Dummy implementation, to be overwritten.
        return crate::magnets::EvenlyDistributedMagnets::<true>::new(
            0,
            Length::new::<meter>(0.0),
            Vec::new(),
            0.0,
            0,
            0.0,
        )
        .into();
    }

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
    [`V1rFluxBarrier`], but 1 for a [`Spoke1FluxBarrier`]. The number of axial
    subdivisions depends entirely on the flux barrier implementation.

    The [`FluxBarrier::interior_magnets`] returns an iterator over the shapes
    of all interior magnets within `self`. By indexing into this slice with
    [`PositionedMagnetShape::magnet_idx`](crate::magnets::PositionedMagnetShape::magnet_idx),
    the magnet assembly to which a particular shape belongs can be determined.
    For example, if `self.magnet_assemblies().len() == 2` and the
    `PositionedMagnetShape::magnet_idx` of a returned shape is 1, that shape
    belongs to the second magnet assembly in the slice. This can e.g. be used to
    calculate the total mass of all interior magnets (see source code of
    [`CoreExt::mass_interior_magnets`]. When implementing [`FluxBarrier`] for
    an external type, this relation needs to be uphold, because otherwise these
    calculations will return wrong results.

    If the flux barrier does not hold magnets ([`FluxBarrier::interior_magnets`]
    returns an empty iterator), this method can be implemented by simply
    returning an empty slice.

    # Examples

    ```
    use std::f64::consts::FRAC_PI_2;
    use std::sync::Arc;

    use stem_core::prelude::*;

    let spoke1 = Spoke1FluxBarrier {
        magnet_space_width: Length::new::<millimeter>(10.0),
        glue_gap: Length::new::<millimeter>(0.2),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(5.0),
        height_split: Spoke1HeightSplit::ReliefPathWidth(Length::new::<
            millimeter,
        >(2.0)),
    };
    let spoke1_core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(18.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        skew_angle: 0.0,
        air_gap: Box::new(PlainAirGap::default()),
        flux_barrier: Some(Box::new(spoke1)),
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

    let binding = spoke1_core.flux_barrier();
    let spoke1 = binding.as_ref().expect("has flux barrier");
    let sum_spoke1_mags: usize = spoke1.magnet_assemblies(spoke1_core.as_core_ref()).iter().map(|m|m.num_magnets()).sum();
    assert_eq!(sum_spoke1_mags, 1);

    let binding = v1r_core.flux_barrier();
    let v1r = binding.as_ref().expect("has flux barrier");
    let sum_v1r_mags: usize = v1r.magnet_assemblies(v1r_core.as_core_ref()).iter().map(|m|m.num_magnets()).sum();
    assert_eq!(sum_v1r_mags, 2);
    ```
     */
    fn magnet_assemblies(&self, core: CoreRef<'_>) -> &[MagnetAssembly];

    /// Returns the offset of the first positive d-axis against the "start" of
    /// the core in electrical radians.
    ///
    /// This method implements [`CoreExt::d_axis_offset`], see its documentation
    /// for details. The default implementation returns [`FRAC_PI_2`], although
    /// it might be necessary to adjust this depending on the flux barrier
    /// (compare [`Spoke1FluxBarrier`] and [`V1rFluxBarrier`] in the image
    /// below).
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![d-Axis offset from core start][d_axis_offset]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image("d_axis_offset", "docs/img/d_axis_offset.svg")
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    fn d_axis_offset(&self, _core: CoreRef<'_>) -> f64 {
        return FRAC_PI_2;
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
    use stem_magnet::planar_geo::Transformation;
    use uom::si::length::meter;

    let offset_slot = if air_gap.starts_in_slot_middle {
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

/// Returns the distance from the given point to the q-axis.
pub(crate) fn dist_to_q_axis(pt: [f64; 2], pole_pairs: u16) -> f64 {
    let x = pt[0];
    let y = pt[1];

    // Define the q-axis line from the origin to the air gap as an equation
    // ax + by + c
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
        approxim::assert_abs_diff_eq!(closest_pt[0], 0.0449, epsilon = 1e-3);
        approxim::assert_abs_diff_eq!(closest_pt[1], 0.0157, epsilon = 1e-3);
    }
}
