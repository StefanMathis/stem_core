/*!
This module provides the [`StraightIndentsAirGap`] struct which represents an
air gap with straight indents, possibly sunken into or extruded from the core.
Additionally, the module contains the [`PolygonAirGapBuilder`] builder struct
which can be used to create [`StraightIndentsAirGap`]s with a polygon shape.
 */
#![cfg_attr(feature = "doc-images", doc = "")]
#![cfg_attr(
    feature = "doc-images",
    doc = "![Linear and rotary core with a straight indents air gap][lin_and_rot_core_straight_indents.svg]"
)]
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("lin_and_rot_core_straight_indents.svg", "docs/img/lin_and_rot_core_straight_indents.svg"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/*!
[`StraightIndentsAirGap`] implements the [`AirGap`] trait and can therefore be
used to build magnetic cores. See the struct docstring for more.
*/

use std::{
    f64::consts::{FRAC_PI_2, PI, TAU},
    num::NonZero,
};

use crate::{
    magnets::{surface_magnet_assembly_shapes_lin, surface_magnet_assembly_shapes_rot},
    planar_geo,
};
use compare_variables::compare_variables;
use planar_geo::prelude::*;
use stem_magnet::prelude::*;
use stem_slot::stem_coil_layout::CoilLayout;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    air_gap::AirGap,
    core::{CoreExt, CoreRef, LinCore, RotCore},
    error::Error,
    magnets::{Magnets, MagnetsPeriodic},
    winding_zones::{WindingZones, WindingZonesPeriodic},
};

/**
An air gap with straight indents for mounting magnets with planar backs like
e.g., [`BlockMagnet`]s.

This air gap features one or more indents per pole which can be used for
mounting magnets with planar surfaces. This is especially interesting for
[`RotCore`]s which otherwise require more expensive arced magnets. The indents
can extrude into the air gap or be sunken into the core. The image below shows
the extrusion case for a linear and the sunken case for a rotary motor (same
indent width, two indents per pole).
*/
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Linear and rotary core with a straight indents air gap][lin_and_rot_core_straight_indents]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image(
        "lin_and_rot_core_straight_indents",
        "docs/img/lin_and_rot_core_straight_indents.svg"
    )
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

_This image was produced with `examples/air_gap_plots.rs`._

A [`StraightIndentsAirGap`] cannot be wound, so [`AirGap::slots`] always
returns zero.

When used in a linear motor, [`CoreExt::d_axis_offset`] must be `pi/2`, which it
is by default if the core has no flux barrier. If the core does have a flux
barrier, that value depends on the implementation of
[`FluxBarrier::d_axis_offset`](crate::flux_barrier::FluxBarrier::d_axis_offset).
If the d-axis offset isn't `pi/2`, trying to [`combine`](AirGap::combine) the
core and the air gap will return an error.

The effect of the indents on the air gap field is neglected in the analytical
approximations like [`CoreExt::carter_factor`] or [`CoreExt::slotting_ordinals`].

# Dimensions

The image below shows the definition of the following geometric variables
for the example of a [`RotCore`]. For a [`LinCore`], the geometric relations are
much simpler and only the first two values are needed.
- [`indent_width`](StraightIndentsAirGap::indent_width)
- [`indent_depth`](StraightIndentsAirGap::indent_depth)
- [`indent_opening_angle`](StraightIndentsAirGap::indent_opening_angle)
- [`indent_center_radius`](StraightIndentsAirGap::indent_center_radius)
- [`indent_corner_radius`](StraightIndentsAirGap::indent_corner_radius)

*/
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Dimensions of a StraightIndentsAirGap][cad_straight_indents_air_gap_dims]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image(
        "cad_straight_indents_air_gap_dims",
        "docs/img/cad_straight_indents_air_gap_dims.svg"
    )
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

# Constructors

Besides using the struct constructor directly, a [`PolygonAirGapBuilder`] can be
used to build a [`StraightIndentsAirGap`] for a [`RotCore`] where the entire air
gap surface is covered by indents (i.e., the indents of one pole directly touch
that of the next one, resulting in a symmetric polygon). A
[`PolygonAirGapBuilder`] can be fallibly converted into a
[`StraightIndentsAirGap`] via [`TryFrom`].

# Serialization and deserialization

A [`StraightIndentsAirGap`] can be deserialized directly from its fields or
alternatively from a [`PolygonAirGapBuilder`].

```
use stem_core::prelude::*;
use yaml_serde;

let str = indoc::indoc! {"
num_segments: 2
indent_width: 10 mm
indent_depth: -2 mm
indents_per_pole: 2
"};
assert!(yaml_serde::from_str::<StraightIndentsAirGap>(&str).is_ok());

// Deserialize from PolygonAirGapBuilder
let str = indoc::indoc! {"
num_segments: 2
indents_per_pole: 2
pole_pairs: 3
air_gap_radius: 100 mm
"};
assert!(yaml_serde::from_str::<StraightIndentsAirGap>(&str).is_ok());
```
 */
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct StraightIndentsAirGap {
    /// Number of segments of the core.
    ///
    /// A [`StraightIndentsAirGap`] can be segmented, but not continuously
    /// skewed. It is possible to mount a magnet assembly on a segmented
    /// core if the assembly is segmented itself in the same way (i.e
    /// [`MagnetAssembly::num_axial`] matches [`CoreExt::num_segments`] and
    /// [`MagnetAssembly::length`] matches [`CoreExt::axial_length`]), but a
    /// continuously skewed core does not provide a planar surface. Hence,
    /// `num_segments` has the [`NonZero<usize>`] type.
    pub num_segments: NonZero<usize>,
    /// Width of a single indent. Must not be negative (`indent_width >= 0 m`);
    /// otherwise [`StraightIndentsAirGap::combine`] will return an error.
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    pub indent_width: Length,
    /// Depth of a single indent. A positive value means the indent is sunken
    /// into the core, a negative indent means that the indent protrudes from
    /// the core.
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    pub indent_depth: Length,
    /// Number of indents per pole.
    pub indents_per_pole: usize,
}

impl StraightIndentsAirGap {
    /// Returns the opening angle of a single indent in radians.
    ///
    /// This angle is measured from the corners of a single indent with the
    /// origin of the [`RotCore`] being the angle center. See the drawing below.
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Dimensions of a StraightIndentsAirGap][cad_straight_indents_air_gap_dims]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "cad_straight_indents_air_gap_dims",
            "docs/img/cad_straight_indents_air_gap_dims.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    ///
    /// Besides the `air_gap_radius`, it is also necessary to specify whether
    /// the core is an inner or an outer core, because a positive indent depth
    /// should always sink the indent into the core and therefore its sign needs
    /// to change depending on `is_outer` (see drawing). These arguments can be
    /// read from a [`RotCore`] via [`RotCore::air_gap_radius`] and
    /// [`RotCore::is_outer`].
    ///
    /// # Examples
    ///
    /// ```
    /// use approxim::assert_abs_diff_eq;
    ///
    /// use stem_core::prelude::*;
    ///
    /// let air_gap = StraightIndentsAirGap {
    ///     num_segments: 2.try_into().expect("not zero"),
    ///     indent_width: Length::new::<millimeter>(10.0),
    ///     indent_depth: Length::new::<millimeter>(2.0),
    ///     indents_per_pole: 2,
    /// };
    /// assert_abs_diff_eq!(air_gap.indent_opening_angle(Length::new::<millimeter>(60.0), true), 0.161, epsilon = 1e-3);
    /// assert_abs_diff_eq!(air_gap.indent_opening_angle(Length::new::<millimeter>(60.0), false), 0.172, epsilon = 1e-3);
    /// ```
    pub fn indent_opening_angle(&self, air_gap_radius: Length, is_outer: bool) -> f64 {
        use uom::typenum::P2;

        let middle_radius = self.indent_corner_radius(air_gap_radius, is_outer);

        let adjusted_middle_radius =
            (middle_radius.powi(P2::new()) + (self.indent_width / 2.0).powi(P2::new())).sqrt();

        return 2.0 * f64::from(self.indent_width / (2.0 * adjusted_middle_radius)).asin();
    }

    /// Returns the radius at the indent center.
    ///
    /// All indents of the air gap contour of a [`RotCore`] with a
    /// [`StraightIndentsAirGap`] lay on a common circle which shares its center
    /// with that of the [`RotCore`]. The `indent_center_radius` is the radius
    /// at an indent center, see the drawing below.
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Dimensions of a StraightIndentsAirGap][cad_straight_indents_air_gap_dims]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "cad_straight_indents_air_gap_dims",
            "docs/img/cad_straight_indents_air_gap_dims.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    ///
    /// Besides the `air_gap_radius`, it is also necessary to specify whether
    /// the core is an inner or an outer core, because a positive indent depth
    /// should always sink the indent into the core and therefore its sign needs
    /// to change depending on `is_outer` (see drawing). These arguments can be
    /// read from a [`RotCore`] via [`RotCore::air_gap_radius`] and
    /// [`RotCore::is_outer`].
    ///
    /// # Examples
    ///
    /// ```
    /// use approxim::assert_abs_diff_eq;
    ///
    /// use stem_core::prelude::*;
    ///
    /// let ag1 = StraightIndentsAirGap {
    ///     num_segments: 2.try_into().expect("not zero"),
    ///     indent_width: Length::new::<millimeter>(10.0),
    ///     indent_depth: Length::new::<millimeter>(2.0),
    ///     indents_per_pole: 2,
    /// };
    /// assert_abs_diff_eq!(ag1.indent_center_radius(Length::new::<millimeter>(60.0), true).get::<millimeter>(), 61.791, epsilon = 1e-3);
    /// assert_abs_diff_eq!(ag1.indent_center_radius(Length::new::<millimeter>(60.0), false).get::<millimeter>(), 57.791, epsilon = 1e-3);
    ///
    /// let ag2 = StraightIndentsAirGap {
    ///     num_segments: 2.try_into().expect("not zero"),
    ///     indent_width: Length::new::<millimeter>(10.0),
    ///     indent_depth: Length::new::<millimeter>(0.0),
    ///     indents_per_pole: 2,
    /// };
    /// assert_abs_diff_eq!(ag2.indent_center_radius(Length::new::<millimeter>(60.0), true).get::<millimeter>(), 59.791, epsilon = 1e-3);
    /// assert_abs_diff_eq!(ag2.indent_center_radius(Length::new::<millimeter>(60.0), false).get::<millimeter>(), 59.791, epsilon = 1e-3);
    ///
    /// let ag3 = StraightIndentsAirGap {
    ///     num_segments: 2.try_into().expect("not zero"),
    ///     indent_width: Length::new::<millimeter>(10.0),
    ///     indent_depth: Length::new::<millimeter>(-2.0),
    ///     indents_per_pole: 2,
    /// };
    /// assert_abs_diff_eq!(ag3.indent_center_radius(Length::new::<millimeter>(60.0), true).get::<millimeter>(), 57.791, epsilon = 1e-3);
    /// assert_abs_diff_eq!(ag3.indent_center_radius(Length::new::<millimeter>(60.0), false).get::<millimeter>(), 61.791, epsilon = 1e-3);
    /// ```
    pub fn indent_center_radius(&self, air_gap_radius: Length, is_outer: bool) -> Length {
        use uom::typenum::P2;
        let delta = air_gap_radius
            - 0.5
                * (4.0 * air_gap_radius.powi(P2::new()) - self.indent_width.powi(P2::new())).sqrt();
        return air_gap_radius + self.indent_depth_signed(is_outer) - delta;
    }

    /// Returns the radius at the indent center.
    ///
    /// All indents of the air gap contour of a [`RotCore`] with a
    /// [`StraightIndentsAirGap`] lay on a common circle which shares its center
    /// with that of the [`RotCore`]. The `indent_corner_radius` is the radius
    /// at the indent corners, see the drawing below. If
    /// [`StraightIndentsAirGap::indent_depth`] is zero, this value is equal to
    /// [`RotCore::air_gap_radius`].
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Dimensions of a StraightIndentsAirGap][cad_straight_indents_air_gap_dims]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "cad_straight_indents_air_gap_dims",
            "docs/img/cad_straight_indents_air_gap_dims.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    ///
    /// Besides the `air_gap_radius`, it is also necessary to specify whether
    /// the core is an inner or an outer core, because a positive indent depth
    /// should always sink the indent into the core and therefore its sign needs
    /// to change depending on `is_outer` (see drawing). These arguments can be
    /// read from a [`RotCore`] via [`RotCore::air_gap_radius`] and
    /// [`RotCore::is_outer`].
    ///
    /// # Examples
    ///
    /// ```
    /// use approxim::assert_abs_diff_eq;
    ///
    /// use stem_core::prelude::*;
    ///
    /// let ag1 = StraightIndentsAirGap {
    ///     num_segments: 2.try_into().expect("not zero"),
    ///     indent_width: Length::new::<millimeter>(10.0),
    ///     indent_depth: Length::new::<millimeter>(2.0),
    ///     indents_per_pole: 2,
    /// };
    /// assert_abs_diff_eq!(ag1.indent_corner_radius(Length::new::<millimeter>(60.0), true).get::<millimeter>(), 61.993, epsilon = 1e-3);
    /// assert_abs_diff_eq!(ag1.indent_corner_radius(Length::new::<millimeter>(60.0), false).get::<millimeter>(), 58.007, epsilon = 1e-3);
    /// assert!(ag1.indent_corner_radius(Length::new::<millimeter>(60.0), true) > ag1.indent_center_radius(Length::new::<millimeter>(60.0), true));
    /// assert!(ag1.indent_corner_radius(Length::new::<millimeter>(60.0), false) > ag1.indent_center_radius(Length::new::<millimeter>(60.0), false));
    ///
    /// let ag2 = StraightIndentsAirGap {
    ///     num_segments: 2.try_into().expect("not zero"),
    ///     indent_width: Length::new::<millimeter>(10.0),
    ///     indent_depth: Length::new::<millimeter>(0.0),
    ///     indents_per_pole: 2,
    /// };
    /// assert_abs_diff_eq!(ag2.indent_corner_radius(Length::new::<millimeter>(60.0), true).get::<millimeter>(), 60.0, epsilon = 1e-3);
    /// assert_abs_diff_eq!(ag2.indent_corner_radius(Length::new::<millimeter>(60.0), false).get::<millimeter>(), 60.0, epsilon = 1e-3);
    /// assert!(ag2.indent_corner_radius(Length::new::<millimeter>(60.0), true) > ag2.indent_center_radius(Length::new::<millimeter>(60.0), true));
    /// assert!(ag2.indent_corner_radius(Length::new::<millimeter>(60.0), false) > ag2.indent_center_radius(Length::new::<millimeter>(60.0), false));
    ///
    /// let ag3 = StraightIndentsAirGap {
    ///     num_segments: 2.try_into().expect("not zero"),
    ///     indent_width: Length::new::<millimeter>(10.0),
    ///     indent_depth: Length::new::<millimeter>(-2.0),
    ///     indents_per_pole: 2,
    /// };
    /// assert_abs_diff_eq!(ag3.indent_corner_radius(Length::new::<millimeter>(60.0), true).get::<millimeter>(), 58.007, epsilon = 1e-3);
    /// assert_abs_diff_eq!(ag3.indent_corner_radius(Length::new::<millimeter>(60.0), false).get::<millimeter>(), 61.993, epsilon = 1e-3);
    /// assert!(ag3.indent_corner_radius(Length::new::<millimeter>(60.0), true) > ag3.indent_center_radius(Length::new::<millimeter>(60.0), true));
    /// assert!(ag3.indent_corner_radius(Length::new::<millimeter>(60.0), false) > ag3.indent_center_radius(Length::new::<millimeter>(60.0), false));
    /// ```
    pub fn indent_corner_radius(&self, air_gap_radius: Length, is_outer: bool) -> Length {
        use uom::typenum::P2;

        // Use the Pythagorean theorem to determine the indent corner radius
        return (self
            .indent_center_radius(air_gap_radius, is_outer)
            .powi(P2::new())
            + (0.5 * self.indent_width).powi(P2::new()))
        .sqrt();
    }

    /// Helper function to adjust the sign of self.indent_depth depending on
    /// whether a rotary core is an inner or an outer core.
    fn indent_depth_signed(&self, is_outer: bool) -> Length {
        return self.indent_depth * (is_outer as i32 as f64)
            - self.indent_depth * (!is_outer as i32 as f64);
    }

    /// Returns the core shape for a linear core, if the combination of `self`
    /// and `core` results in a valid shape.
    fn shape_lin(&self, core: &LinCore) -> Result<Shape, Error> {
        use compare_variables::{Comparison, ComparisonOperator, ComparisonValue};

        let zero_length = Length::new::<meter>(0.0);
        compare_variables!(self.indent_width > zero_length)?;

        // A linear core can only be built if the d-axis offset is pi/2
        if approxim::relative_ne!(core.d_axis_offset(), FRAC_PI_2) {
            return Err(Comparison::new(
                ComparisonValue::new(core.d_axis_offset(), Some("core.d_axis_offset")),
                ComparisonOperator::Inequal,
                ComparisonValue::new(FRAC_PI_2, None),
                ComparisonOperator::Equal,
                None,
            )
            .into());
        }

        let indent_depth = self.indent_depth.get::<meter>();
        let indent_width = self.indent_width.get::<meter>();
        let width = core.width().get::<meter>();
        let height = core.height().get::<meter>();

        if indent_depth == 0.0 {
            let contour = Contour::rectangle([0.0, 0.0], [width, height]);
            return Shape::try_from(contour).map_err(From::from);
        }

        let poles = 2 * usize::from(core.pole_pairs());
        let total_indent_width = indent_width * self.indents_per_pole as f64;
        let width_per_pole = width / poles as f64;

        let mut ps = Polysegment::with_capacity(4 + poles);
        let ls = LineSegment::new([width, height], [0.0, height])?;
        ps.push_back(ls.into());
        ps.extend_back([0.0, 0.0]);

        let mut offset = 0.5 * (width_per_pole - total_indent_width);
        for _ in 0..poles {
            ps.extend_back([offset, 0.0]);
            ps.extend_back([offset, indent_depth]);
            ps.extend_back([offset + total_indent_width, indent_depth]);
            ps.extend_back([offset + total_indent_width, 0.0]);
            offset += width_per_pole;
        }

        ps.extend_back([width, 0.0]);

        return Shape::try_from(ps).map_err(From::from);
    }

    /// Returns the core shape for a rotary core, if the combination of `self`
    /// and `core` results in a valid shape.
    fn shape_rot(&self, core: &RotCore) -> Result<Shape, Error> {
        let zero_length = Length::new::<meter>(0.0);
        compare_variables!(self.indent_width > zero_length)?;

        let indent_center_radius = self
            .indent_center_radius(core.air_gap_radius(), core.is_outer())
            .get::<meter>();
        let indent_corner_radius = self
            .indent_corner_radius(core.air_gap_radius(), core.is_outer())
            .get::<meter>();
        let indent_width = self.indent_width.get::<meter>();
        let indent_depth = if core.is_outer() {
            self.indent_depth.get::<meter>()
        } else {
            -self.indent_depth.get::<meter>()
        };
        let ag_radius = core.air_gap_radius().get::<meter>();

        // Build the first indent chain, starting with the first indent as a vertical
        // line symmetric about the x-axis
        let mut air_gap = Polysegment::with_capacity(
            (self.indents_per_pole + 2) * 2 * usize::from(core.pole_pairs()),
        );

        // Connection indent <-> air gap. If the indent depth is zero, this
        // line does not exist.
        if let Ok(ls) = LineSegment::new(
            [indent_center_radius - indent_depth, -0.5 * indent_width],
            [indent_center_radius, -0.5 * indent_width],
        ) {
            air_gap.push_back(ls.into());
        }

        let indent_angle = 2.0 * (0.5 * indent_width / indent_corner_radius).asin();
        let angle_between_indents = PI - indent_angle;

        // Add indents
        for i in 0..self.indents_per_pole {
            let start = air_gap
                .back()
                .map(|s| s.stop())
                .unwrap_or([indent_center_radius, -0.5 * indent_width]);

            let angle = i as f64 * (PI - angle_between_indents) + FRAC_PI_2;

            // Should never fail, because indent_width is assured to be greater than zero.
            let ls = LineSegment::from_start_angle_length(start, angle, indent_width)?;
            air_gap.push_back(ls.into());
        }

        // Add the connection back to the air gap
        let start = air_gap
            .back()
            .map(|s| s.stop())
            .ok_or(Error::IncompatibleToRotCore("could not create indents"))?;

        // Can fail, if indent depth is zero - this is completely fine.
        let angle = (PI - angle_between_indents) * (self.indents_per_pole - 1) as f64 + PI;
        if let Ok(ls) = LineSegment::from_start_angle_length(start, angle, indent_depth) {
            air_gap.push_back(ls.into());
        }

        // Calculate the angle which is covered by the entire polysegment. The
        // reference radius is ag_radius and the reference center is the origin.
        let start = air_gap
            .front()
            .map(|s| s.start())
            .ok_or(Error::IncompatibleToRotCore("could not create indents"))?;
        let start_angle = start[1].atan2(start[0]);

        let stop = air_gap
            .back()
            .map(|s| s.stop())
            .ok_or(Error::IncompatibleToRotCore("could not create indents"))?;
        let stop_angle = stop[1].atan2(stop[0]);

        let indents_angle = stop_angle - start_angle;
        let angle_per_pole = TAU / core.poles() as f64;
        let sweep_angle = angle_per_pole - indents_angle;
        if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
            [0.0, 0.0],
            ag_radius,
            stop_angle,
            sweep_angle,
        ) {
            air_gap.push_back(arc.into());
        }

        // Rotate the air gap so the q-axis is on the positive x-axis
        air_gap.rotate(
            [0.0, 0.0],
            0.5 * sweep_angle - start_angle
                + (core.d_axis_offset() - FRAC_PI_2) / core.pole_pairs() as f64,
        );

        // Repeat the pattern for all poles
        air_gap.rotational_pattern([0.0, 0.0], angle_per_pole, usize::from(core.poles()) - 1);

        // Create the yoke circle
        let yoke = ArcSegment::circle([0.0, 0.0], core.yoke_radius().get::<meter>())?;

        if core.is_outer() {
            return Shape::new(vec![yoke.into(), air_gap.into()]).map_err(From::from);
        } else {
            return Shape::new(vec![air_gap.into(), yoke.into()]).map_err(From::from);
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl AirGap for StraightIndentsAirGap {
    fn num_segments(&self, _core: CoreRef<'_>) -> usize {
        self.num_segments.into()
    }

    fn surface_magnets(
        &self,
        magnet_assembly: &MagnetAssembly,
        core: CoreRef<'_>,
        split: bool,
    ) -> Magnets {
        match core {
            CoreRef::Lin(_) => {
                let mut magnets = surface_magnet_assembly_shapes_lin(
                    magnet_assembly,
                    split,
                    Some(self.indent_width),
                );
                magnets.iter_mut().for_each(|m| {
                    m.translate([0.0, self.indent_depth.get::<meter>()]);
                });

                MagnetsPeriodic::<true>::new(
                    core.air_gap_length(),
                    magnets,
                    core.poles().into(),
                    core.d_axis_offset(),
                )
                .into()
            }
            CoreRef::Rot(rot_core) => {
                let indent_center_radius =
                    self.indent_center_radius(rot_core.air_gap_radius(), rot_core.is_outer());
                let indent_corner_radius =
                    self.indent_corner_radius(rot_core.air_gap_radius(), rot_core.is_outer());

                let magnet_coverage = 2.0
                    * (self.indent_width / (2.0 * indent_corner_radius))
                        .get::<ratio>()
                        .asin();

                let magnets = surface_magnet_assembly_shapes_rot(
                    magnet_assembly,
                    split,
                    indent_center_radius,
                    rot_core.is_outer(),
                    Some(magnet_coverage),
                );
                MagnetsPeriodic::<false>::new(
                    indent_center_radius * TAU,
                    magnets,
                    core.poles().into(),
                    core.d_axis_offset(),
                )
                .into()
            }
        }
    }

    fn winding_zones(&self, _core: CoreRef<'_>, _coil_layout: &CoilLayout) -> WindingZones {
        // Placeholder, since the StraightIndentsAirGap can't be wound
        WindingZonesPeriodic::<Contour, true>::no_slots().into()
    }

    fn slots(&self, _core: CoreRef<'_>) -> u16 {
        0
    }

    fn combine(&mut self, core: CoreRef<'_>) -> Result<Shape, Error> {
        let zero_length = super::zero_length();
        compare_variables!(self.indent_width >= zero_length)?;
        match core {
            CoreRef::Lin(lin_core) => self.shape_lin(lin_core),
            CoreRef::Rot(rot_core) => self.shape_rot(rot_core),
        }
    }

    fn slot_opening_factor(&self, _core: CoreRef<'_>, _mech_ordinal: i32) -> f64 {
        0.0
    }

    fn carter_factor(&self, _core: CoreRef<'_>, _air_gap_width: Length) -> f64 {
        return 1.0;
    }

    fn slot(&self, _core: CoreRef<'_>) -> Option<&dyn stem_slot::slot::Slot> {
        return None;
    }
}

/**
A builder struct for a [`StraightIndentsAirGap`].

The purpose of this builder is the creation of[`StraightIndentsAirGap`]s for
[`RotCore`]s whose air-gap contour is a regular polygon. The number of sides is
defined as 2 times [`PolygonAirGapBuilder::pole_pairs`] times
[`PolygonAirGapBuilder::indents_per_pole`], and the
[`PolygonAirGapBuilder::air_gap_radius`] is the circumradius of the polygon:

`indent_width = 2 * air_gap_radius * sin(PI / (2 * pole_pairs * indents_per_pole))`

The [`indent_depth`](StraightIndentsAirGap::indent_depth) is always zero,
meaning that [`StraightIndentsAirGap::indent_corner_radius`] equals the
[`PolygonAirGapBuilder::air_gap_radius`] / circumradius of the polygon. The
following image shows the resulting air gap contours for an outer and an inner
core respectively:
*/
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Inner and outer cores created with PolygonAirGapBuilder][polygon_cores]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image("polygon_cores", "docs/img/polygon_cores.svg")
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

_This image was produced with `examples/air_gap_plots.rs`._

A [`PolygonAirGapBuilder`] can be fallibly converted into a
[`StraightIndentsAirGap`] with [`TryFrom`] / [`TryInto`]:

```
use approxim::assert_abs_diff_eq;
use stem_core::prelude::*;

let builder = PolygonAirGapBuilder {
    num_segments: 2.try_into().expect("not zero"),
    indents_per_pole: 2,
    pole_pairs: 3,
    air_gap_radius: Length::new::<millimeter>(60.0),
};

let ag = StraightIndentsAirGap::try_from(builder).expect("valid data");
assert_abs_diff_eq!(ag.indent_width.get::<millimeter>(), 31.058, epsilon = 1e-3);
```
The conversion fails if the calculated `indent_width` is negative (i.e.,
[`PolygonAirGapBuilder::air_gap_radius`] is negative).

As shown in the docstring of [`StraightIndentsAirGap`], it is also possible to
deserialize a [`StraightIndentsAirGap`] directly from the serialized
representation of this struct.
*/
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct PolygonAirGapBuilder {
    /// Number of segments of the resulting core. See
    /// [`StraightIndentsAirGap::num_segments`].
    pub num_segments: NonZero<usize>,
    /// Number of indents per pole (see
    /// [`StraightIndentsAirGap::indents_per_pole`]).
    ///
    /// The number of sides of the regular polygon is `2 * pole_pairs *
    /// indents_per_pole`.
    pub indents_per_pole: usize,
    /// Number of pole pairs of the core.
    ///
    /// The number of sides of the regular polygon is `2 * pole_pairs *
    /// indents_per_pole`.
    pub pole_pairs: u16,
    /// Circumradius of the polygon.
    ///
    /// This value should be equal to the [`RotCore::air_gap_radius`] of the
    /// resulting core to create the regular polygon air gap contour. It must
    /// be positive, otherwise the conversion into a [`StraightIndentsAirGap`]
    /// will fail.
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub air_gap_radius: Length,
}

impl TryFrom<PolygonAirGapBuilder> for StraightIndentsAirGap {
    type Error = crate::error::Error;

    fn try_from(value: PolygonAirGapBuilder) -> Result<Self, Self::Error> {
        // Calculate the side length of the regular polygon
        let sides = 2.0 * value.pole_pairs as f64 * value.indents_per_pole as f64;
        let indent_width = 2.0 * value.air_gap_radius * (std::f64::consts::PI / sides).sin();

        let zero_length = super::zero_length();
        compare_variables!(indent_width >= zero_length)?;

        return Ok(StraightIndentsAirGap {
            num_segments: value.num_segments,
            indent_width,
            indent_depth: super::zero_length(),
            indents_per_pole: value.indents_per_pole,
        });
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for StraightIndentsAirGap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct StraightIndentsAirGapsBuilder {
            num_segments: NonZero<usize>,
            #[serde(deserialize_with = "deserialize_quantity")]
            indent_width: Length,
            #[serde(deserialize_with = "deserialize_quantity")]
            indent_depth: Length,
            indents_per_pole: usize,
        }

        #[derive(deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError)]
        enum AirGapEnum {
            StraightIndentsAirGap(StraightIndentsAirGapsBuilder),
            PolygonAirGapBuilder(PolygonAirGapBuilder),
        }

        let ag = AirGapEnum::deserialize(deserializer)?;
        match ag {
            AirGapEnum::StraightIndentsAirGap(ag) => {
                let indent_width = ag.indent_width;
                let zero_length = super::zero_length();
                compare_variables!(indent_width >= zero_length)
                    .map_err(serde::de::Error::custom)?;
                Ok(StraightIndentsAirGap {
                    num_segments: ag.num_segments,
                    indent_width: indent_width,
                    indent_depth: ag.indent_depth,
                    indents_per_pole: ag.indents_per_pole,
                })
            }

            AirGapEnum::PolygonAirGapBuilder(ag) => ag.try_into().map_err(serde::de::Error::custom),
        }
    }
}
