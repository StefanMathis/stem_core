/*!
A module providing iterators for positioning surface and interior magnets inside
or on magnetic cores.

Permanent magnets can be integrated into a magnetic cores in two different
fashions: They can be either mounted on the air gap surface ("surface magnets")
or inside cutouts ([`FluxBarrier`](crate::flux_barrier::FluxBarrier)s) within
the core. The following image shows both variants with the surface magnets using
arabic and the interior magnets using roman numbers:

 */
#![cfg_attr(feature = "doc-images", doc = "")]
#![cfg_attr(
    feature = "doc-images",
    doc = "![Surface and interior magnets][surface_and_interior_magnets.svg]"
)]
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("surface_and_interior_magnets.svg", "docs/img/surface_and_interior_magnets.svg"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/*!

There are various use cases where the exact positioning of surface and interior
magnets must be known, for example for FEM simulations (where the model is
derived directly from the geometry), for collision detection (i.e. can surface
magnets fit inside a given air gap) or for visualization as shown above.

The [`CoreExt`](crate::core::CoreExt) therefore provides the
[`CoreExt::surface_magnets`](crate::core::CoreExt::surface_magnets) and
[`CoreExt::interior_magnets`](crate::core::CoreExt::interior_magnets) methods
which return a [`Magnets`] iterator. This iterator returns
[`PositionedMagnetShape`]s which contain a magnet [`Shape`] positioned relative
to the origin of the core as well as some metadata.

[`Magnets`] itself is a wrapper around iterators such as [`MagnetsPeriodic`]
which implement `Iterator<Item=PositionedMagnetShape`>. It is possible to wrap
custom iterators via [`Magnets::from_iter`], which allows using any iterator
returning [`PositionedMagnetShape`]s for implementing
[`CoreExt::surface_magnets`](crate::core::CoreExt::surface_magnets) or
[`CoreExt::interior_magnets`](crate::core::CoreExt::interior_magnets).

`examples/magnet_plots.rs` demonstrates how to utilize the
[`Magnets`] iterator for creating the above image. The following snippet in
particular shows how to to draw the interior [`PositionedMagnetShape`]s:

```ignore
// "cr" is a cairo::Context
for (idx, m) in core.interior_magnets(true).enumerate() {
    m.as_drawable().draw(cr)?;
}
```
*/

use crate::planar_geo;
use num::Integer;
use planar_geo::prelude::*;
use std::f64::consts::{FRAC_PI_2, PI, TAU};
use stem_magnet::prelude::*;

/// A positioned magnet [`Shape`] with some metadata.
///
/// This struct is returned by the [`Magnets`] iterator and contains the
/// [`Shape`] of the zone positioned relative to the magnetic core which created
/// [`Magnets`] via the
/// [`CoreExt::surface_magnets`](crate::core::CoreExt::surface_magnets) or
/// [`CoreExt::interior_magnets`](crate::core::CoreExt::interior_magnets)
/// method. Additionally, it provides some metadata such as the polarity of the
/// magnet and the type index.
///
/// This struct implements the [`Transformation`] trait. The trait methods are
/// applied to [`PositionedZoneContour::contour`] using the implementation
/// of that trait for [`Contour`].
///
/// See the [module documentation](crate::magnets) for examples.
#[derive(Debug, Clone)]
pub struct PositionedMagnetShape {
    /// Positioned magnet shape.
    pub shape: Shape,
    /// Whether the shape represents a north or a south pole magnet.
    pub is_north: bool,
    /**
    A [`FluxBarrier`](crate::flux_barrier::FluxBarrier) can have multiple types
    of magnets. A slice of all the magnet types can be retrieved with the
    [`FluxBarrier::magnet_assemblies`](crate::flux_barrier::FluxBarrier::magnet_assemblies)
    method. This index can be used for retrieving the magnet type of `self` from
    that list. If the [`PositionedMagnetShape`] was created from an iterator
    over the surface magnets, this value is simply 0.
    */
    pub magnet_type: usize,
}

impl PositionedMagnetShape {
    /// Converts [`PositionedMagnetShape::shape`] into a [`Drawable`] using
    /// [`stem_magnet::NORTH_POLE_STYLE`] if [`PositionedMagnetShape::is_north`]
    /// is true and [`stem_magnet::SOUTH_POLE_STYLE`] otherwise.
    #[cfg(feature = "cairo")]
    pub fn into_drawable(self) -> Drawable {
        let style = if self.is_north {
            stem_magnet::NORTH_POLE_STYLE
        } else {
            stem_magnet::SOUTH_POLE_STYLE
        };
        return Drawable::new(self.shape, style);
    }

    /// Converts [`PositionedMagnetShape::shape`] into a [`Drawable`] using
    /// the provided `style`.
    #[cfg(feature = "cairo")]
    pub fn into_drawable_with_style(self, style: planar_geo::draw::Style) -> Drawable {
        Drawable::new(self.shape, style)
    }

    /// Wraps [`PositionedMagnetShape::shape`] into a [`DrawableRef`] using
    /// [`stem_magnet::NORTH_POLE_STYLE`] if [`PositionedMagnetShape::is_north`]
    /// is true and [`stem_magnet::SOUTH_POLE_STYLE`] otherwise.
    #[cfg(feature = "cairo")]
    pub fn as_drawable<'a>(&'a self) -> DrawableRef<'a> {
        let style = if self.is_north {
            stem_magnet::NORTH_POLE_STYLE
        } else {
            stem_magnet::SOUTH_POLE_STYLE
        };
        return DrawableRef::new(&self.shape, style);
    }

    /// Wraps [`PositionedMagnetShape::shape`] into a [`DrawableRef`] using
    /// the provided `style`.
    #[cfg(feature = "cairo")]
    pub fn as_drawable_with_style<'a>(&'a self, style: planar_geo::draw::Style) -> DrawableRef<'a> {
        return DrawableRef::new(&self.shape, style);
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

/**
An iterator which periodically repeats the surface or interior magnet assembly
over all poles.

This struct is essentially a builder for [`Magnets`] which repeats all
[`PositionedMagnetShape`]s of a surface or interior magnet assembly over the
poles of a core. It can be used for both linear (`LIN = true`) and rotary cores
(`LIN = false`). It implements `Into<Magnets>`.

The following image shows the principle for both a linear and a rotary core:
A magnet assembly (in this example a V-shape consisting of two split magnets,
so 4 [`PositionedMagnetShape`]s in total) is given to [`MagnetsPeriodic::new`].
It is then offset from the core reference point by `d_axis_offset` and repeated
equidistantly over all poles of the core (in this case, two times). This is done
by shifting and rotating the coordinate system and then simply placing the
assembly into the new coordinate system.
 */
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![How MagnetsPeriodic distributes the winding zones][cad_magnets_periodic]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image("cad_magnets_periodic", "docs/img/cad_magnets_periodic.svg")
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

This iterator can be used for both surface and interior magnet assemblies. It is
not strictly necessary to use the `air_gap_length` as the reference for
distributing the coordinate system. For example, for a
[`StraightIndentsAirGap`](crate::air_gap::StraightIndentsAirGap), it might be
easier to use the
[`indent_center_radius`](crate::air_gap::StraightIndentsAirGap::indent_center_radius)
instead and align the coordinate system of the `magnets` with it rather than
with the `air_gap_length`.
 */
#[derive(Debug, Clone)]
pub struct MagnetsPeriodic<const LIN: bool> {
    air_gap_length: Length,
    magnets: Vec<PositionedMagnetShape>,
    poles: usize,
    d_axis_offset: f64,
    index: usize,
}

impl<const LIN: bool> MagnetsPeriodic<LIN> {
    /// Creates a new [`MagnetsPeriodic`] for a linear or rotary core.
    ///
    /// - `air_gap_length`: The cross-section length along which the `magnets`
    ///   will be distributed. Especially for surface magnet assemblies, it is
    ///   often convenient to use
    ///   [`CoreExt::air_gap_length`](crate::core::CoreExt::air_gap_length) for
    ///   this (hence the name). But other values can be used as well, provided
    ///   that the `magnets` coordinate system is properly aligned to
    ///   `air_gap_length`.
    /// - `magnets`: The individual shapes of a surface or interior magnet
    ///   assembly belonging to a single pole. [`PositionedMagnetShape::shape`]
    ///   will be transformed when distributing those shapes over the `poles`.
    ///   If a pole is odd, [`PositionedMagnetShape::is_north`] will be inverted
    ///   (see example below). If an interior magnet assembly has multiple
    ///   different types of magnets, [`PositionedMagnetShape::magnet_type`]
    ///   needs to be set accordingly (it won't be changed by the iterator). See
    ///   [`FluxBarrier::magnet_assemblies`](crate::flux_barrier::FluxBarrier::magnet_assemblies)
    ///   for more information on this.
    /// - `poles`: Number of magnetic poles. The total number of
    ///   [`PositionedMagnetShape`] produced by this iterator is `poles *
    ///   magnets.len()`.
    /// - `d_axis_offset`: Offset of the d-axis to the core reference axis in
    ///   electrical radians. The core reference axis is the left side of the
    ///   cross section for a linear core and the negative x-axis for a rotary
    ///   core, see image below.
    ///
    /// In the following example, `magnets` consists of four individual
    /// [`PositionedMagnetShape`]s, `poles` is 2 and `d_axis_offset` is pi/2. In
    /// this particular case, the mechanical offset angle is therefore also pi/2
    /// (in the general case, it would be `pi/poles`).
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![How MagnetsPeriodic distributes the winding zones][cad_magnets_periodic]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "cad_magnets_periodic",
            "docs/img/cad_magnets_periodic.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    ///
    /// # Examples
    ///
    /// ```
    /// use stem_core::prelude::*;
    /// use planar_geo::prelude::*;
    ///
    /// // Dummy shapes
    /// let north_shape = Shape::from_outer(Contour::rectangle([0.0, 0.0], [1.0, 0.5])).expect("valid shape");
    /// let north = PositionedMagnetShape {shape: north_shape, is_north: true, magnet_type: 0};
    ///
    /// let south_shape = Shape::from_outer(Contour::rectangle([0.0, 0.5], [1.0, 1.0])).expect("valid shape");
    /// let south = PositionedMagnetShape {shape: south_shape, is_north: false, magnet_type: 0};
    ///
    /// let mut mags = MagnetsPeriodic::<true>::new(
    ///     Length::new::<millimeter>(100.0),
    ///     vec![north, south],
    ///     2,
    ///     0.0,
    /// );
    ///
    /// // First pole (positive pole)
    /// assert_eq!(mags.next().unwrap().is_north, true);
    /// assert_eq!(mags.next().unwrap().is_north, false);
    ///
    /// // Second pole (negative pole)
    /// assert_eq!(mags.next().unwrap().is_north, false);
    /// assert_eq!(mags.next().unwrap().is_north, true);
    ///
    /// // All poles are covered
    /// assert!(mags.next().is_none());
    /// ```
    pub fn new(
        air_gap_length: Length,
        magnets: Vec<PositionedMagnetShape>,
        poles: usize,
        d_axis_offset: f64,
    ) -> Self {
        Self {
            poles,
            air_gap_length,
            magnets,
            index: 0,
            d_axis_offset,
        }
    }

    /// Resets the iterator.
    ///
    /// After "resetting" the iterator, the next yielded item is again the first
    /// magnet.
    ///
    /// # Examples
    ///
    /// ```
    /// use stem_core::prelude::*;
    /// use planar_geo::prelude::*;
    ///
    /// // Dummy shapes
    /// let north_shape = Shape::from_outer(Contour::rectangle([0.0, 0.0], [1.0, 0.5])).expect("valid shape");
    /// let north = PositionedMagnetShape {shape: north_shape, is_north: true, magnet_type: 0};
    ///
    /// let south_shape = Shape::from_outer(Contour::rectangle([0.0, 0.5], [1.0, 1.0])).expect("valid shape");
    /// let south = PositionedMagnetShape {shape: south_shape, is_north: false, magnet_type: 0};
    ///
    /// let mut mags = MagnetsPeriodic::<true>::new(
    ///     Length::new::<millimeter>(100.0),
    ///     vec![north, south],
    ///     2,
    ///     0.0,
    /// );
    ///
    /// // First pole (positive pole)
    /// assert_eq!(mags.next().unwrap().is_north, true);
    /// assert_eq!(mags.next().unwrap().is_north, false);
    ///
    /// // Second pole (negative pole)
    /// assert_eq!(mags.next().unwrap().is_north, false);
    /// assert_eq!(mags.next().unwrap().is_north, true);
    ///
    /// // All poles are covered
    /// assert!(mags.next().is_none());
    ///
    /// // Now reset the iterator
    /// mags.reset();
    /// assert_eq!(mags.next().unwrap().is_north, true);
    /// ```
    pub fn reset(&mut self) {
        self.index = 0;
    }
}

impl<const LIN: bool> Iterator for MagnetsPeriodic<LIN> {
    type Item = PositionedMagnetShape;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.poles * self.magnets.len() {
            return None;
        }

        let current_pole = self.index / (self.magnets.len());

        // Index counter repeats from zero to the number of shapes per pole
        let pole_idx = self.index.rem_euclid(self.magnets.len());
        let shape_idx = pole_idx.rem_euclid(self.magnets.len());

        self.index = self.index + 1;

        let mut shape = self.magnets[shape_idx].clone();

        if LIN {
            let width_per_pole = self.air_gap_length.get::<meter>() / self.poles as f64;
            let offset = (self.d_axis_offset / PI + current_pole as f64) * width_per_pole;
            shape.translate([offset, 0.0]);
        } else {
            shape.translate([0.0, self.air_gap_length.get::<meter>() / TAU]);
            let angle_per_pole = TAU / self.poles as f64;
            let angle = angle_per_pole * (0.5 + current_pole as f64);
            shape.rotate([0.0, 0.0], -angle + self.d_axis_offset);
        }

        if current_pole.is_odd() {
            shape.is_north = !shape.is_north;
        }

        return Some(shape);
    }
}

// =============================================================================

/**
An iterator over the surface or interior [`PositionedMagnetShape`]s of a
magnetic core.

This iterator is created from the
[`CoreExt::surface_magnets`](crate::core::CoreExt::surface_magnets) or
[`CoreExt::interior_magnets`](crate::core::CoreExt::interior_magnets) methods
and returns the [`PositionedMagnetShape`]s for all surface / interior magnets
mounted on the core, starting with the magnets at the first pole and ending with
those of the last. Hence, [`PositionedMagnetShape::is_north`] is inverted for
every odd pole:

```
use std::sync::Arc;
use stem_core::prelude::*;

let fb = Spoke1FluxBarrier {
    air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
    yoke_leakage_path_width: Length::new::<millimeter>(1.0),
    relief_path_air_gap_width: Length::new::<millimeter>(3.0),
    magnet_space_width: Length::new::<millimeter>(10.0),
    height_split: Spoke1HeightSplit::ReliefPathWidth(Length::new::<millimeter>(1.0)),
    glue_gap: Length::new::<millimeter>(0.2),
    magnet_material: Some(Default::default()),
    cache: None,
};

let core: LinCore = LinCoreBuilder {
    height: Length::new::<millimeter>(15.0),
    width: Length::new::<millimeter>(200.0),
    axial_length: Length::new::<millimeter>(1.0),
    axial_coil_overhang: Length::new::<millimeter>(0.0),
    skew_angle: 0.0,
    iron_fill_factor: 1.0,
    material: Arc::new(Default::default()),
    pole_pairs: 2,
    air_gap: Box::new(PlainAirGap {
        num_segments: 0,
        air_gap_winding_height: Length::new::<millimeter>(8.0),
        winding_coverage: 0.7,
        starts_in_slot_middle: false,
        slots: 3,
    }),
    flux_barrier: Some(Box::new(fb)),
}
.try_into().expect("valid dimensions");

// To illustrate the inversion of PositionedMagnetShape::is_north, the magnets
// are not splitted in a north and south part.

// Surface magnets: Two magnets per pole -> 8 positioned shapes in total
let magnet = BreadLoafMagnet::new(
    Length::new::<millimeter>(165.0),
    Length::new::<millimeter>(20.0),
    Length::new::<millimeter>(10.0),
    Length::new::<millimeter>(50.0),
    Arc::new(Default::default()),
)
.unwrap();
let surface_mag_assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 2.try_into().unwrap());
let mut surface_magnets = core.surface_magnets(&surface_mag_assembly, false);

// First pole (north pole)
assert!(surface_magnets.next().unwrap().is_north);
assert!(surface_magnets.next().unwrap().is_north);

// Second pole (south pole)
assert!(!surface_magnets.next().unwrap().is_north);
assert!(!surface_magnets.next().unwrap().is_north);

// Third pole (north pole)
assert!(surface_magnets.next().unwrap().is_north);
assert!(surface_magnets.next().unwrap().is_north);

// Fourth pole (south pole)
assert!(!surface_magnets.next().unwrap().is_north);
assert!(!surface_magnets.next().unwrap().is_north);

// Core has eight surface magnets in total, hence the iterator is now exhausted
assert!(surface_magnets.next().is_none());

// Interior magnets: One magnet per pole -> 4 positioned shapes in total
let mut interior_magnets = core.interior_magnets(false);
assert!(interior_magnets.next().unwrap().is_north); // First pole (north pole)
assert!(!interior_magnets.next().unwrap().is_north); // Second pole (south pole)
assert!(interior_magnets.next().unwrap().is_north); // Third pole (north pole)
assert!(!interior_magnets.next().unwrap().is_north); // Fourth pole (south pole)

// Core has four interior magnets in total, hence the iterator is now exhausted
assert!(interior_magnets.next().is_none());
```

All predefined specialized iterators (such as [`MagnetsPeriodic`]) can be
converted into [`Magnets`] via its [`From`] implementations.

When implementing
[`AirGap::surface_magnets`](crate::air_gap::AirGap::surface_magnets) or
[`FluxBarrier::interior_magnets`](crate::flux_barrier::FluxBarrier::interior_magnets)
(which drive [`CoreExt::surface_magnets`](crate::core::CoreExt::surface_magnets)
and [`CoreExt::interior_magnets`](crate::core::CoreExt::interior_magnets), the
[`Magnets::from_iter`] method can be used for wrapping a custom iterator. The
custom iterator should produce the [`PositionedMagnetShape`]s in ascending pole
order as shown in the example.
 */
pub struct Magnets(MagnetsInner);

impl Magnets {
    /// Creates [`Magnets`] from a custom iterator.
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: Iterator<Item = PositionedMagnetShape> + 'static,
    {
        Self(MagnetsInner::Other(Box::new(iter)))
    }
}

enum MagnetsInner {
    MagnetsPeriodicLin(MagnetsPeriodic<true>),
    MagnetsPeriodicRot(MagnetsPeriodic<false>),
    Other(Box<dyn Iterator<Item = PositionedMagnetShape>>),
}

impl From<MagnetsInner> for Magnets {
    fn from(value: MagnetsInner) -> Self {
        Self(value)
    }
}

impl From<MagnetsPeriodic<true>> for Magnets {
    fn from(value: MagnetsPeriodic<true>) -> Self {
        MagnetsInner::MagnetsPeriodicLin(value).into()
    }
}

impl From<MagnetsPeriodic<false>> for Magnets {
    fn from(value: MagnetsPeriodic<false>) -> Self {
        MagnetsInner::MagnetsPeriodicRot(value).into()
    }
}

impl From<Box<dyn Iterator<Item = PositionedMagnetShape>>> for Magnets {
    fn from(value: Box<dyn Iterator<Item = PositionedMagnetShape>>) -> Self {
        MagnetsInner::Other(value).into()
    }
}

impl Iterator for Magnets {
    type Item = PositionedMagnetShape;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            MagnetsInner::MagnetsPeriodicLin(i) => i.next(),
            MagnetsInner::MagnetsPeriodicRot(i) => i.next(),
            MagnetsInner::Other(i) => i.next(),
        }
    }
}

#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Creation of the shapes of a surface magnet assembly][cad_create_surface_mag_assembly]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image(
        "cad_create_surface_mag_assembly",
        "docs/img/cad_create_surface_mag_assembly.svg"
    )
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
pub fn surface_magnet_assembly_shapes_lin(
    magnet_assembly: &MagnetAssembly,
    split: bool,
    coverage_single_magnet: Option<Length>,
) -> Vec<PositionedMagnetShape> {
    let mut proto_shapes: Vec<PositionedMagnetShape> = if split {
        magnet_assembly
            .magnet()
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
            shape: magnet_assembly.magnet().shape().into_owned(),
            is_north: true,
            magnet_type: 0,
        }]
    };

    for s in proto_shapes.iter_mut() {
        s.line_reflection([0.0, 0.0], [1.0, 0.0]);
    }

    let magnet_coverage = coverage_single_magnet
        .map(|l| l.get::<meter>())
        .unwrap_or_else(|| {
            BoundingBox::from_bounded_entities(proto_shapes.iter().map(|p| &p.shape))
                .map(|m| m.width())
                .unwrap_or(0.0)
        });

    let mut shapes = Vec::with_capacity(magnet_assembly.num_tangential() * proto_shapes.len());

    for tan_idx in 0..magnet_assembly.num_tangential() {
        for mut shape in proto_shapes.iter().cloned() {
            let offset = (tan_idx as f64 - 0.5 * (magnet_assembly.num_tangential() as f64 - 1.0))
                * magnet_coverage;
            shape.translate([offset, 0.0]);
            shapes.push(shape);
        }
    }

    return shapes;
}

#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Creation of the shapes of a surface magnet assembly][cad_create_surface_mag_assembly]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image(
        "cad_create_surface_mag_assembly",
        "docs/img/cad_create_surface_mag_assembly.svg"
    )
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/// coverage_single_magnet: angle
pub fn surface_magnet_assembly_shapes_rot(
    magnet_assembly: &MagnetAssembly,
    split: bool,
    radius: Length,
    is_outer: bool,
    coverage_single_magnet: Option<f64>,
) -> Vec<PositionedMagnetShape> {
    let mut proto_shapes: Vec<PositionedMagnetShape> = if split {
        magnet_assembly
            .magnet()
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
            shape: magnet_assembly.magnet().shape().into_owned(),
            is_north: true,
            magnet_type: 0,
        }]
    };

    if is_outer {
        for s in proto_shapes.iter_mut() {
            s.line_reflection([0.0, 0.0], [1.0, 0.0]);
        }
    }

    let magnet_coverage = coverage_single_magnet.unwrap_or_else(|| {
        let radius = radius.get::<meter>();
        pole_coverage_angle(
            proto_shapes.iter().map(|p| &p.shape),
            radius,
            Length::new::<meter>(0.0),
        )
    });

    let mut shapes = Vec::with_capacity(magnet_assembly.num_tangential() * proto_shapes.len());

    for tan_idx in 0..magnet_assembly.num_tangential() {
        for mut shape in proto_shapes.iter().cloned() {
            let angle = (tan_idx as f64 - 0.5 * (magnet_assembly.num_tangential() as f64 - 1.0))
                * magnet_coverage;
            shape.rotate([0.0, -radius.get::<meter>()], -angle);
            shapes.push(shape);
        }
    }

    return shapes;
}

/// Assumptions: shapes are as required by Magnet trait
/// TODO: Explain that this can deal with negative radii (as might be reported
/// from some magnets such as ArcSegmentMagnet to signal convex / concave)
pub fn pole_coverage_angle<'a, I: Iterator<Item = &'a Shape> + Clone>(
    magnets: I,
    radius: f64,
    gap_between_partial_magnets: Length,
) -> f64 {
    // Based on the shape points, calculate the preliminary coverage angle
    let mut angle = magnets
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
    for s in magnets.map(|shape| shape.contour().segments()).flatten() {
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
