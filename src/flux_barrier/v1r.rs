/*!
This module provides the [`V1rFluxBarrier`] struct. This flux barrier is
V-shaped with a variable angle between the sides and a single (optional) relief
path in the center (hence the "1r" in the name). As shown in the image below,
this flux barrier might hold interior magnets.
 */
#![cfg_attr(feature = "doc-images", doc = "")]
#![cfg_attr(
    feature = "doc-images",
    doc = "![Rotary core with a V1rFluxBarrier][rot_core_v1r.svg]"
)]
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("rot_core_v1r.svg", "docs/img/rot_core_v1r.svg"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/*!
This struct implements the [`FluxBarrier`] trait and can therefore be used to
build magnetic cores. See the struct docstring for more.
*/

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
    magnets::{Magnets, MagnetsPeriodic},
};

#[cfg(feature = "serde")]
use serde_mosaic::{deserialize_opt_arc_link, serialize_opt_arc_link};

/**
A flux barrier with two rectangular cutouts forming a "V" shape at each pole and
an optional relief / leakage path in the middle of the "V".

*/
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Rotary core with a V1rFluxBarrier][rot_core_v1r]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image("rot_core_v1r", "docs/img/rot_core_v1r.svg")
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**
_This image was produced with `examples/flux_barrier_plots.rs`._

The cutouts of this flux barrier ressemble a "V" at each pole of the core. Each
pole cutout can hold two [`BlockMagnet`]s as shown in the image (see
[`V1rFluxBarrier::magnet_material`]). In the middle of the "V", a relief path
can optionally be added (hence the name "V1r" for a V-shape with one relief
path). If [`V1rFluxBarrier::relief_path_air_gap_width`] is zero, the relief path
has no air gap and is effectively a third leakage path providing additional
mechanical stability at the cost of performance. This flux barrier is currently
only compatible with [`RotCore`]s.

A relief path is a leakage path which partially consists of air and therefore
doesn't exist due to mechanical reasons, but instead protects the magnet against
large magnetic fields originating from the stator by providing a "relief valve"
for the magnetic flux while offering a high enough magnetic resistance so th
 magnet flux doesn't get short-circuited. For a throughout explanation of the
 concept, see [\[1\]](#v1r_fb_1).

Constructing a [`V1rFluxBarrier`] requires specifying some geometric
dimensions, while other dimensions are calculated later when
[combined](FluxBarrier::combine) with a magnetic core. The drawing below shows
the definition of all dimensions of this flux barrier type:
 */
#[doc = ""]
#[cfg_attr(feature = "doc-images", doc = "![V1r drawing][cad_v1r]")]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image("cad_v1r", "docs/img/cad_v1r.svg")
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

During the combination, the calculated dimensions are stored within a [`Cache`]
which is then put into [`V1rFluxBarrier::cache`]. When constructing a
[`V1rFluxBarrier`], this field therefore should be simply set to `None` (it
is not possible to create a [`Cache`] directly anyway). Once the cache has been
populated, the calculated dimensions can be retrieved from it.

# Literature
<a id="v1r_fb_1">\[1\]</a>
Mathis, S.: Permanentmagneterregte Line-Start-Antriebe in Ferrittechnik,
PhD thesis, Shaker, 2019, URL:
<https://kluedo.ub.rptu.de/frontdoor/index/index/docId/8192>

# Examples

The following example creates the rotary core shown in the first image of this
documentation and compares the core surface area with and without the flux
barrier.

```
use std::f64::consts::FRAC_PI_2;
use std::sync::Arc;

use approxim::assert_abs_diff_eq;
use stem_core::prelude::*;

// Create the core without flux barrier first
let mut core: RotCore = RotCoreBuilder {
    air_gap_radius: Length::new::<millimeter>(40.0),
    yoke_radius: Length::new::<millimeter>(19.0),
    axial_length: Length::new::<millimeter>(165.0),
    axial_coil_overhang: Length::new::<millimeter>(0.0),
    iron_fill_factor: 1.0,
    material: Arc::new(Material::default()),
    pole_pairs: 3,
    skew_angle: 0.0,
    air_gap: Box::new(PlainAirGap::default()),
    flux_barrier: None, // Flux barrier will be added later.
}.try_into().expect("valid dimensions");

// Core surface area without flux barrier
assert_abs_diff_eq!(core.cross_section_area().get::<square_millimeter>(), 3892.433, epsilon=1e-3);

// Add the flux barrier
let fb = V1rFluxBarrier {
    yoke_distance: Length::new::<millimeter>(3.0),
    relief_path_air_gap_width: Length::new::<millimeter>(2.0),
    relief_path_length: Length::new::<millimeter>(4.0),
    relief_path_width: Length::new::<millimeter>(2.0),
    opening_angle: FRAC_PI_2,
    magnet_space_width: Length::new::<millimeter>(6.0),
    magnet_space_height: Length::new::<millimeter>(13.0),
    glue_gap: Length::new::<millimeter>(0.2),
    leakage_path_width: Length::new::<millimeter>(1.0),
    magnet_material: Some(Arc::new(Material::default())),
    cache: None,
};
core.set_flux_barrier(Some(Box::new(fb))).expect("is compatible to core");

// Core surface area is now considerably smaller due to the cutouts
assert_abs_diff_eq!(core.cross_section_area().get::<square_millimeter>(), 2516.336, epsilon=1e-3);

// Cache has been populated and data can be read out.
let binding = core.flux_barrier().expect("exists");
let any = binding as &dyn std::any::Any;
let fb_read_out = any.downcast_ref::<V1rFluxBarrier>().expect("is a V1rFluxBarrier");
let cache = fb_read_out.cache.as_ref().expect("has been populated");
assert_abs_diff_eq!(cache.pt_corner_relief_q, [0.004, 0.022], epsilon=1e-3);
```
 */
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct V1rFluxBarrier {
    /// Distance between the yoke surface and the center of the "V". Must be
    /// positive (`yoke_distance > 0 m`).
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub yoke_distance: Length,
    /// Width of the air gap part in the relief path. Must not be negative
    /// (`relief_path_air_gap_width >= 0 m`). If set to zero, no relief path
    /// exists.
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub relief_path_air_gap_width: Length,
    /// Length of the core material part in the relief path. Must not be
    /// negative (`relief_path_length >= 0 m`).
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub relief_path_length: Length,
    /// Width of the relief path. Must not be negative (`relief_path_width >=
    /// 0 m`). If set to zero, no relief path exists.
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub relief_path_width: Length,
    /// Opening angle between the sides of the V. Must be between zero and pi
    /// (`0 <= opening_angle <= PI`).
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_angle"))]
    pub opening_angle: f64,
    /// Width of the space available for an interior magnet. Must be positive
    /// (`magnet_space_width > 0 m`).
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub magnet_space_width: Length,
    /// Height of the space available for an interior magnet. Must be positive
    /// (`magnet_space_height > 0 m`).
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub magnet_space_height: Length,
    /// Glue gap width. The glue gap is an optional "margin" between the magnet
    /// and the flux barrier sides and can be used to provide space for glue and
    /// easier assembly. Must not be negative (`glue_gap >= 0 m`).
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub glue_gap: Length,
    /// Width of the leakage path between the sides of the V and the air gap.
    /// Must be positive (`leakage_path_width > 0 m`).
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub leakage_path_width: Length,
    /// Material of the magnet, if the flux barrier has one.
    ///
    /// If a material is given, a [`BlockMagnet`] is created by
    /// [`FluxBarrier::combine`] whose dimensions are defined by
    /// [`V1rFluxBarrier::magnet_space_width`] and
    /// [`V1rFluxBarrier::magnet_space_height`]. This
    /// magnet can be accessed with [`V1rFluxBarrier::magnet`] or via
    /// [`FluxBarrier::magnet_assemblies`]. Changing this field after
    /// [`FluxBarrier::combine`] does not magically create a new magnet. In
    /// practice, this doesn't really matter, since a flux barrier is not meant
    /// to be used stand-alone and [`FluxBarrier::combine`] runs anyway when
    /// inserting the flux barrier into a core.
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serialize_opt_arc_link",
            deserialize_with = "deserialize_opt_arc_link"
        )
    )]
    pub magnet_material: Option<Arc<Material>>,
    /// Geometry data generated from [`FluxBarrier::combine`]. Set this field to
    /// [`None`] when building a new [`V1rFluxBarrier`] instance.
    ///
    /// If this field is not [`None`], the [`Cache`] holds data resulting from
    /// the combination of [`V1rFluxBarrier`] with a [`CoreRef`]. This data
    /// might be partially public and partially internal information.
    ///
    /// See the docstring of [`Cache`] for more.
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub cache: Option<Cache>,
}

/// A struct resulting from combining a [`V1rFluxBarrier`] with a [`CoreRef`]
/// via [`FluxBarrier::combine`].
///
/// This struct is created by applying [`FluxBarrier::combine`] to a
/// [`V1rFluxBarrier`] and is then placed into the [`V1rFluxBarrier::cache`]
/// field. It caches information from the combination procedure which is
/// expensive to calculate; some of this information might be public and other
/// might be private. Therefore, this struct cannot be created on its own. It is
/// overwritten each time a [`V1rFluxBarrier`] is combined with a [`CoreRef`],
/// therefore it makes no sense to move it from one [`V1rFluxBarrier`] to
/// another one.
#[derive(Debug, Clone)]
pub struct Cache {
    /// Right-side corner of the "lip" reaching into the relief path.
    pub pt_relief_path_lip: [f64; 2],
    /// Right-side corner at the inner (d-axis) transition from relief path to
    /// magnet space.
    pub pt_magnet_relief_d: [f64; 2],
    /// Right-side corner at the inner (d-axis) transition from magnet space to
    /// leakage area.
    pub pt_magnet_leakage_d: [f64; 2],
    /// Right-side corner at the outer (q-axis) transition from leakage area to
    /// magnet space.
    pub pt_magnet_leakage_q: [f64; 2],
    /// Lower q-axis corner of the right V-leg. Might be equal to
    /// [`Cache::pt_corner_relief_q`] if [`V1rFluxBarrier::magnet_space_width`]
    /// is small enough.
    pub pt_magnet_relief_q: [f64; 2],
    /// Right-side corner at the outer (q-axis) transition from magnet space to
    /// relief path. Might be equal to [`Cache::pt_magnet_relief_q`] if
    /// [`V1rFluxBarrier::magnet_space_width`] is small enough.
    pub pt_corner_relief_q: [f64; 2],
    /// Segment of the flux barrier contour which borders the leakage path.
    pub leakage_segment: Segment,
    /// Number of pole pairs (copied from the `core` argument of
    /// [`FluxBarrier::combine`]).
    pub pole_pairs: u16,
    magnets: Option<[MagnetAssembly; 1]>,
}

impl V1rFluxBarrier {
    /// Returns true if the center "lip" is a closed flux leakage path (without
    /// an air part).
    pub fn middle_path_is_closed(&self) -> bool {
        return self.relief_path_air_gap_width <= Length::new::<meter>(0.0);
    }

    /// Returns the total magnet space width.
    ///
    /// This is [`V1rFluxBarrier::magnet_space_width`] plus twice the
    /// [`V1rFluxBarrier::glue_gap`].
    pub fn total_magnet_space_width(&self) -> Length {
        return self.magnet_space_width + 2.0 * self.glue_gap;
    }

    /// Returns the total magnet space height.
    ///
    /// This is [`V1rFluxBarrier::magnet_space_height`] plus twice the
    /// [`V1rFluxBarrier::glue_gap`].
    pub fn total_magnet_space_height(&self) -> Length {
        return self.magnet_space_height + 2.0 * self.glue_gap;
    }

    /// Returns the distance between the barrier and the q-axis at the yoke
    /// height (where [`Cache::pt_magnet_relief_q`] is), if
    /// [`V1rFluxBarrier::cache`] is populated. Otherwise, this
    /// value is zero.
    pub fn distance_to_q_axis_at_yoke(&self) -> Length {
        match self.cache.as_ref() {
            Some(c) => Length::new::<meter>(dist_to_q_axis(c.pt_magnet_relief_q, c.pole_pairs)),
            None => Length::new::<meter>(0.0),
        }
    }

    /// Returns the distance between the barrier and the q-axis at the air gap
    /// height (where [`Cache::pt_magnet_leakage_q`] is), if
    /// [`V1rFluxBarrier::cache`] is populated. Otherwise, this
    /// value is zero.
    pub fn distance_to_q_axis_at_air_gap(&self) -> Length {
        match self.cache.as_ref() {
            Some(c) => Length::new::<meter>(dist_to_q_axis(c.pt_magnet_leakage_q, c.pole_pairs)),
            None => Length::new::<meter>(0.0),
        }
    }

    /// Returns the interior [`BlockMagnet`], if the flux barrier holds one.
    ///    
    /// If the cache has been created (i.e. if [`FluxBarrier::combine`] has been
    /// called) and if [`V1rFluxBarrier::magnet_material`] isn't `None`, a
    /// [`BlockMagnet`] is stored in the cache and can be accessed either
    /// indirectly with [`FluxBarrier::magnet_assemblies`] or directly with
    /// this method.
    pub fn magnet(&self) -> Option<&BlockMagnet> {
        self.cache
            .as_ref()
            .map(|c| {
                let magnets = (&c.magnets).as_ref()?;
                (magnets[0].magnet() as &dyn std::any::Any).downcast_ref::<BlockMagnet>()
            })
            .flatten()
    }

    fn combine_rot(&mut self, core: &RotCore) -> Result<Vec<Contour>, Error> {
        let zero = Length::new::<meter>(0.0);
        compare_variables!(val zero < self.yoke_distance)?;
        compare_variables!(val zero <= self.relief_path_air_gap_width)?;
        compare_variables!(val zero <= self.relief_path_length)?;
        compare_variables!(val zero <= self.relief_path_width)?;
        compare_variables!(0.0 <= self.opening_angle <= PI)?;

        let yoke_radius = core.yoke_radius();
        let air_gap_radius = core.air_gap_radius();
        let m = if core.is_outer() { -1.0 } else { 1.0 };

        // Calculate the vertices for the right side of the shape(s)
        let relief_path_radius = (yoke_radius + m * self.yoke_distance).get::<meter>();
        let middle_radius = relief_path_radius + m * self.relief_path_air_gap_width.get::<meter>();
        let pt_relief_path_lip = [
            0.5 * self.relief_path_width.get::<meter>(),
            (middle_radius.powi(2) - (0.5 * self.relief_path_width.get::<meter>()).powi(2)).sqrt(),
        ];

        let pt_magnet_relief_d = [
            pt_relief_path_lip[0],
            pt_relief_path_lip[1] + m * self.relief_path_length.get::<meter>(),
        ];
        let angle = FRAC_PI_2 - 0.5 * self.opening_angle;

        let x = self.total_magnet_space_height().get::<meter>() * angle.cos();
        let y = m * self.total_magnet_space_height().get::<meter>() * angle.sin();
        let pt_magnet_leakage_d = [pt_magnet_relief_d[0] + x, pt_magnet_relief_d[1] + y];

        let x = self.total_magnet_space_width().get::<meter>() * angle.sin();
        let y = -self.total_magnet_space_width().get::<meter>() * angle.cos();

        let pt_magnet_leakage_q = [pt_magnet_leakage_d[0] + x, pt_magnet_leakage_d[1] + y];
        let pt_magnet_relief_q = [pt_magnet_relief_d[0] + x, pt_magnet_relief_d[1] + y];

        let relief_path_circle = ArcSegment::circle([0.0, 0.0], relief_path_radius)?;

        // 10.0 * relief_path_radius is a placeholder for "large value which is
        // guaranteed to intersect with the relief_path_circle if
        // pt_magnet_relief_q is inside the relief_path_circle".
        let ls = LineSegment::from_start_angle_length(
            pt_magnet_relief_q,
            angle + FRAC_PI_2,
            10.0 * relief_path_radius,
        )?;

        let pt_corner_relief_q = match relief_path_circle.intersections_primitive(&ls) {
            PrimitiveIntersections::Zero => {
                // pt_magnet_relief_q is outside the circle
                let ls = LineSegment::from_start_angle_length(
                    pt_magnet_relief_q,
                    angle,
                    -10.0 * relief_path_radius,
                )?;

                match relief_path_circle.intersections_primitive(&ls) {
                    PrimitiveIntersections::Zero => pt_magnet_relief_q,
                    PrimitiveIntersections::One(i) => i,
                    PrimitiveIntersections::Two([i1, i2]) => {
                        let pt = pt_magnet_relief_q;
                        if (i1[0] - pt[0]).powi(2) + (i1[1] - pt[1]).powi(2)
                            < (i2[0] - pt[0]).powi(2) + (i2[1] - pt[1]).powi(2)
                        {
                            i1
                        } else {
                            i2
                        }
                    }
                }
            }
            PrimitiveIntersections::One(i) => i,
            // Should not occur, but might still happen due to floating point
            // rounding errors. Just take the intersection next to pt_magnet_relief_q
            PrimitiveIntersections::Two([i1, i2]) => {
                let pt = pt_magnet_relief_q;
                if (i1[0] - pt[0]).powi(2) + (i1[1] - pt[1]).powi(2)
                    < (i2[0] - pt[0]).powi(2) + (i2[1] - pt[1]).powi(2)
                {
                    i1
                } else {
                    i2
                }
            }
        };

        // Calculate the vertices of the leakage path, starting with the point on the
        // inner side of the V
        let leakage_segment: Segment = if let Some(ags) =
            (core.air_gap() as &dyn std::any::Any).downcast_ref::<SlottedAirGap>()
        {
            /*
            Find the slot whose slot bottom center is closest to the middle of the line
            pt_magnet_leakage_d - pt_magnet_leakage_q
             */
            let x = 0.5 * (pt_magnet_leakage_d[0] + pt_magnet_leakage_q[0]);
            let y = 0.5 * (pt_magnet_leakage_d[1] + pt_magnet_leakage_q[1]);
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

            let slot_bottom_width = ags.slot().width_at(ags.slot().height()).get::<meter>();
            let mut leakage_segment = LineSegment::new(
                [
                    closest_slot_bottom[0] - 0.5 * slot_bottom_width,
                    closest_slot_bottom[1] - m * self.leakage_path_width.get::<meter>(),
                ],
                [
                    closest_slot_bottom[0] + 0.5 * slot_bottom_width,
                    closest_slot_bottom[1] - m * self.leakage_path_width.get::<meter>(),
                ],
            )?;
            leakage_segment.rotate(closest_slot_bottom, slot_angle - FRAC_PI_2);
            leakage_segment.into()
        } else {
            let leakage_radius = (air_gap_radius - m * self.leakage_path_width).get::<meter>();
            let leakage_circle = ArcSegment::circle([0.0, 0.0], leakage_radius)?;

            let ext_d_axis = LineSegment::from_start_angle_length(
                pt_magnet_leakage_d,
                angle,
                10.0 * air_gap_radius.get::<meter>(),
            )?;
            let start_d = match leakage_circle.intersections_primitive(&ext_d_axis) {
                PrimitiveIntersections::Zero => pt_magnet_leakage_d,
                PrimitiveIntersections::One(i) => i,
                PrimitiveIntersections::Two([i1, i2]) => {
                    let pt = pt_magnet_leakage_d;
                    if (i1[0] - pt[0]).powi(2) + (i1[1] - pt[1]).powi(2)
                        < (i2[0] - pt[0]).powi(2) + (i2[1] - pt[1]).powi(2)
                    {
                        i1
                    } else {
                        i2
                    }
                }
            };

            let ext_q_axis = LineSegment::from_start_angle_length(
                pt_magnet_leakage_q,
                angle,
                10.0 * air_gap_radius.get::<meter>(),
            )?;
            let start_q = match leakage_circle.intersections_primitive(&ext_q_axis) {
                PrimitiveIntersections::Zero => pt_magnet_leakage_q,
                PrimitiveIntersections::One(i) => i,
                PrimitiveIntersections::Two([i1, i2]) => {
                    let pt = pt_magnet_leakage_q;
                    if (i1[0] - pt[0]).powi(2) + (i1[1] - pt[1]).powi(2)
                        < (i2[0] - pt[0]).powi(2) + (i2[1] - pt[1]).powi(2)
                    {
                        i1
                    } else {
                        i2
                    }
                }
            };

            ArcSegment::from_start_stop_center_radius(
                start_d,
                start_q,
                [0.0, 0.0],
                leakage_radius,
                false,
            )?
            .into()
        };

        // =====================================================================
        // Populate the cache

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
            pt_relief_path_lip,
            pt_magnet_relief_d,
            pt_magnet_leakage_d,
            pt_magnet_leakage_q,
            pt_magnet_relief_q,
            pt_corner_relief_q,
            leakage_segment: leakage_segment.clone(),
            pole_pairs: core.pole_pairs(),
            magnets,
        });

        // =====================================================================
        // Create the contour(s) for a single pole pair

        let mut ps = Polysegment::new();

        let middle_path_is_closed = self.relief_path_air_gap_width.get::<meter>() == 0.0;
        let capacity = if middle_path_is_closed {
            core.poles()
        } else {
            core.pole_pairs()
        };
        let mut contours: Vec<Contour> = Vec::with_capacity(capacity.into());

        if middle_path_is_closed {
            let segment = LineSegment::new(pt_relief_path_lip, pt_magnet_relief_d)?;
            ps.push_back(segment.into());
        } else {
            let relief_path_radius =
                (core.yoke_radius() + m * self.yoke_distance + m * self.relief_path_air_gap_width)
                    .get::<meter>();

            if let Ok(segment) = ArcSegment::from_start_stop_center_radius(
                [0.0, relief_path_radius],
                pt_relief_path_lip,
                [0.0, 0.0],
                relief_path_radius,
                false,
            ) {
                ps.push_back(segment.into());
            }
        }

        let segment = LineSegment::new(pt_magnet_relief_d, pt_magnet_leakage_d)?;
        ps.push_back(segment.into());
        ps.push_back(leakage_segment);
        ps.extend_back(pt_magnet_leakage_q);
        ps.extend_back(pt_magnet_relief_q);
        ps.extend_back(pt_corner_relief_q);

        if middle_path_is_closed {
            if let Ok(segment) = ArcSegment::from_start_stop_center_radius(
                pt_corner_relief_q,
                pt_relief_path_lip,
                [0.0, 0.0],
                relief_path_radius,
                true,
            ) {
                ps.push_back(segment.into());
            }
        } else {
            let relief_path_radius = (core.yoke_radius() + m * self.yoke_distance).get::<meter>();
            if let Ok(segment) = ArcSegment::from_start_stop_center_radius(
                pt_corner_relief_q,
                [0.0, relief_path_radius],
                [0.0, 0.0],
                relief_path_radius,
                true,
            ) {
                ps.push_back(segment.into());
            }
        }

        // Mirror the segment_chain along the y-axis, then reverse the mirrored
        // segment_chain, then connect it with the orginial. This forms the
        // contour of the flux barrier.
        let mut ps_mirror = ps.clone();
        ps.line_reflection([0.0, 0.0], [0.0, 1.0]);
        ps.reverse();

        if middle_path_is_closed {
            contours.push(ps.into());
            contours.push(ps_mirror.into());
        } else {
            ps.append(&mut ps_mirror);
            contours.push(ps.into());
        }

        // Rotate flux barrier clockwise by 90°-180°/(2 p) to have the negative
        // q-axis start in the x-axis (d-axis at 90° electrical)
        let q_axis_alignment_angle = FRAC_PI_2 * (1.0 - 2.0 / (core.poles() as f64));
        contours
            .iter_mut()
            .for_each(|c| c.rotate([0.0, 0.0], q_axis_alignment_angle));

        // Repeat the contours over all pole pairs
        for p in 1..core.poles() {
            let rot_angle = p as f64 * TAU / core.poles() as f64;

            let mut c0 = contours[0].clone();
            c0.rotate([0.0, 0.0], rot_angle);
            contours.push(c0);

            if middle_path_is_closed {
                let mut c1 = contours[1].clone();
                c1.rotate([0.0, 0.0], rot_angle);
                contours.push(c1);
            }
        }

        return Ok(contours);
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl FluxBarrier for V1rFluxBarrier {
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
            None => return Magnets::from_iter([].into_iter()),
        };
        let magnet = match self.magnet() {
            Some(m) => m,
            None => return Magnets::from_iter([].into_iter()),
        };

        let mut shapes: Vec<PositionedMagnetShape> = if split {
            magnet
                .north_south_shapes()
                .into_iter()
                .enumerate()
                .map(|(i, m)| PositionedMagnetShape {
                    shape: m.into_owned(),
                    is_north: i.is_odd(),
                    magnet_type: 0,
                })
                .collect()
        } else {
            vec![PositionedMagnetShape {
                shape: magnet.shape().into_owned(),
                is_north: true,
                magnet_type: 0,
            }]
        };

        let angle = FRAC_PI_2 - 0.5 * self.opening_angle;

        let gg = self.glue_gap.get::<meter>();
        let x = 0.5 * (c.pt_magnet_relief_d[0] + c.pt_magnet_leakage_d[0]) + gg * angle.sin();
        let y = 0.5 * (c.pt_magnet_relief_d[1] + c.pt_magnet_leakage_d[1]) - gg * angle.cos();
        let radius = (x.powi(2) + y.powi(2)).sqrt();

        // Correct for the fact that the magnet origin is not at the
        // radius after the shift by x
        let h = radius - (radius.powi(2) - x.powi(2)).sqrt();

        shapes.iter_mut().for_each(|s| {
            s.rotate([0.0, 0.0], angle + PI);
            s.translate([x, -h]);
            s.line_reflection([0.0, 0.0], [0.0, 1.0]);
        });

        for i in 0..shapes.len() {
            let mut shape = shapes[i].clone();
            shape.line_reflection([0.0, 0.0], [0.0, 1.0]);
            shapes.push(shape);
        }

        return MagnetsPeriodic::<false>::new(
            Length::new::<meter>(radius * TAU),
            shapes,
            core.poles().into(),
            core.d_axis_offset(),
        )
        .into();
    }

    fn combine(&mut self, core: CoreRef<'_>) -> Result<Vec<Contour>, Error> {
        match core {
            CoreRef::Lin(_) => Err(Error::IncompatibleToLinCore("V1rFluxBarrier")),
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
