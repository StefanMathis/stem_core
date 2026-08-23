/*!
This module provides the [`Spoke1FluxBarrier`] struct and the
[`Spoke1HeightSplit`] helper enum. When combined with a [`RotCore`], the
resulting shape resembles a wheel with spokes, hence the name. As shown in the
image below, this flux barrier might hold interior magnets.
 */
#![cfg_attr(feature = "doc-images", doc = "")]
#![cfg_attr(
    feature = "doc-images",
    doc = "![Linear and rotary core with a Spoke1FluxBarrier][lin_and_rot_core_spoke1.svg]"
)]
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("lin_and_rot_core_spoke1.svg", "docs/img/lin_and_rot_core_spoke1.svg"),
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
    magnets::{Magnets, MagnetsPeriodic},
};

#[cfg(feature = "serde")]
use serde_mosaic::{deserialize_opt_arc_link, serialize_opt_arc_link};

/**
A helper enum for creating a [`Spoke1FluxBarrier`] from either the
`magnet_space_height` or `relief_path_width`.

As evident from the drawing below, `magnet_space_height` and `relief_path_width`
are not independent parameters, but are related via the following equation:

`magnet_space_height + relief_path_width =`[`CoreExt::yoke_height`]`- 2 *`
[`Spoke1FluxBarrier::glue_gap`]
[`CoreExt::yoke_height`]`-`[`Spoke1FluxBarrier::air_gap_leakage_path_width`]`-`
[`Spoke1FluxBarrier::yoke_leakage_path_width`]

*/
#[doc = ""]
#[cfg_attr(feature = "doc-images", doc = "![Spoke1 drawing][cad_spoke1]")]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image("cad_spoke1", "docs/img/cad_spoke1.svg")
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

Hence, it is sufficient to specify only one of them in
[`Spoke1FluxBarrier::height_split`] via this enum. The second parameter can then
be calculated with [`Spoke1HeightSplit::height_and_width`] during
[`Spoke1FluxBarrier::combine`].
*/
#[derive(Clone, Debug, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Spoke1HeightSplit {
    /// Definition of the `magnet_space_height`, `relief_path_width` is then
    /// calculated with [`Spoke1HeightSplit::height_and_width`]. Must be
    /// positive (`magnet_space_height > 0 m`).
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    MagnetSpaceHeight(Length),
    /// Definition of the `relief_path_width`, `magnet_space_height` is then
    /// calculated with [`Spoke1HeightSplit::height_and_width`]. Must not be
    /// negative (`relief_path_width >= 0 m`). If set to zero, the
    /// [`Spoke1FluxBarrier`] doesn't have a relief path.
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    ReliefPathWidth(Length),
}

impl Spoke1HeightSplit {
    /// Calculates `[magnet_space_height, relief_path_width]` from the total
    /// available height for both.
    ///
    /// The `total_height` is the sum of `magnet_space_height` and
    /// `relief_path_width`. One of the summands is given by `self`, and this
    /// method calculates the other summand using that relationship. The
    /// `total_height` itself defined as:
    ///
    /// `total_height =`[`CoreExt::yoke_height`]`- 2 *`
    /// [`Spoke1FluxBarrier::glue_gap`]`-`
    /// [`Spoke1FluxBarrier::air_gap_leakage_path_width`]`-`
    /// [`Spoke1FluxBarrier::yoke_leakage_path_width`]
    ///
    /// # Examples
    ///
    /// ```
    /// use approxim::assert_abs_diff_eq;
    /// use stem_core::prelude::*;
    ///
    /// let height_split = Spoke1HeightSplit::MagnetSpaceHeight(Length::new::<millimeter>(10.0));
    /// let [magnet_space_height, relief_path_width] = height_split.height_and_width(Length::new::<millimeter>(11.0));
    /// assert_abs_diff_eq!(magnet_space_height.get::<millimeter>(), 10.0, epsilon=1e-10);
    /// assert_abs_diff_eq!(relief_path_width.get::<millimeter>(), 1.0, epsilon=1e-10);
    ///
    /// let height_split = Spoke1HeightSplit::ReliefPathWidth(Length::new::<millimeter>(2.0));
    /// let [magnet_space_height, relief_path_width] = height_split.height_and_width(Length::new::<millimeter>(10.0));
    /// assert_abs_diff_eq!(magnet_space_height.get::<millimeter>(), 8.0, epsilon=1e-10);
    /// assert_abs_diff_eq!(relief_path_width.get::<millimeter>(), 2.0, epsilon=1e-10);
    /// ```
    pub fn height_and_width(&self, total_height: Length) -> [Length; 2] {
        match self {
            Spoke1HeightSplit::MagnetSpaceHeight(magnet_space_height) => {
                [*magnet_space_height, total_height - *magnet_space_height]
            }
            Spoke1HeightSplit::ReliefPathWidth(relief_path_width) => {
                [total_height - *relief_path_width, *relief_path_width]
            }
        }
    }
}

/**
A flux barrier with rectangular cutouts which resembles the spokes of a wheel
when combined with a [`RotCore`].

 */
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Linear and rotary core with a Spoke1FluxBarrier][lin_and_rot_core_spoke1]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image(
        "lin_and_rot_core_spoke1",
        "docs/img/lin_and_rot_core_spoke1.svg"
    )
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**
_This image was produced with `examples/flux_barrier_plots.rs`._

This is a pretty basic flux barrier type which - as shown in the image above -
is compatible with both [`LinCore`] and [`RotCore`]. It consists of a
rectangular magnet space which is perpendicular to the air gap surface. Inside
the magnet space, it can hold a single [`BlockMagnet`], hence the "1" in the
name (see [`Spoke1FluxBarrier::magnet_material`]).

In addition, it can have a "relief path" between the magnet space and the air
gap leakage path. A relief path is a leakage path which partially consists of
air and therefore doesn't exist due to mechanical reasons, but instead protects
the magnet against large magnetic fields originating from the stator by
providing a "relief valve" for the magnetic flux while offering a high enough
magnetic resistance so the magnet flux doesn't get short-circuited. For a
throughout explanation of the concept, see [\[1\]](#spoke1_fb_1).

Constructing a [`Spoke1FluxBarrier`] requires specifying some geometric
dimensions, while other dimensions are calculated later when
[combined](FluxBarrier::combine) with a magnetic core. The drawing below shows
the definition of all dimensions of this flux barrier type:
 */
#[doc = ""]
#[cfg_attr(feature = "doc-images", doc = "![Spoke1 drawing][cad_spoke1]")]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image("cad_spoke1", "docs/img/cad_spoke1.svg")
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

During the combination, the calculated dimensions are stored within a [`Cache`]
which is then put into [`Spoke1FluxBarrier::cache`]. When constructing a
[`Spoke1FluxBarrier`], this field therefore should be simply set to `None` (it
is not possible to create a [`Cache`] directly anyway). Once the cache has been
populated, the calculated dimensions can be retrieved from it.

# Literature
<a id="spoke1_fb_1">\[1\]</a>
Mathis, S.: Permanentmagneterregte Line-Start-Antriebe in Ferrittechnik,
PhD thesis, Shaker, 2019, URL:
<https://kluedo.ub.rptu.de/frontdoor/index/index/docId/8192>

# Examples

The following example creates the rotary core shown in the first image of this
documentation and compares the core surface area with and without the flux
barrier.

```
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
let fb = Spoke1FluxBarrier {
    air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
    yoke_leakage_path_width: Length::new::<millimeter>(1.0),
    relief_path_air_gap_width: Length::new::<millimeter>(3.0),
    magnet_space_width: Length::new::<millimeter>(10.0),
    height_split: Spoke1HeightSplit::ReliefPathWidth(Length::new::<millimeter>(1.0)),
    glue_gap: Length::new::<millimeter>(0.2),
    magnet_material: Some(Default::default()),
    cache: None, // As stated, must be initialized to None.
};
core.set_flux_barrier(Some(Box::new(fb))).expect("is compatible to core");

// Core surface area is now considerably smaller due to the cutouts
assert_abs_diff_eq!(core.cross_section_area().get::<square_millimeter>(), 2738.480, epsilon=1e-3);

// Cache has been populated and data can be read out.
let binding = core.flux_barrier().expect("exists");
let any = binding as &dyn std::any::Any;
let fb_read_out = any.downcast_ref::<Spoke1FluxBarrier>().expect("is a Spoke1FluxBarrier");
let cache = fb_read_out.cache.as_ref().expect("has been populated");
assert_abs_diff_eq!(cache.relief_path_width.get::<millimeter>(), 1.0, epsilon=1e-3);
```
 */
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Spoke1FluxBarrier {
    /// Width of the air gap leakage path. Must be positive
    /// (`air_gap_leakage_path_width > 0 m`).
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub air_gap_leakage_path_width: Length,
    /// Width of the yoke leakage path. Must be positive
    /// (`yoke_leakage_path_width > 0 m`).
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub yoke_leakage_path_width: Length,
    /// Width of the air gap part in the relief path. Must not be negative
    /// (`relief_path_air_gap_width >= 0 m`). If set to zero, the relief path
    /// becomes part of the air gap leakage path.
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub relief_path_air_gap_width: Length,
    /// Width of the space available for an interior magnet. Must be positive
    /// (`magnet_space_width > 0 m`).
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub magnet_space_width: Length,
    /// Definition of either the magnet space height or the relief path width,
    /// the other information is then calculated by
    /// [`FluxBarrier::combine`]. See [`Spoke1HeightSplit`] for details.
    pub height_split: Spoke1HeightSplit,
    /// Glue gap width. The glue gap is an optional "margin" between the magnet
    /// and the flux barrier sides and can be used to provide space for glue and
    /// easier assembly. Must not be negative (`glue_gap >= 0 m`).
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub glue_gap: Length,
    /// Material of the magnet, if the flux barrier has one.
    ///
    /// If a material is given, a [`BlockMagnet`] is created by
    /// [`FluxBarrier::combine`] whose dimensions are defined by
    /// [`Spoke1FluxBarrier::magnet_space_width`] and
    /// [`Spoke1FluxBarrier::height_split`]. This
    /// magnet can be accessed with [`Spoke1FluxBarrier::magnet`] or via
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
    /// [`None`] when building a new [`Spoke1FluxBarrier`] instance.
    ///
    /// If this field is not [`None`], the [`Cache`] holds data resulting from
    /// the combination of [`Spoke1FluxBarrier`] with a [`CoreRef`]. This data
    /// might be partially public and partially internal information.
    ///
    /// See the docstring of [`Cache`] for more.
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub cache: Option<Cache>,
}

/// A struct resulting from combining a [`Spoke1FluxBarrier`] with a [`CoreRef`]
/// via [`FluxBarrier::combine`].
///
/// This struct is created by applying [`FluxBarrier::combine`] to a
/// [`Spoke1FluxBarrier`] and is then placed into the
/// [`Spoke1FluxBarrier::cache`] field. It caches information from the
/// combination procedure which is expensive to calculate; some of this
/// information might be public and other might be private. Therefore, this
/// struct cannot be created on its own. It is overwritten each time a
/// [`Spoke1FluxBarrier`] is combined with a [`CoreRef`], therefore it makes no
/// sense to move / clone it from one [`Spoke1FluxBarrier`] to another one.
///
/// The image below shows the dimensions of a [`Spoke1FluxBarrier`], some of
/// those dimensions are calculated and stored in the [`Cache`] struct. All
/// points (prefix `pt_`) use the coordinate system shown in the image.
#[doc = ""]
#[cfg_attr(feature = "doc-images", doc = "![Spoke1 drawing][cad_spoke1]")]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image("cad_spoke1", "docs/img/cad_spoke1.svg")
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
#[derive(Debug, Clone)]
pub struct Cache {
    /// Right-side corner of the [`Cache::yoke_leakage_segment`].
    pub pt_yoke_leakage: [f64; 2],
    /// Right-side corner of the transition from the magnet space to the relief
    /// path. If no relief path exists, this point is equal to
    /// [`Cache::pt_air_gap_leakage`].
    pub pt_inner_relief: [f64; 2],
    /// Right-side corner of the magnet space near the air gap. If no relief
    /// path exists, this point is equal to [`Cache::pt_air_gap_leakage`].
    pub pt_outer_relief: [f64; 2],
    /// Corner between relief path and [`Cache::air_gap_leakage_segment`]. If no
    /// relief path exists, this is the corner between the magnet space and
    /// the [`Cache::air_gap_leakage_segment`].
    pub pt_air_gap_leakage: [f64; 2],
    /// Height of the magnet space (derived from
    /// [`Spoke1FluxBarrier::height_split`]).
    pub magnet_space_height: Length,
    /// Relief path width (derived from
    /// [`Spoke1FluxBarrier::height_split`]).
    pub relief_path_width: Length,
    /// Segment of the flux barrier contour which borders the air gap leakage
    /// path.
    pub air_gap_leakage_segment: Segment,
    /// Segment of the flux barrier contour which borders the yoke leakage path.
    pub yoke_leakage_segment: Segment,
    /// Number of pole pairs (copied from the `core` argument of
    /// [`FluxBarrier::combine`]).
    pub pole_pairs: u16,
    magnets: Option<[MagnetAssembly; 1]>,
}

impl Spoke1FluxBarrier {
    /// Returns the total magnet space width.
    ///
    /// This is [`Spoke1FluxBarrier::magnet_space_width`] plus twice the
    /// [`Spoke1FluxBarrier::glue_gap`].
    pub fn total_magnet_space_width(&self) -> Length {
        return self.magnet_space_width + 2.0 * self.glue_gap;
    }

    /// Returns the total magnet space height.
    ///
    /// If the cache has been created (i.e. if [`FluxBarrier::combine`] has been
    /// called), this is [`Cache::magnet_space_height`] plus twice the
    /// [`Spoke1FluxBarrier::glue_gap`]. Otherwise, zero is returned as a
    /// default / placeholder value.
    pub fn total_magnet_space_height(&self) -> Length {
        return self
            .cache
            .as_ref()
            .map_or(Length::new::<meter>(0.0), |c| c.magnet_space_height)
            + 2.0 * self.glue_gap;
    }

    /// Returns the interior [`BlockMagnet`], if the flux barrier holds one.
    ///    
    /// If the cache has been created (i.e. if [`FluxBarrier::combine`] has been
    /// called) and if [`Spoke1FluxBarrier::magnet_material`] isn't `None`, a
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

    fn combine_lin(&mut self, core: &LinCore) -> Result<Vec<Contour>, Error> {
        let tooth_height = core.tooth_height();
        let yoke_height = core.yoke_height();
        let zero = Length::new::<meter>(0.0);
        let height_for_flux_barrier = yoke_height
            - 2.0 * self.glue_gap
            - self.air_gap_leakage_path_width
            - self.yoke_leakage_path_width;
        compare_variables!(val zero < height_for_flux_barrier)?;

        let [magnet_space_height, relief_path_width] =
            self.height_split.height_and_width(height_for_flux_barrier);
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

        let has_relief_path = match self.height_split {
            Spoke1HeightSplit::MagnetSpaceHeight(_) => true,
            Spoke1HeightSplit::ReliefPathWidth(width) => width > zero,
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

        let [magnet_space_height, relief_path_width] =
            self.height_split.height_and_width(height_for_flux_barrier);

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
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl FluxBarrier for Spoke1FluxBarrier {
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
                    is_north: i.is_even(),
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

        match core {
            CoreRef::Lin(lin_core) => {
                let pole_width = lin_core.width().get::<meter>() / lin_core.poles() as f64;
                let shift = [
                    0.5 * magnet.thickness().get::<meter>() + 0.5 * pole_width,
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
                return MagnetsPeriodic::<true>::new(
                    lin_core.air_gap_length(),
                    shapes,
                    core.poles().into(),
                    core.d_axis_offset(),
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

                return MagnetsPeriodic::<false>::new(
                    Length::new::<meter>(radius * TAU),
                    shapes,
                    core.poles().into(),
                    core.d_axis_offset(),
                )
                .into();
            }
        }
    }

    fn combine(&mut self, core: CoreRef<'_>) -> Result<Vec<Contour>, Error> {
        let zero = Length::new::<meter>(0.0);
        compare_variables!(val zero <= self.glue_gap)?;
        compare_variables!(val zero < self.magnet_space_width)?;
        match self.height_split {
            Spoke1HeightSplit::MagnetSpaceHeight(magnet_space_height) => {
                compare_variables!(val zero <= magnet_space_height)?;
            }
            Spoke1HeightSplit::ReliefPathWidth(relief_path_width) => {
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

    fn d_axis_offset(&self, core: CoreRef<'_>) -> f64 {
        match core {
            CoreRef::Lin(_) => 0.0,
            CoreRef::Rot(_) => FRAC_PI_2,
        }
    }
}
