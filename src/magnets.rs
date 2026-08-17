use crate::planar_geo;
use num::Integer;
use planar_geo::prelude::*;
use std::f64::consts::{FRAC_PI_2, PI, TAU};
use stem_magnet::prelude::*;

#[derive(Debug, Clone)]
pub struct PositionedMagnetShape {
    pub shape: Shape,
    pub is_north: bool,
    /**
    A [`FluxBarrier`](crate::flux_barrier::FluxBarrier) can have multiple types
    of magnets. A slice of all the magnet types can be retrieved with the
    [`FluxBarrier::magnet_assemblies`](crate::flux_barrier::FluxBarrier::magnet_assemblies)
    method. This index can be used for retrieving the magnet type of `self` from
    that list. If the [`PositionedMagnetShape`] was created from an iterator
    over the surface magnets, this value is simply 0.
    */
    pub magnet_idx: usize,
}

impl PositionedMagnetShape {
    pub fn into_drawable(self) -> Drawable {
        let style = if self.is_north {
            stem_magnet::NORTH_POLE_STYLE
        } else {
            stem_magnet::SOUTH_POLE_STYLE
        };
        return Drawable::new(self.shape, style);
    }
}

impl From<PositionedMagnetShape> for Drawable {
    fn from(value: PositionedMagnetShape) -> Self {
        value.into_drawable()
    }
}

impl Transformation for PositionedMagnetShape {
    fn translate(&mut self, shift: [f64; 2]) {
        self.shape.translate(shift);
    }

    fn rotate(&mut self, center: [f64; 2], angle: f64) {
        self.shape.rotate(center, angle);
    }

    fn scale(&mut self, factor: f64) {
        self.shape.scale(factor);
    }

    fn line_reflection(&mut self, start: [f64; 2], stop: [f64; 2]) -> () {
        self.shape.line_reflection(start, stop);
    }
}

// =============================================================================
// Core with a smooth surface

#[derive(Debug, Clone)]
pub struct EvenlyDistributedMagnets<const LIN: bool> {
    poles: usize,
    dist_width_or_circumference: Length,
    magnet_shapes: Vec<PositionedMagnetShape>,
    magnet_coverage: f64, // If LIN, this is a length, otherwise it is an angle
    num_tangential: usize,
    index: usize,
    d_axis_offset: f64,
}

impl<const LIN: bool> EvenlyDistributedMagnets<LIN> {
    pub fn new(
        poles: usize,
        dist_width_or_circumference: Length,
        magnet_shapes: Vec<PositionedMagnetShape>,
        magnet_coverage: f64,
        num_tangential: usize,
        d_axis_offset: f64,
    ) -> Self {
        Self {
            poles,
            dist_width_or_circumference,
            magnet_shapes,
            num_tangential,
            magnet_coverage,
            index: 0,
            d_axis_offset,
        }
    }

    /// magnet_shapes must have one or 2 entries (otherwise panic!)
    pub fn with_calculated_coverage(
        poles: usize,
        dist_width_or_circumference: Length,
        magnet_shapes: Vec<PositionedMagnetShape>,
        num_tangential: usize,
        d_axis_offset: f64,
    ) -> Self {
        let magnet_coverage = if LIN {
            BoundingBox::from_bounded_entities(magnet_shapes.iter().map(|p| &p.shape))
                .map(|m| m.width())
                .unwrap_or(0.0)
        } else {
            let radius = dist_width_or_circumference.get::<meter>() / TAU;
            pole_coverage_angle(
                magnet_shapes.iter().map(|p| &p.shape),
                radius,
                Length::new::<meter>(0.0),
            )
        };

        Self {
            poles,
            dist_width_or_circumference,
            magnet_shapes,
            num_tangential,
            magnet_coverage,
            index: 0,
            d_axis_offset,
        }
    }

    pub fn from_magnet_assembly(
        poles: usize,
        air_gap_length: Length,
        assembly: &MagnetAssembly,
        split: bool,
        outer_core: bool,
        d_axis_offset: f64,
    ) -> Self {
        if LIN {
            let num_tangential = assembly.num_tangential();
            let mut magnet_shapes = if split {
                assembly
                    .magnet()
                    .north_south_shapes()
                    .into_iter()
                    .map(|c| c.into_owned())
                    .collect()
            } else {
                vec![assembly.magnet().shape().into_owned()]
            };

            magnet_shapes.iter_mut().for_each(|s| {
                s.line_reflection([0.0, 0.0], [1.0, 0.0]);
            });

            return Self::with_calculated_coverage(
                poles,
                air_gap_length,
                magnet_shapes
                    .into_iter()
                    .enumerate()
                    .map(|(i, shape)| PositionedMagnetShape {
                        shape,
                        is_north: i.is_even(),
                        magnet_idx: 0,
                    })
                    .collect(),
                num_tangential,
                d_axis_offset,
            );
        } else {
            let num_tangential = assembly.num_tangential();
            let mut magnet_shapes = if split {
                assembly
                    .magnet()
                    .north_south_shapes()
                    .into_iter()
                    .map(|c| c.into_owned())
                    .collect()
            } else {
                vec![assembly.magnet().shape().into_owned()]
            };

            if outer_core {
                for s in magnet_shapes.iter_mut() {
                    s.line_reflection([0.0, 0.0], [1.0, 0.0]);
                }
            }

            return Self::with_calculated_coverage(
                poles,
                air_gap_length,
                magnet_shapes
                    .into_iter()
                    .enumerate()
                    .map(|(i, shape)| PositionedMagnetShape {
                        shape,
                        is_north: i.is_even(),
                        magnet_idx: 0,
                    })
                    .collect(),
                num_tangential,
                d_axis_offset,
            );
        }
    }
}

impl<const LIN: bool> Iterator for EvenlyDistributedMagnets<LIN> {
    type Item = PositionedMagnetShape;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.num_tangential * self.poles * self.magnet_shapes.len() {
            return None;
        }

        let current_pole = self.index / (self.num_tangential * self.magnet_shapes.len());

        // Index counter repeats from zero to the number of shapes per pole
        let pole_idx = self
            .index
            .rem_euclid(self.num_tangential * self.magnet_shapes.len());
        let tan_idx = pole_idx / self.magnet_shapes.len();
        let shape_idx = pole_idx.rem_euclid(self.magnet_shapes.len());

        self.index = self.index + 1;

        let mut shape = self.magnet_shapes[shape_idx].clone();

        if LIN {
            let width_per_pole =
                self.dist_width_or_circumference.get::<meter>() / self.poles as f64;
            let offset = (self.d_axis_offset / PI + current_pole as f64) * width_per_pole
                + (tan_idx as f64 - 0.5 * (self.num_tangential as f64 - 1.0))
                    * self.magnet_coverage;
            shape.translate([offset, 0.0]);
        } else {
            shape.translate([0.0, self.dist_width_or_circumference.get::<meter>() / TAU]);
            let angle_per_pole = TAU / self.poles as f64;
            let angle = angle_per_pole * (0.5 + current_pole as f64)
                + (tan_idx as f64 - 0.5 * (self.num_tangential as f64 - 1.0))
                    * self.magnet_coverage;
            shape.rotate([0.0, 0.0], -angle + self.d_axis_offset);
        }

        if current_pole.is_odd() {
            shape.is_north = !shape.is_north;
        }

        return Some(shape);
    }
}

// =============================================================================

pub enum Magnets {
    EvenlyDistributedMagnetsLin(EvenlyDistributedMagnets<true>),
    EvenlyDistributedMagnetsRot(EvenlyDistributedMagnets<false>),
    Other(Box<dyn Iterator<Item = PositionedMagnetShape>>),
}

impl From<EvenlyDistributedMagnets<true>> for Magnets {
    fn from(value: EvenlyDistributedMagnets<true>) -> Self {
        Self::EvenlyDistributedMagnetsLin(value)
    }
}

impl From<EvenlyDistributedMagnets<false>> for Magnets {
    fn from(value: EvenlyDistributedMagnets<false>) -> Self {
        Self::EvenlyDistributedMagnetsRot(value)
    }
}

impl From<Box<dyn Iterator<Item = PositionedMagnetShape>>> for Magnets {
    fn from(value: Box<dyn Iterator<Item = PositionedMagnetShape>>) -> Self {
        Self::Other(value)
    }
}

impl Iterator for Magnets {
    type Item = PositionedMagnetShape;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Magnets::EvenlyDistributedMagnetsLin(i) => i.next(),
            Magnets::EvenlyDistributedMagnetsRot(i) => i.next(),
            Magnets::Other(i) => i.next(),
        }
    }
}

/// Assumptions: shapes are as required by Magnet trait
/// TODO: Explain that this can deal with negative radii (as might be reported
/// from some magnets such as ArcSegmentMagnet to signal convex / concave)
pub fn pole_coverage_angle<'a, I: Iterator<Item = &'a Shape> + Clone>(
    magnet_shapes: I,
    radius: f64,
    gap_between_partial_magnets: Length,
) -> f64 {
    // Based on the shape points, calculate the preliminary coverage angle
    let mut angle = magnet_shapes
        .clone()
        .map(|shape| shape.contour().points())
        .flatten()
        .map(|p| {
            let x = p[0];
            let buf = x.signum() * 0.5 * gap_between_partial_magnets.get::<meter>();
            (p[1] + radius).atan2(p[0] + buf)
        })
        .reduce(f64::max)
        .unwrap_or(FRAC_PI_2 * radius.signum());

    /*
    Check if the line through [0.0, -radius] with angle cuts through any
    of the arc segments, merely touches them or does not interact with them at
    all. In the first case, increase the angle slightly and test again. Line
    segments can be ignored, since the construction of angle already ensures
    that the first case cannot happen for them.
     */
    for s in magnet_shapes
        .map(|shape| shape.contour().segments())
        .flatten()
    {
        'angle_increment: while angle < PI {
            let line = Line::from_point_angle([0.0, -radius], angle);

            if let Segment::ArcSegment(a) = s {
                match a.intersections_primitive(&line) {
                    PrimitiveIntersections::Zero => break 'angle_increment,
                    PrimitiveIntersections::One(_) => break 'angle_increment,
                    PrimitiveIntersections::Two(_) => {
                        angle += 0.1;
                    }
                }
            } else {
                break 'angle_increment;
            }
        }
    }

    return 2.0 * (angle - FRAC_PI_2 * radius.signum());
}
