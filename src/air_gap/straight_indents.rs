use std::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::{magnets::PositionedMagnetShape, planar_geo};
use compare_variables::compare_variables;
use num::Integer;
use planar_geo::prelude::*;
use stem_magnet::assembly::MagnetAssembly;
use stem_slot::{coil_layout::CoilLayout, prelude::stem_material::prelude::*};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    air_gap::AirGap,
    core::{CoreExt, CoreRef, LinCore, RotCore},
    error::Error,
    magnets::{EvenlyDistributedMagnets, Magnets},
    winding_zones::{WindingZones, WindingZonesEqSpaced},
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct StraightIndentsAirGap {
    pub num_segments: usize,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub indent_width: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub indent_depth: Length, // Depth of the indent. A negative depth leads to an extrusion
    pub indents_per_pole: usize,
}

impl StraightIndentsAirGap {
    pub fn new(
        num_segments: usize,
        indent_width: Length,
        indent_depth: Length,
        indents_per_pole: usize,
    ) -> Self {
        return Self {
            num_segments,
            indent_width,
            indent_depth,
            indents_per_pole,
        };
    }
}

impl StraightIndentsAirGap {
    pub fn indent_width(&self) -> Length {
        return self.indent_width;
    }

    pub fn indent_depth(&self) -> Length {
        return self.indent_depth;
    }

    pub fn indents_per_pole(&self) -> usize {
        return self.indents_per_pole;
    }

    /// The inner radius is the radius of the larges circle which still fits
    /// inside the air gap contour.
    pub fn inner_air_gap_radius(&self, core: &RotCore) -> Length {
        use uom::typenum::P2;

        // Calculate the radius of the inner circle enscribed by the slot contour
        let middle_radius_wo_depth = (core.air_gap_radius().powi(P2::new())
            - (self.indent_width() / 2.0).powi(P2::new()))
        .sqrt();
        if core.is_outer() {
            return middle_radius_wo_depth + self.indent_depth;
        } else {
            return middle_radius_wo_depth - self.indent_depth;
        }
    }

    /// Opening angle of an indent (measured from the edges of the indent)
    pub fn opening_angle_indent(&self, core: &RotCore) -> f64 {
        use uom::typenum::P2;

        let middle_radius = self.inner_air_gap_radius(core);

        let adjusted_middle_radius =
            (middle_radius.powi(P2::new()) + (self.indent_width() / 2.0).powi(P2::new())).sqrt();

        return 2.0 * f64::from(self.indent_width() / (2.0 * adjusted_middle_radius)).asin();
    }

    /// Return the radius at the center of the indent.
    ///
    /// TODO: Drawing
    pub fn indent_center_radius(&self, air_gap_radius: Length, is_outer: bool) -> Length {
        // Read out available local information and assign shorter variable
        // names to improve readability
        let indent_width = self.indent_width.get::<meter>();
        let indent_depth = if is_outer {
            self.indent_depth.get::<meter>()
        } else {
            -self.indent_depth.get::<meter>()
        };
        let ag_radius = air_gap_radius.get::<meter>();

        // Height of the circular segment using indent_width as the sekant / chord
        let circ_seg_height =
            ag_radius - 0.5 * (4.0 * ag_radius.powi(2) - indent_width.powi(2)).sqrt();

        // Radius at the indent_center
        let indent_center_radius = ag_radius + indent_depth - circ_seg_height;

        return Length::new::<meter>(indent_center_radius);
    }

    /// Return the radius at the indent corner.
    ///
    /// If [`StraightIndentsAirGap::indent_depth`] is zero, this value is equal
    /// to `air_gap_radius`.
    ///
    /// TODO: Drawing
    pub fn indent_corner_radius(&self, air_gap_radius: Length, is_outer: bool) -> Length {
        // Read out available local information and assign shorter variable
        // names to improve readability
        let indent_width = self.indent_width.get::<meter>();
        let indent_depth = if is_outer {
            self.indent_depth.get::<meter>()
        } else {
            -self.indent_depth.get::<meter>()
        };
        let ag_radius = air_gap_radius.get::<meter>();

        // Half the opening angle of the indent, measured from the connection of
        // the indent extrusion to the indent center at air gap radius
        let alpha = (0.5 * indent_width / ag_radius).asin();
        let k = indent_depth + ag_radius * alpha.cos();

        // Radius at the indent_center
        let indent_corner_radius = ((0.5 * indent_width).powi(2) + k.powi(2)).sqrt();

        return Length::new::<meter>(indent_corner_radius);
    }

    /// Returns the core shape for a linear core, if the combination of `self`
    /// and `core` results in a valid shape.
    fn shape_lin(&self, core: &LinCore) -> Result<Shape, Error> {
        let zero_length = Length::new::<meter>(0.0);
        compare_variables!(self.indent_width > zero_length)?;

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
            ps.extend_back([offset, -indent_depth]);
            ps.extend_back([offset + total_indent_width, -indent_depth]);
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
        air_gap.rotate([0.0, 0.0], 0.5 * sweep_angle - start_angle);

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
        self.num_segments
    }

    fn surface_magnets(
        &self,
        magnet_assembly: &MagnetAssembly,
        core: CoreRef<'_>,
        split: bool,
    ) -> Magnets {
        let magnet_width = magnet_assembly.magnet().width();
        let gap_between_partial_magnets = self.indent_width - magnet_width;
        let num_tangential = magnet_assembly.num_tangential();
        let mut magnet_shapes: Vec<PositionedMagnetShape> = if split {
            magnet_assembly
                .magnet()
                .north_south_shapes()
                .into_iter()
                .enumerate()
                .map(|(i, m)| PositionedMagnetShape {
                    shape: m.into_owned(),
                    is_north: i.is_even(),
                    magnet_idx: 0,
                })
                .collect()
        } else {
            vec![PositionedMagnetShape {
                shape: magnet_assembly.magnet().shape().into_owned(),
                is_north: true,
                magnet_idx: 0,
            }]
        };

        match core {
            CoreRef::Lin(lin_core) => {
                let magnet_coverage =
                    BoundingBox::from_bounded_entities(magnet_shapes.iter().map(|p| &p.shape))
                        .map(|m| m.width() + gap_between_partial_magnets.get::<meter>())
                        .unwrap_or(0.0);

                magnet_shapes.iter_mut().for_each(|s| {
                    s.line_reflection([0.0, 0.0], [1.0, 0.0]);
                    s.translate([0.0, -self.indent_depth.get::<meter>()]);
                });

                return EvenlyDistributedMagnets::<true>::new(
                    core.poles().into(),
                    lin_core.air_gap_length(),
                    magnet_shapes,
                    magnet_coverage,
                    num_tangential,
                )
                .into();
            }
            CoreRef::Rot(rot_core) => {
                let indent_center_radius =
                    self.indent_center_radius(rot_core.air_gap_radius(), rot_core.is_outer());

                // Needs to be done before rotating the magnet shapes!
                let magnet_coverage = crate::magnets::pole_coverage_angle(
                    magnet_shapes.iter().map(|p| &p.shape),
                    indent_center_radius.get::<meter>(),
                    gap_between_partial_magnets,
                );

                if rot_core.is_outer() {
                    for s in magnet_shapes.iter_mut() {
                        s.line_reflection([0.0, 0.0], [1.0, 0.0]);
                    }
                }

                return EvenlyDistributedMagnets::<false>::new(
                    core.poles().into(),
                    indent_center_radius * TAU,
                    magnet_shapes,
                    magnet_coverage,
                    num_tangential,
                )
                .into();
            }
        }
    }

    fn winding_zones(&self, _core: CoreRef<'_>, _coil_layout: &CoilLayout) -> WindingZones {
        // Placeholder, since the StraightIndentsAirGap can't be wound
        WindingZonesEqSpaced::<Contour, true>::no_slots().into()
    }

    fn slots(&self, _core: CoreRef<'_>) -> u16 {
        0
    }

    fn combine(&mut self, core: CoreRef<'_>) -> Result<Shape, Error> {
        match core {
            CoreRef::Lin(lin_core) => self.shape_lin(lin_core),
            CoreRef::Rot(rot_core) => self.shape_rot(rot_core),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct AirGapPolygonBuilder {
    pub num_segments: usize,
    pub indents_per_pole: usize,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub indent_depth: Length, // Depth of the indent. A negative depth leads to an extrusion
}

impl AirGapPolygonBuilder {
    pub fn convert_rot(self, core: &RotCore) -> StraightIndentsAirGap {
        // Calculate the side length of the regular polygon
        let n_sides = 2.0 * core.pole_pairs() as f64 * self.indents_per_pole as f64;
        let indent_width = 2.0 * core.air_gap_radius() * (std::f64::consts::PI / n_sides).sin();

        return StraightIndentsAirGap {
            num_segments: self.num_segments,
            indent_width: indent_width,
            indent_depth: self.indent_depth,
            indents_per_pole: self.indents_per_pole,
        };
    }
}
