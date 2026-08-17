use std::{
    f64::consts::{FRAC_PI_2, PI, TAU},
    sync::Arc,
};

use crate::{
    air_gap::SlottedAirGap,
    core::RotCore,
    flux_barrier::{FluxBarrier, dist_to_q_axis},
    magnets::PositionedMagnetShape,
    planar_geo,
};
use compare_variables::compare_variables;
use num::Integer;
use stem_magnet::{assembly::MagnetAssembly, block::BlockMagnet, magnet::Magnet};
use stem_slot::prelude::stem_material::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use planar_geo::prelude::*;

use crate::{
    core::{CoreExt, CoreRef},
    error::Error,
    magnets::{EvenlyDistributedMagnets, Magnets},
};

#[cfg(feature = "serde")]
use serde_mosaic::{deserialize_opt_arc_link, serialize_opt_arc_link};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct V2rFluxBarrier {
    /// Distance between the core yoke and the center of the flux barrier
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub yoke_distance: Length,
    /// Length of the air gap in the relief path
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub relief_path_air_gap_width: Length,
    // Length of the core material part in the relief path
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub relief_path_length: Length,
    // Width of the core material part in the relief path
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub relief_path_width: Length,
    // Opening angle between the sides
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_angle"))]
    pub opening_angle: f64,
    // Width of the magnet space
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub magnet_space_width: Length,
    // Height of the magnet space
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub magnet_space_height: Length,
    /**
    Fillet at the intersection between the extension of the q-axis sided magnet height and
    the q-axis sided line perpendicular to the flux leakage path. If this value is set None,
    no intersection is calculated and the magnet height and flux leakage path are instead
    connected by a straight line
     */
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_opt_quantity"))]
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "deserialize_opt_quantity")
    )]
    #[cfg_attr(feature = "serde", serde(default))]
    pub fillet_q_side_leakage_space: Option<Length>,
    // Glue gap around the magnet space
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub glue_gap: Length,
    // Width of the leakage path between the motor air gap and the flux barrier
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub leakage_path_width: Length,
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serialize_opt_arc_link",
            deserialize_with = "deserialize_opt_arc_link"
        )
    )]
    pub magnet_material: Option<Arc<Material>>,
    /// Geometry data generated from [`FluxBarrier::combine`]. Set this field to
    /// [`None`] when building a new [`V2rFluxBarrier`] instance.
    ///
    /// If this field is not [`None`], the [`Cache`] holds data resulting from
    /// the combination of [`V2rFluxBarrier`] with a [`CoreRef`]. This data
    /// might be partially public and partially internal information.
    ///
    /// See the docstring of [`Cache`] for more.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub cache: Option<Cache>,
}

/// A struct resulting from combining a [`V2rFluxBarrier`] with a [`CoreRef`]
/// via [`FluxBarrier::combine`].
///
/// This struct is created by applying [`FluxBarrier::combine`] to a
/// [`V2rFluxBarrier`] and is then placed into the [`V2rFluxBarrier::cache`]
/// field. It caches information from the combination procedure which is
/// expensive to calculate; some of this information might be public and other
/// might be private. Therefore, this struct cannot be created on its own. It is
/// overwritten each time a [`V2rFluxBarrier`] is combined with a [`CoreRef`],
/// therefore it makes no sense to move it from one [`V2rFluxBarrier`] to
/// another one.
#[derive(Debug, Clone)]
pub struct Cache {
    pub pt_magnet_center_d: [f64; 2],
    pub pt_magnet_relief_d: [f64; 2],
    pub pt_magnet_relief_q: [f64; 2],
    pub pt_relief_outer_corner_d: [f64; 2],
    pub pt_relief_inner_corner_d: [f64; 2],
    pub pt_relief_outer_corner_q: [f64; 2],
    pub pt_relief_inner_corner_q: [f64; 2],
    pub pt_q_leakage: Option<[f64; 2]>,
    pub pt_magnet_center_q: [f64; 2],
    pub leakage_segment: Segment,
    pub pole_pairs: u16,
    magnets: Option<[MagnetAssembly; 1]>,
}

impl V2rFluxBarrier {
    pub fn magnet(&self) -> Option<&BlockMagnet> {
        self.cache
            .as_ref()
            .map(|c| {
                let magnets = (&c.magnets).as_ref()?;
                (magnets[0].magnet() as &dyn std::any::Any).downcast_ref::<BlockMagnet>()
            })
            .flatten()
    }

    pub fn total_magnet_space_width(&self) -> Length {
        return self.magnet_space_width + 2.0 * self.glue_gap;
    }

    pub fn total_magnet_space_height(&self) -> Length {
        return self.magnet_space_height + 2.0 * self.glue_gap;
    }

    pub fn flux_leakage_path_length(&self) -> Length {
        return Length::new::<meter>(
            self.cache
                .as_ref()
                .map_or(0.0, |c| c.leakage_segment.length()),
        );
    }

    /**
    Return the distance between the yoke side of the magnet space and the q-axis.
     */
    pub fn barrier_to_q_axis_at_yoke(&self) -> Length {
        match self.cache.as_ref() {
            Some(c) => Length::new::<meter>(dist_to_q_axis(c.pt_magnet_relief_q, c.pole_pairs)),
            None => Length::new::<meter>(0.0),
        }
    }

    /**
    Return the distance between the air gap side of the magnet space and the q-axis.
     */
    pub fn distance_to_q_axis_at_leakage_to_magnet_transition(&self) -> Length {
        match self.cache.as_ref() {
            Some(c) => {
                if let Some(pt_q_leakage) = c.pt_q_leakage {
                    return Length::new::<meter>(dist_to_q_axis(pt_q_leakage, c.pole_pairs));
                } else {
                    return self.distance_to_q_axis_at_yoke();
                }
            }
            None => Length::new::<meter>(0.0),
        }
    }

    /**
    Return the distance between the air gap side of the magnet space and the q-axis.
     */
    pub fn distance_to_q_axis_at_air_gap(&self) -> Length {
        match self.cache.as_ref() {
            Some(c) => {
                Length::new::<meter>(dist_to_q_axis(c.pt_relief_outer_corner_q, c.pole_pairs))
            }
            None => Length::new::<meter>(0.0),
        }
    }

    /**
    Return the distance between the air gap side of the magnet space and the q-axis.
     */
    pub fn distance_to_q_axis_at_yoke(&self) -> Length {
        match self.cache.as_ref() {
            Some(c) => {
                if let Some(q_corner) = c.pt_q_leakage {
                    return Length::new::<meter>(dist_to_q_axis(q_corner, c.pole_pairs));
                } else {
                    return Length::new::<meter>(dist_to_q_axis(
                        c.pt_magnet_relief_q,
                        c.pole_pairs,
                    ));
                }
            }
            None => Length::new::<meter>(0.0),
        }
    }

    fn combine_rot(&mut self, core: &RotCore) -> Result<Vec<Contour>, Error> {
        let zero = Length::new::<meter>(0.0);
        compare_variables!(val zero < self.yoke_distance)?;
        compare_variables!(val zero <= self.relief_path_air_gap_width)?;
        compare_variables!(val zero <= self.relief_path_width)?;
        compare_variables!(0.0 <= self.opening_angle <= PI)?;
        compare_variables!(val zero < self.magnet_space_width)?;
        compare_variables!(val zero < self.magnet_space_height)?;
        compare_variables!(val zero < self.glue_gap)?;
        compare_variables!(val zero < self.leakage_path_width)?;

        let yoke_radius = core.yoke_radius();
        let air_gap_radius = core.air_gap_radius();
        let m = if core.is_outer() { -1.0 } else { 1.0 };

        // pt at the magnet space corner next to the yoke
        let angle = FRAC_PI_2 - 0.5 * self.opening_angle;
        let (angle_sin, angle_cos) = angle.sin_cos();

        let radius = (yoke_radius + m * self.yoke_distance).get::<meter>();
        let h = (2.0 * self.glue_gap + self.magnet_space_height).get::<meter>();
        let w = (2.0 * self.glue_gap + self.magnet_space_width).get::<meter>();

        let xh = h * angle_cos;
        let yh = h * angle_sin;
        let xw = w * angle_sin;
        let yw = w * angle_cos;

        // The two magnet space widths "w" form together with "xw" an isosceles triangle
        // Another isosceles triangle is formed by two "radius" and
        // "dist_magnet_center_q". One of the corner points is pt_magnet_center_q
        let pt_magnet_center_q = [xw, (radius.powi(2) - xw.powi(2)).sqrt()];
        let pt_magnet_relief_q = [pt_magnet_center_q[0] + xh, pt_magnet_center_q[1] + yh];
        let pt_magnet_center_d = [pt_magnet_center_q[0] - xw, pt_magnet_center_q[1] + yw];
        let pt_magnet_relief_d = [pt_magnet_relief_q[0] - xw, pt_magnet_relief_q[1] + yw];

        // Calculate the vertices of the leakage path, starting with the point on the
        // inner side of the V
        let leakage_segment: Segment = if let Some(ags) =
            (core.air_gap() as &dyn std::any::Any).downcast_ref::<SlottedAirGap>()
        {
            /*
            Find the slot whose slot bottom center is closest to the middle of the line
            pt_magnet_leakage_d - pt_magnet_leakage_q
             */
            let x = 0.5 * (pt_magnet_relief_d[0] + pt_magnet_relief_q[0]);
            let y = 0.5 * (pt_magnet_relief_d[1] + pt_magnet_relief_q[1]);
            let mut middle = [x, y];

            /*
            Transform from the current coordinate system, where the d-axis equals the y-axis to the core
            coordinate system, where the x-axis equals the q-axis. Then transform the result back
             */
            let pole_pairs = core.pole_pairs();
            let rotate_to_core_coords = -FRAC_PI_2 + TAU / (4.0 * pole_pairs as f64);
            middle.rotate([0.0, 0.0], rotate_to_core_coords);

            let (mut closest_slot_bottom, slot) =
                crate::flux_barrier::closest_slot_bottom_middle_rot(middle, &core, ags);
            closest_slot_bottom.rotate([0.0, 0.0], -rotate_to_core_coords);

            let offset_tooth = if ags.starts_in_slot_middle { 0.0 } else { 0.5 };
            let slot_angle = std::f64::consts::TAU / core.slots() as f64
                * (slot as f64 + offset_tooth)
                - rotate_to_core_coords;

            let mut leakage_segment = LineSegment::new(
                [
                    closest_slot_bottom[0] - 0.5 * self.relief_path_air_gap_width.get::<meter>(),
                    closest_slot_bottom[1] - m * self.leakage_path_width.get::<meter>(),
                ],
                [
                    closest_slot_bottom[0] + 0.5 * self.relief_path_air_gap_width.get::<meter>(),
                    closest_slot_bottom[1] - m * self.leakage_path_width.get::<meter>(),
                ],
            )?;
            leakage_segment.rotate(closest_slot_bottom, slot_angle - FRAC_PI_2);
            leakage_segment.into()
        } else {
            let relief_radius = (air_gap_radius
                - m * (self.leakage_path_width + self.relief_path_width))
                .get::<meter>();
            let relief_circle = ArcSegment::circle([0.0, 0.0], relief_radius)?;

            let mag_space_middle = [
                0.5 * (pt_magnet_relief_d[0] + pt_magnet_relief_q[0]),
                0.5 * (pt_magnet_relief_d[1] + pt_magnet_relief_q[1]),
            ];
            let ext_middle = LineSegment::from_start_angle_length(
                mag_space_middle,
                angle,
                10.0 * air_gap_radius.get::<meter>(),
            )?;
            let intersection = match relief_circle.intersections_primitive(&ext_middle) {
                PrimitiveIntersections::Zero => mag_space_middle,
                PrimitiveIntersections::One(i) => i,
                PrimitiveIntersections::Two([i1, i2]) => {
                    let pt = mag_space_middle;
                    if (i1[0] - pt[0]).powi(2) + (i1[1] - pt[1]).powi(2)
                        < (i2[0] - pt[0]).powi(2) + (i2[1] - pt[1]).powi(2)
                    {
                        i1
                    } else {
                        i2
                    }
                }
            };

            let middle_angle = intersection[1].atan2(intersection[0]);

            let leakage_radius = (air_gap_radius - m * (self.leakage_path_width)).get::<meter>();
            let sweep_angle =
                2.0 * (0.5 * self.relief_path_air_gap_width.get::<meter>() / leakage_radius).asin();
            let start_angle = middle_angle + 0.5 * sweep_angle;

            ArcSegment::from_center_radius_start_sweep_angle(
                [0.0, 0.0],
                leakage_radius,
                start_angle,
                -sweep_angle,
            )?
            .into()
        };

        let start = leakage_segment.start();
        let stop = leakage_segment.stop();
        let angle = (start[1] - stop[1]).atan2(start[0] - stop[0]);
        let (angle_sin, angle_cos) = angle.sin_cos();

        let x = self.relief_path_width.get::<meter>() * -angle_sin;
        let y = self.relief_path_width.get::<meter>() * angle_cos;
        let pt_relief_inner_corner_d = [start[0] + x, start[1] + y];
        let pt_relief_inner_corner_q = [stop[0] + x, stop[1] + y];

        let half_length = 0.5 * self.relief_path_length.get::<meter>();
        let x = half_length * angle_cos;
        let y = half_length * angle_sin;
        let pt_relief_outer_corner_d = [
            pt_relief_inner_corner_d[0] + x,
            pt_relief_inner_corner_d[1] + y,
        ];
        let pt_relief_outer_corner_q = [
            pt_relief_inner_corner_q[0] - x,
            pt_relief_inner_corner_q[1] - y,
        ];

        /*
        If a fillet is specified, calculate the point where it is going to be placed
         */
        let pt_q_leakage_and_fillet_radius: Option<([f64; 2], f64)> = self
            .fillet_q_side_leakage_space
            .map(|f| {
                let angle = LineSegment::new(leakage_segment.stop(), leakage_segment.start())
                    .map_or(0.0, |l| l.angle())
                    + FRAC_PI_2;
                let l1 = Line::from_point_angle(pt_relief_outer_corner_q, angle);
                let l2 = Line::from_point_angle(
                    pt_magnet_relief_q,
                    FRAC_PI_2 - 0.5 * self.opening_angle,
                );

                // Line-line intersection either results in PrimitiveIntersections::Zero or
                // PrimitiveIntersections::One, there cannot be more than one intersection
                // point.
                l1.intersections_primitive(&l2)
                    .into_iter()
                    .next()
                    .map(|pt| (pt, f.get::<meter>()))
            })
            .flatten();

        // Try to create the magnet, if a magnet material was given and the
        // magnet space permits it. If that succeeds, create the assembly.
        let magnets = self
            .magnet_material
            .as_ref()
            .map(|material| {
                BlockMagnet::new(
                    core.axial_length(),
                    self.magnet_space_height,
                    self.magnet_space_width,
                    Length::new::<meter>(0.0),
                    material.clone(),
                )
                .map(|m| {
                    MagnetAssembly::new(
                        m,
                        1.try_into().expect("is not zero"),
                        2.try_into().expect("is not zero"), // 2 magnets per pole
                    )
                })
                .ok()
            })
            .flatten()
            .map(|a| [a]);

        self.cache = Some(Cache {
            pt_magnet_center_d,
            pt_magnet_relief_d,
            pt_relief_inner_corner_d,
            pt_relief_outer_corner_d,
            pt_relief_outer_corner_q,
            pt_relief_inner_corner_q,
            pt_magnet_relief_q,
            pt_magnet_center_q,
            leakage_segment: leakage_segment.clone(),
            pt_q_leakage: pt_q_leakage_and_fillet_radius.map(|t| t.0),
            pole_pairs: core.pole_pairs(),
            magnets,
        });

        // =====================================================================
        // Create the contour(s) for a single pole pair

        let mut ps = Polysegment::new();

        let segment = LineSegment::new(pt_magnet_center_d, pt_magnet_relief_d)?;
        ps.push_back(segment.into());

        if self.relief_path_air_gap_width.get::<meter>() == 0.0 {
            let segment = LineSegment::new(pt_magnet_relief_d, pt_magnet_relief_q)?;
            ps.push_back(segment.into());
        } else {
            let segment = LineSegment::new(pt_magnet_relief_d, pt_relief_outer_corner_d)?;
            ps.push_back(segment.into());

            let segment = LineSegment::new(pt_relief_outer_corner_d, pt_relief_inner_corner_d)?;
            ps.push_back(segment.into());

            ps.push_back(leakage_segment);

            let segment = LineSegment::new(pt_relief_inner_corner_q, pt_relief_outer_corner_q)?;
            ps.push_back(segment.into());
        }

        if let Some((pt_q_leakage, fillet_radius)) = pt_q_leakage_and_fillet_radius {
            if let Ok(arc) = ArcSegment::fillet(
                pt_relief_outer_corner_q,
                pt_q_leakage,
                pt_magnet_relief_q,
                fillet_radius,
            ) {
                ps.push_back(arc.into());
            }
        }

        let segment = LineSegment::new(pt_magnet_relief_q, pt_magnet_center_q)?;
        ps.push_back(segment.into());

        // Mirror the segment_chain along the y-axis, then reverse the mirrored
        // segment_chain, then connect it with the orginal. This forms the
        // contour of the flux barrier.
        let mut ps_mirror = ps.clone();

        if let Ok(arc) = ArcSegment::from_start_stop_center_radius(
            pt_magnet_center_q,
            [-pt_magnet_center_q[0], pt_magnet_center_q[1]],
            [0.0, 0.0],
            (pt_magnet_center_q[0].powi(2) + pt_magnet_center_q[1].powi(2)).sqrt(),
            true,
        ) {
            ps.push_back(arc.into());
        }

        ps_mirror.line_reflection([0.0, 0.0], [0.0, 1.0]);
        ps_mirror.reverse();

        ps.append(&mut ps_mirror);

        let mut contours: Vec<Contour> = Vec::with_capacity(core.poles().into());
        contours.push(ps.into());

        // Rotate flux barrier clockwise by 90°-360/4 p to have the negative
        // q-axis start in the x-axis (d-axis at 90° electrical)
        let q_axis_alignment_angle = PI / (core.poles() as f64);
        contours
            .iter_mut()
            .for_each(|c| c.rotate([0.0, 0.0], q_axis_alignment_angle));

        // Repeat the contours over all pole pairs
        for p in 1..core.poles() {
            let rot_angle = p as f64 * TAU / core.poles() as f64;

            let mut c1 = contours[0].clone();
            c1.rotate([0.0, 0.0], rot_angle);
            contours.push(c1);
        }

        return Ok(contours);
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl FluxBarrier for V2rFluxBarrier {
    fn pole_coverage(&self, core: CoreRef<'_>) -> f64 {
        let middle_leakage_segment = self
            .cache
            .as_ref()
            .map_or([0.0, 0.0], |c| c.leakage_segment.segment_point(0.5));
        let angle = FRAC_PI_2 - middle_leakage_segment[1].atan2(middle_leakage_segment[0]);
        return 2.0 * angle / PI * core.pole_pairs() as f64;
    }

    fn interior_magnets(&self, core: CoreRef<'_>, split: bool) -> Magnets {
        let c = match self.cache.as_ref() {
            Some(c) => c,
            None => return Magnets::Other(Box::new([].into_iter())).into(),
        };
        let magnet = match self.magnet() {
            Some(m) => m,
            None => return Magnets::Other(Box::new([].into_iter())).into(),
        };

        let mut shapes: Vec<PositionedMagnetShape> = if split {
            magnet
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
                shape: magnet.shape().into_owned(),
                is_north: true,
                magnet_idx: 0,
            }]
        };

        let angle = FRAC_PI_2 - 0.5 * self.opening_angle;

        let gg = self.glue_gap.get::<meter>();
        let x = 0.5 * (c.pt_magnet_relief_d[0] + c.pt_magnet_center_d[0]) + gg * angle.sin();
        let y = 0.5 * (c.pt_magnet_relief_d[1] + c.pt_magnet_center_d[1]) - gg * angle.cos();
        let radius = (x.powi(2) + y.powi(2)).sqrt();

        // Correct for the fact that the magnet origin is not at the
        // radius after the shift by x
        let h = radius - (radius.powi(2) - x.powi(2)).sqrt();

        shapes.iter_mut().for_each(|s| {
            s.rotate([0.0, 0.0], angle + PI);
            s.translate([x, -h]);
        });

        for i in 0..shapes.len() {
            let mut shape = shapes[i].clone();
            shape.line_reflection([0.0, 0.0], [0.0, 1.0]);
            shapes.push(shape);
        }

        return EvenlyDistributedMagnets::<false>::new(
            core.poles().into(),
            Length::new::<meter>(radius * TAU),
            shapes,
            0.0,
            1,
            core.d_axis_offset(),
        )
        .into();
    }

    fn combine(&mut self, core: CoreRef<'_>) -> Result<Vec<Contour>, Error> {
        match core {
            CoreRef::Lin(_) => Err(Error::IncompatibleToLinCore("V2rFluxBarrier")),
            CoreRef::Rot(rot_core) => self.combine_rot(rot_core),
        }
    }

    fn magnet_assemblies(&self, _core: CoreRef<'_>) -> &[MagnetAssembly] {
        self.cache
            .as_ref()
            .and_then(|c| c.magnets.as_ref())
            .map_or(&[], |m| m.as_slice())
    }
}
