use std::{
    f64::consts::{FRAC_PI_2, PI, TAU},
    sync::Arc,
};

use crate::{
    core::{LinCore, RotCore},
    flux_barrier::FluxBarrier,
    magnets::PositionedMagnetShape,
    planar_geo,
};
use compare_variables::compare_variables;
use num::Integer;
use stem_magnet::{assembly::MagnetAssembly, block::BlockMagnet, magnet::Magnet};
use stem_slot::prelude::stem_material::prelude::*;
use stem_slot::prelude::*;

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

#[derive(Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Star1HeightSplit {
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    MagnetSpaceHeight(Length),
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    ReliefPathWidth(Length),
}

impl Star1HeightSplit {
    /// Returns an array `[magnet_space_height, relief_path_width]`.
    fn height_and_width(&self, total: Length) -> [Length; 2] {
        match self {
            Star1HeightSplit::MagnetSpaceHeight(magnet_space_height) => {
                [*magnet_space_height, total - *magnet_space_height]
            }
            Star1HeightSplit::ReliefPathWidth(relief_path_width) => {
                [total - *relief_path_width, *relief_path_width]
            }
        }
    }
}

/// TODO
#[doc = ""]
#[cfg_attr(feature = "doc-images", doc = "![Star1 drawing][cad_star1]")]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image("cad_star1", "docs/img/cad_star1.svg")
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
///
/// TODO
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Star1FluxBarrier {
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub air_gap_leakage_path_width: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub yoke_leakage_path_width: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub relief_path_air_gap_width: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub magnet_space_width: Length,
    pub magnet_space_height_or_relief_path_width: Star1HeightSplit,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub glue_gap: Length,
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serialize_opt_arc_link",
            deserialize_with = "deserialize_opt_arc_link"
        )
    )]
    pub magnet_material: Option<Arc<Material>>,
    /// Geometry data generated from [`FluxBarrier::combine`]. Set this field to
    /// [`None`] when building a new [`Star1FluxBarrier`] instance.
    ///
    /// If this field is not [`None`], the [`Cache`] holds data resulting from
    /// the combination of [`Star1FluxBarrier`] with a [`CoreRef`]. This data
    /// might be partially public and partially internal information.
    ///
    /// See the docstring of [`Cache`] for more.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub cache: Option<Cache>,
}

/// A struct resulting from combining a [`Star1FluxBarrier`] with a [`CoreRef`]
/// via [`FluxBarrier::combine`].
///
/// This struct is created by applying [`FluxBarrier::combine`] to a
/// [`Star1FluxBarrier`] and is then placed into the [`Star1FluxBarrier::cache`]
/// field. It caches information from the combination procedure which is
/// expensive to calculate; some of this information might be public and other
/// might be private. Therefore, this struct cannot be created on its own. It is
/// overwritten each time a [`Star1FluxBarrier`] is combined with a [`CoreRef`],
/// therefore it makes no sense to move it from one [`Star1FluxBarrier`] to
/// another one.
#[derive(Debug, Clone)]
pub struct Cache {
    pub pt_yoke_leakage: [f64; 2],
    pub pt_inner_relief: [f64; 2],
    pub pt_outer_relief: [f64; 2],
    pub pt_air_gap_leakage: [f64; 2],
    pub magnet_space_height: Length,
    pub relief_path_width: Length,
    pub air_gap_leakage_segment: Segment,
    pub yoke_leakage_segment: Segment,
    pub pole_pairs: u16,
    magnets: Option<[MagnetAssembly; 1]>,
}

impl Star1FluxBarrier {
    pub fn total_magnet_space_width(&self) -> Length {
        return self.magnet_space_width + 2.0 * self.glue_gap;
    }

    pub fn total_magnet_space_height(&self) -> Length {
        return self
            .cache
            .as_ref()
            .map_or(Length::new::<meter>(0.0), |c| c.magnet_space_height)
            + 2.0 * self.glue_gap;
    }

    fn combine_lin(&mut self, core: &LinCore) -> Result<Vec<Contour>, Error> {
        let tooth_height = core.tooth_height();
        let yoke_height = core.yoke_height();
        let zero = Length::new::<meter>(0.0);
        let height_for_flux_barrier = yoke_height
            - 2.0 * self.glue_gap
            - self.air_gap_leakage_path_width
            - self.yoke_leakage_path_width;
        compare_variables!(val zero < height_for_flux_barrier)?;

        let [magnet_space_height, relief_path_width] = self
            .magnet_space_height_or_relief_path_width
            .height_and_width(height_for_flux_barrier);
        let half_width = 0.5 * self.magnet_space_width + self.glue_gap;

        let has_relief_path = relief_path_width != zero;

        // Calculate the points
        let pt_air_gap_leakage = if has_relief_path {
            [
                0.5 * self.relief_path_air_gap_width.get::<meter>(),
                (tooth_height + self.air_gap_leakage_path_width).get::<meter>(),
            ]
        } else {
            [
                half_width.get::<meter>(),
                (tooth_height + self.air_gap_leakage_path_width).get::<meter>(),
            ]
        };

        let pt_inner_relief = [
            pt_air_gap_leakage[0],
            pt_air_gap_leakage[1] + relief_path_width.get::<meter>(),
        ];

        let pt_outer_relief = [half_width.get::<meter>(), pt_inner_relief[1]];
        let pt_yoke_leakage = [
            half_width.get::<meter>(),
            pt_outer_relief[1] + (2.0 * self.glue_gap + magnet_space_height).get::<meter>(),
        ];

        // Calculate the leakage segments
        let air_gap_leakage_segment: Segment = LineSegment::new(
            [-pt_air_gap_leakage[0], pt_air_gap_leakage[1]],
            pt_air_gap_leakage,
        )?
        .into();

        let yoke_leakage_segment: Segment =
            LineSegment::new(pt_yoke_leakage, [-pt_yoke_leakage[0], pt_yoke_leakage[1]])?.into();

        // Build the contour
        let mut ps = Polysegment::with_capacity(8);
        ps.push_back(air_gap_leakage_segment.clone().into());
        if has_relief_path {
            if let Ok(ls) = LineSegment::new(pt_inner_relief, pt_outer_relief) {
                ps.push_back(ls.into());
            }
        }
        ps.push_back(yoke_leakage_segment.clone().into());
        if has_relief_path {
            if let Ok(ls) = LineSegment::new(
                [-pt_outer_relief[0], pt_outer_relief[1]],
                [-pt_inner_relief[0], pt_inner_relief[1]],
            ) {
                ps.push_back(ls.into());
            }
        }
        let contour = Contour::new(ps);

        // Populate the cache
        self.cache = Some(Cache {
            pt_yoke_leakage,
            pt_inner_relief,
            pt_outer_relief,
            pt_air_gap_leakage,
            magnet_space_height,
            relief_path_width,
            air_gap_leakage_segment,
            yoke_leakage_segment,
            pole_pairs: core.pole_pairs(),
            magnets: self.magnet_assembly(magnet_space_height, core.axial_length()),
        });

        // Repeat the contours
        let mut contours = Vec::with_capacity(core.poles().into());
        contours.push(contour);
        let dist_between_poles = core.width().get::<meter>() / core.poles() as f64;

        contours
            .iter_mut()
            .for_each(|c| c.translate([0.5 * dist_between_poles, 0.0]));

        // Repeat the contours over all pole pairs
        for p in 1..core.poles() {
            let mut c0 = contours[0].clone();
            c0.translate([p as f64 * dist_between_poles, 0.0]);
            contours.push(c0);
        }

        return Ok(contours);
    }

    fn combine_rot(&mut self, core: &RotCore) -> Result<Vec<Contour>, Error> {
        let zero = Length::new::<meter>(0.0);

        let yoke_radius = core.yoke_radius();
        let air_gap_radius = core.air_gap_radius();
        let m = if core.is_outer() { -1.0 } else { 1.0 };

        let yoke_leakage_radius = (yoke_radius + m * self.yoke_leakage_path_width).get::<meter>();
        let air_gap_leakage_radius =
            (air_gap_radius - m * self.air_gap_leakage_path_width).get::<meter>();
        let total_width = self.total_magnet_space_width().get::<meter>();
        let yoke_sweep_angle = 2.0 * (0.5 * total_width / yoke_leakage_radius).asin();

        let has_relief_path = match self.magnet_space_height_or_relief_path_width {
            Star1HeightSplit::MagnetSpaceHeight(_) => true,
            Star1HeightSplit::ReliefPathWidth(width) => width > zero,
        };

        let air_gap_leakage_segment_width = if has_relief_path {
            self.relief_path_air_gap_width.get::<meter>()
        } else {
            total_width
        };

        let yoke_leakage_segment: Segment = ArcSegment::from_center_radius_start_sweep_angle(
            [0.0, 0.0],
            yoke_leakage_radius,
            FRAC_PI_2 + 0.5 * yoke_sweep_angle,
            -yoke_sweep_angle,
        )?
        .into();

        let air_gap_leakage_segment: Segment = if let Some(slot_height) =
            core.slot().map(Slot::height)
        {
            let y = (core.air_gap_radius() - m * slot_height - m * self.air_gap_leakage_path_width)
                .get::<meter>();
            LineSegment::new(
                [0.5 * air_gap_leakage_segment_width, y],
                [-0.5 * air_gap_leakage_segment_width, y],
            )?
            .into()
        } else {
            let ag_sweep_angle =
                2.0 * (0.5 * air_gap_leakage_segment_width / air_gap_leakage_radius).asin();
            ArcSegment::from_center_radius_start_sweep_angle(
                [0.0, 0.0],
                air_gap_leakage_radius,
                FRAC_PI_2 - 0.5 * ag_sweep_angle,
                ag_sweep_angle,
            )?
            .into()
        };

        let pt_yoke_leakage = yoke_leakage_segment.stop();
        let pt_air_gap_leakage = air_gap_leakage_segment.start();

        let height_for_flux_barrier = if core.is_outer() {
            if !has_relief_path && core.slot().is_none() {
                Length::new::<meter>(pt_yoke_leakage[1] - air_gap_leakage_radius).abs()
                    - 2.0 * self.glue_gap
            } else {
                Length::new::<meter>(pt_yoke_leakage[1] - pt_air_gap_leakage[1]).abs()
                    - 2.0 * self.glue_gap
            }
        } else {
            Length::new::<meter>(pt_air_gap_leakage[1] - yoke_leakage_radius).abs()
                - 2.0 * self.glue_gap
        };
        compare_variables!(val zero < height_for_flux_barrier)?;

        let [magnet_space_height, relief_path_width] = self
            .magnet_space_height_or_relief_path_width
            .height_and_width(height_for_flux_barrier);

        let pt_inner_relief = [
            0.5 * air_gap_leakage_segment_width,
            air_gap_leakage_segment.stop()[1] - m * relief_path_width.get::<meter>(),
        ];
        let pt_outer_relief = [pt_yoke_leakage[0], pt_inner_relief[1]];

        // Build the contour
        let mut ps = Polysegment::with_capacity(8);
        ps.push_back(yoke_leakage_segment.clone().into());
        if has_relief_path {
            if let Ok(ls) = LineSegment::new(pt_outer_relief, pt_inner_relief) {
                ps.push_back(ls.into());
            }
        }
        ps.push_back(air_gap_leakage_segment.clone().into());
        if has_relief_path {
            if let Ok(ls) = LineSegment::new(
                [-pt_inner_relief[0], pt_inner_relief[1]],
                [-pt_outer_relief[0], pt_outer_relief[1]],
            ) {
                ps.push_back(ls.into());
            }
        }
        let contour = Contour::new(ps);

        // Populate the cache
        self.cache = Some(Cache {
            pt_yoke_leakage,
            pt_inner_relief,
            pt_outer_relief,
            pt_air_gap_leakage,
            magnet_space_height,
            relief_path_width,
            air_gap_leakage_segment,
            yoke_leakage_segment,
            pole_pairs: core.pole_pairs(),
            magnets: self.magnet_assembly(magnet_space_height, core.axial_length()),
        });

        let mut contours: Vec<Contour> = Vec::with_capacity(core.poles().into());
        contours.push(contour);

        // Rotate the flux barrier so the q-axis is on the x-axis
        contours
            .iter_mut()
            .for_each(|c| c.rotate([0.0, 0.0], -FRAC_PI_2));

        // Repeat the contours over all pole pairs
        for p in 1..core.poles() {
            let rot_angle = p as f64 * TAU / core.poles() as f64;

            let mut c0 = contours[0].clone();
            c0.rotate([0.0, 0.0], rot_angle);
            contours.push(c0);
        }

        return Ok(contours);
    }

    fn magnet_assembly(
        &self,
        magnet_space_height: Length,
        axial_length: Length,
    ) -> Option<[MagnetAssembly; 1]> {
        self.magnet_material
            .as_ref()
            .map(|material| {
                BlockMagnet::new(
                    axial_length,
                    magnet_space_height,
                    self.magnet_space_width,
                    Length::new::<meter>(0.0),
                    material.clone(),
                )
                .map(|m| {
                    MagnetAssembly::new(
                        m,
                        1.try_into().expect("is not zero"),
                        1.try_into().expect("is not zero"), // 1 magnet per pole
                    )
                })
                .ok()
            })
            .flatten()
            .map(|a| [a])
    }

    pub fn magnet(&self) -> Option<&BlockMagnet> {
        self.cache
            .as_ref()
            .map(|c| {
                let magnets = (&c.magnets).as_ref()?;
                (magnets[0].magnet() as &dyn std::any::Any).downcast_ref::<BlockMagnet>()
            })
            .flatten()
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl FluxBarrier for Star1FluxBarrier {
    fn starts_in_d_axis(&self, core: CoreRef<'_>) -> bool {
        match core {
            CoreRef::Lin(_) => true,
            CoreRef::Rot(_) => false,
        }
    }

    fn pole_coverage(&self, core: CoreRef<'_>) -> f64 {
        match core {
            CoreRef::Lin(lin_core) => (self.total_magnet_space_width() * lin_core.poles() as f64
                / lin_core.width())
            .get::<ratio>(),
            CoreRef::Rot(rot_core) => {
                let angle = 2.0
                    * (0.5 * self.total_magnet_space_width() / rot_core.air_gap_radius())
                        .get::<ratio>()
                        .asin();
                return 2.0 * angle / PI * rot_core.poles() as f64;
            }
        }
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

        match core {
            CoreRef::Lin(lin_core) => {
                let shift = [
                    0.5 * magnet.thickness().get::<meter>(),
                    0.5 * magnet.width().get::<meter>()
                        + self.glue_gap.get::<meter>()
                        + c.pt_outer_relief[1],
                ];

                // Rotate the shapes by 90 degree to bring them into position
                shapes.iter_mut().for_each(|s| {
                    s.rotate([0.0, 0.0], FRAC_PI_2);
                    s.translate(shift);
                });

                // Since only every second pole has a flux barrier, use the
                // number of pole pairs as number of poles
                return EvenlyDistributedMagnets::<true>::new(
                    core.poles().into(),
                    lin_core.air_gap_length(),
                    shapes,
                    0.0,
                    1,
                )
                .into();
            }
            CoreRef::Rot(rot_core) => {
                let radius = if rot_core.is_outer() {
                    c.pt_yoke_leakage[1]
                        - 1.0 * self.glue_gap.get::<meter>()
                        - 0.5 * magnet.width().get::<meter>()
                } else {
                    c.pt_outer_relief[1]
                        - 1.0 * self.glue_gap.get::<meter>()
                        - 0.5 * magnet.width().get::<meter>()
                };

                // Rotate the shapes by 90 degree to bring them into position
                let angle = PI / rot_core.poles() as f64;
                shapes.iter_mut().for_each(|s| {
                    s.translate([0.0, -0.5 * magnet.thickness().get::<meter>()]);
                    s.rotate([0.0, 0.0], FRAC_PI_2);
                    s.rotate([0.0, -radius], angle);
                });

                return EvenlyDistributedMagnets::<false>::new(
                    core.poles().into(),
                    Length::new::<meter>(radius * TAU),
                    shapes,
                    0.0,
                    1,
                )
                .into();
            }
        }
    }

    fn combine(&mut self, core: CoreRef<'_>) -> Result<Vec<Contour>, Error> {
        let zero = Length::new::<meter>(0.0);
        compare_variables!(val zero <= self.glue_gap)?;
        compare_variables!(val zero < self.magnet_space_width)?;
        match self.magnet_space_height_or_relief_path_width {
            Star1HeightSplit::MagnetSpaceHeight(magnet_space_height) => {
                compare_variables!(val zero <= magnet_space_height)?;
            }
            Star1HeightSplit::ReliefPathWidth(relief_path_width) => {
                compare_variables!(val zero <= relief_path_width)?;
            }
        }

        match core {
            CoreRef::Lin(lin_core) => self.combine_lin(lin_core),
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
