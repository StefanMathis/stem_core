/*!
This module contains the [`RotCore`] type and its builder struct
[`RotCoreBuilder`]. [`RotCore`] forms the basis for all rotary magnetic cores
used in the stem ecosystem. See its docstring for more.
 */

use compare_variables::compare_variables;
use planar_geo::{prelude::BoundingBox, shape::Shape};
use std::{f64::consts::TAU, sync::Arc};
use stem_magnet::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use serde_mosaic::{deserialize_arc_link, serialize_arc_link};

use super::CoreExt;
use crate::{LinOrRot, air_gap::AirGap, error::IncompatibleFluxBarrier, flux_barrier::FluxBarrier};

/**
A magnetic core for a rotary electric motor / machine.

Seen from its cross section, a radial flux rotary electric motor consists of two
coaxial hollow cylinders / tubes, where one of them (the rotor) rotates around
the other (the stator). Therefore, the cross section of the stator / rotor core
is effectively also a hollow cylinder described by the inner radius, outer
radius and axial length (which in the cross section view goes into the image
plane). The space between the two cylinders is called the air gap. The outer
radius of the inner cylinder / core is called the
[`air_gap_radius`](RotCore::air_gap_radius) and its inner radius
is called the [`yoke_radius`](RotCore::yoke_radius). For the outer cylinder /
core, it is the other way around: The outer radius is the yoke radius, the inner
one the air gap radius. Hence, a core is called an _inner_ core, if
`core.air_gap_radius() > core.yoke_radius()`, otherwise it is called an _outer_
core.

Both inner and outer cores may have geometric features such as a special air gap
contour or cutouts (flux barriers). The following image shows an inner core with
a simple "star" flux barrier and 6 poles and an outer core with slots for a
winding (not depicted).
 */
#[doc = ""]
#[cfg_attr(feature = "doc-images", doc = "![Rotary core][rot_core]")]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image("rot_core", "docs/img/rot_core.svg")
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

# Building a `RotCore`

A [`RotCore`] is built from a [`RotCoreBuilder`]. If the field values of the
[`RotCoreBuilder`] do not result in a valid core (e.g. if negative dimensions
are given), the conversion fails, as shown in the example below. The field
docstrings of [`RotCoreBuilder`] state the allowed value range for each
parameter. Besides the [`RotCore::new`] constructor, [`TryFrom`] / [`TryInto`]
implementations are also available.

```
use std::sync::Arc;
use stem_core::prelude::*;

// Valid parameters (resulting in an outer core, since air_gap_radius < yoke_radius)
let air_gap = PlainAirGap::default();
let builder = RotCoreBuilder {
    air_gap_radius: Length::new::<millimeter>(55.0),
    yoke_radius: Length::new::<millimeter>(90.0),
    axial_length: Length::new::<millimeter>(165.0),
    axial_coil_overhang: Length::new::<millimeter>(0.0),
    iron_fill_factor: 1.0,
    material: Arc::new(Material::default()),
    pole_pairs: 2,
    skew_angle: 0.0,
    air_gap: Box::new(air_gap),
    flux_barrier: None,
};

let core = RotCore::new(builder).expect("valid inputs");
assert_eq!(core.air_gap_radius().get::<millimeter>(), 55.0);

// Invalid parameters (negative air_gap_radius).
let air_gap = PlainAirGap::default();
let builder = RotCoreBuilder {
    air_gap_radius: Length::new::<millimeter>(-55.0),
    yoke_radius: Length::new::<millimeter>(90.0),
    axial_length: Length::new::<millimeter>(165.0),
    axial_coil_overhang: Length::new::<millimeter>(0.0),
    iron_fill_factor: 1.0,
    material: Arc::new(Material::default()),
    pole_pairs: 2,
    skew_angle: 0.0,
    air_gap: Box::new(air_gap),
    flux_barrier: None,
};

// try_from is equivalent to new
assert!(RotCore::try_from(builder).is_err());
```

# Serialization and deserialization

The serialized representation of a [`LinCore`] is equivalent to that of
[`LinCoreBuilder`]. When deserializing a [`LinCore`], the serialized
representation is first deserialized into a [`LinCoreBuilder`] which is then
converted via [`TryFrom`].

```
use approx;
use stem_core::prelude::*;
use serde_yaml;

let str = indoc::indoc! {"
air_gap_radius: 55 mm
yoke_radius: 90 mm
axial_length: 100 mm
axial_coil_overhang: 0 mm
skew_angle: 0
iron_fill_factor: 1
material:
    name: lamination
    relative_permeability: 6000
pole_pairs: 2
air_gap:
    PlainAirGap:
        air_gap_winding_height: 0 mm
        winding_coverage: 0
        num_segments: 1
        starts_in_slot_middle: true
        slots: 0
"};

let core: RotCore = serde_yaml::from_str(&str).expect("valid dimensions");
assert_eq!(core.air_gap_radius().get::<millimeter>(), 55.0);
```
 */
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "RotCoreBuilder"))]
pub struct RotCore {
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    air_gap_radius: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    yoke_radius: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    axial_length: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    axial_coil_overhang: Length,
    iron_fill_factor: f64,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_arc_link",))]
    material: Arc<Material>,
    pole_pairs: u16,
    skew_angle: f64,
    air_gap: Box<dyn AirGap>,
    flux_barrier: Option<Box<dyn FluxBarrier>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    shape: Shape,
}

impl RotCore {
    /**
    Builds a new [`RotCore`] from a [`RotCoreBuilder`].

    Building a [`RotCore`] can fail if the provided data is invalid (e.g.
    negative dimensions). See the field documentation of [`RotCoreBuilder`] for
    details. In such a case, the resulting error is returned instead.

    This method forwards to the `TryInto<RotCore>` implementation of
    [`RotCoreBuilder`].
     */
    pub fn new(builder: RotCoreBuilder) -> Result<Self, crate::error::Error> {
        builder.try_into()
    }

    /// Returns the air gap radius of the core.
    ///
    /// If this value is larger than [`RotCore::yoke_radius`], the core is an
    /// inner core, otherwise it is an outer core. This value is equivalent to
    /// [`RotCoreBuilder::air_gap_radius`] from the builder struct used to
    /// create `self`.
    pub fn air_gap_radius(&self) -> Length {
        return self.air_gap_radius;
    }

    /// Returns the yoke radius of the core.
    ///
    /// If this value is larger than [`RotCore::air_gap_radius`], the core is an
    /// outer core, otherwise it is an inner core. This value is equivalent to
    /// [`RotCoreBuilder::yoke_radius`] from the builder struct used to create
    /// `self`.
    pub fn yoke_radius(&self) -> Length {
        return self.yoke_radius;
    }

    /**
    Returns whether `self` is an inner or outer core.

    This method is implemented as `self.air_gap_radius() < self.yoke_radius()`.
    See the docstring of [`RotCore`] for an explanation of the concept.
     */
    pub fn is_outer(&self) -> bool {
        self.air_gap_radius() < self.yoke_radius()
    }

    /// Returns the radius at the yoke middle.
    ///
    /// If [`RotCore::is_outer`] is true, this is [`RotCore::yoke_radius`] minus
    /// half the [`CoreExt::yoke_height`]. Otherwise, it is the yoke radius plus
    /// half the yoke height.
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Yoke middle radius][cad_yoke_middle_radius]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "cad_yoke_middle_radius",
            "docs/img/cad_yoke_middle_radius.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use stem_core::prelude::*;
    /// use stem_slot::semi_trapezoid::SemiTrapezoidWithoutSlopesBuilder;
    ///
    /// let air_gap = PlainAirGap::default();
    /// let plain_core: RotCore = RotCoreBuilder {
    ///     air_gap_radius: Length::new::<millimeter>(40.0),
    ///     yoke_radius: Length::new::<millimeter>(20.0),
    ///     axial_length: Length::new::<millimeter>(100.0),
    ///     axial_coil_overhang: Length::new::<millimeter>(0.0),
    ///     skew_angle: 0.0,
    ///     iron_fill_factor: 1.0,
    ///     material: Arc::new(Material::default()),
    ///     pole_pairs: 3,
    ///     air_gap: Box::new(air_gap),
    ///     flux_barrier: None,
    /// }.try_into().expect("valid inputs");
    ///
    /// assert_eq!(plain_core.yoke_middle_radius().get::<millimeter>(), 30.0);
    ///
    /// let slot: SemiTrapezoidSlot = SemiTrapezoidWithoutSlopesBuilder {
    ///     bottom_width: Length::new::<millimeter>(4.0),
    ///     opening_width: Length::new::<millimeter>(2.0),
    ///     height: Length::new::<millimeter>(8.0),
    ///     opening_height: Length::new::<millimeter>(1.0),
    ///     slot_angle: 0.0,
    ///     bottom_radius: Length::new::<millimeter>(0.0),
    ///     top_radius: Length::new::<millimeter>(0.0),
    ///     opening_radius: Length::new::<millimeter>(0.0),
    ///     consider_tooth_tip_leakage: true,
    /// }
    /// .try_into()
    /// .unwrap();
    /// let air_gap = SlottedAirGap::new(
    ///     12,
    ///     true,
    ///     CarterFactorModel::Bin12,
    ///     Box::new(slot),
    /// );
    /// let slotted_core: RotCore = RotCoreBuilder {
    ///     air_gap_radius: Length::new::<millimeter>(40.0),
    ///     yoke_radius: Length::new::<millimeter>(20.0),
    ///     axial_length: Length::new::<millimeter>(100.0),
    ///     axial_coil_overhang: Length::new::<millimeter>(0.0),
    ///     skew_angle: 0.0,
    ///     iron_fill_factor: 1.0,
    ///     material: Arc::new(Material::default()),
    ///     pole_pairs: 3,
    ///     air_gap: Box::new(air_gap),
    ///     flux_barrier: None,
    /// }.try_into().expect("valid inputs");
    ///
    /// assert_eq!(slotted_core.yoke_middle_radius().get::<millimeter>(), 25.0);
    /// ```
    pub fn yoke_middle_radius(&self) -> Length {
        let outer = self.is_outer();
        let sign = outer as i32 as f64 - (!outer) as i32 as f64;
        return self.yoke_radius() - 0.5 * sign * self.yoke_height();
    }

    /// Returns the offset between the coordinate system origin of the core and
    /// that of its slots, if it has any.
    ///
    /// This method returns the `offset` between the coordinate system of a
    /// [`Slot`] and that of `self`:
    ///
    /// `r_core = y_slot + offset`.
    ///
    /// If the slot is closed, `offset` is simply the air gap radius. If the
    /// slot is open, the slot coordinate system is shifted inwards by `delta`
    /// as shown in the image below. This is done because the actual slot
    /// opening height would be larger than [`Slot::opening_height`] because a
    /// rotary core "bends away" from the slot baseline, necessitating the
    /// shift by `delta` to compensate. For an open slot, `offset` is therefore:
    ///
    /// `offset = sqrt(RotCore::air_gap_radius² - (Slot::opening_width/2)²)`
    ///
    /// If the core has no slots, this method returns `0 m`.
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Offset between the core and the slot coordinate system origin][cad_rot_core_slot_cs_offset]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "cad_rot_core_slot_cs_offset",
            "docs/img/cad_rot_core_slot_cs_offset.svg"
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
    /// use std::sync::Arc;
    /// use approx::assert_abs_diff_eq;
    /// use stem_core::prelude::*;
    /// use stem_slot::semi_trapezoid::SemiTrapezoidWithoutSlopesBuilder;
    ///
    /// let air_gap = PlainAirGap::default();
    /// let plain_core: RotCore = RotCoreBuilder {
    ///     air_gap_radius: Length::new::<millimeter>(40.0),
    ///     yoke_radius: Length::new::<millimeter>(20.0),
    ///     axial_length: Length::new::<millimeter>(100.0),
    ///     axial_coil_overhang: Length::new::<millimeter>(0.0),
    ///     skew_angle: 0.0,
    ///     iron_fill_factor: 1.0,
    ///     material: Arc::new(Material::default()),
    ///     pole_pairs: 3,
    ///     air_gap: Box::new(air_gap),
    ///     flux_barrier: None,
    /// }.try_into().expect("valid inputs");
    /// assert_eq!(plain_core.origin_offset_core_to_slot().get::<millimeter>(), 0.0);
    ///
    /// let slot: SemiTrapezoidSlot = SemiTrapezoidWithoutSlopesBuilder {
    ///     bottom_width: Length::new::<millimeter>(4.0),
    ///     opening_width: Length::new::<millimeter>(2.0),
    ///     height: Length::new::<millimeter>(8.0),
    ///     opening_height: Length::new::<millimeter>(1.0),
    ///     slot_angle: 0.0,
    ///     bottom_radius: Length::new::<millimeter>(0.0),
    ///     top_radius: Length::new::<millimeter>(0.0),
    ///     opening_radius: Length::new::<millimeter>(0.0),
    ///     consider_tooth_tip_leakage: true,
    /// }
    /// .try_into()
    /// .unwrap();
    /// let air_gap = SlottedAirGap::new(
    ///     12,
    ///     true,
    ///     CarterFactorModel::Bin12,
    ///     Box::new(slot),
    /// );
    /// let slotted_core: RotCore = RotCoreBuilder {
    ///     air_gap_radius: Length::new::<millimeter>(40.0),
    ///     yoke_radius: Length::new::<millimeter>(20.0),
    ///     axial_length: Length::new::<millimeter>(100.0),
    ///     axial_coil_overhang: Length::new::<millimeter>(0.0),
    ///     skew_angle: 0.0,
    ///     iron_fill_factor: 1.0,
    ///     material: Arc::new(Material::default()),
    ///     pole_pairs: 3,
    ///     air_gap: Box::new(air_gap),
    ///     flux_barrier: None,
    /// }.try_into().expect("valid inputs");
    ///
    /// assert_abs_diff_eq!(slotted_core.origin_offset_core_to_slot().get::<millimeter>(), 39.987, epsilon = 1e-3);
    /// ```
    pub fn origin_offset_core_to_slot(&self) -> Length {
        use uom::typenum::P2;
        if let Some(slot) = self.slot() {
            let r = self.air_gap_radius();
            let s = slot.opening_width();
            return (r.powi(P2::new()) - (0.5 * s).powi(P2::new())).sqrt();
        } else {
            return Length::new::<meter>(0.0);
        }
    }

    /// Fallibly inserts a new [`FluxBarrier`] into `self` or removes an
    /// existing one.
    ///
    /// If `flux_barrier` is `Some`, it is checked whether the wrapped
    /// [`FluxBarrier`] is compatible to `self` by creating the flux barrier
    /// contours via the [`FluxBarrier::combine`] method and checking if those
    /// fit into the shape of `self`. If [`FluxBarrier::combine`] fails or if
    /// the contours don't fit, the resulting error is wrapped into
    /// [`IncompatibleFluxBarrier`] and returned together with the given
    /// [`FluxBarrier`]. Otherwise, the old flux barrier of `self` will be
    /// replaced with the new one. If `flux_barrier` is `None` and `self` has a
    /// flux barrier, it will be removed. Otherwise, this is a no-op.
    ///
    /// # Examples
    ///
    /// The following code shows how adding a compatible flux barrier succeeds,
    /// how it can be removed again and how adding an incompatible flux barrier
    /// fails.
    ///
    /// ```
    /// use std::sync::Arc;
    /// use stem_core::prelude::*;
    ///
    /// let air_gap = PlainAirGap::default();
    /// let mut rot_core: RotCore = RotCoreBuilder {
    ///     air_gap_radius: Length::new::<millimeter>(40.0),
    ///     yoke_radius: Length::new::<millimeter>(15.0),
    ///     axial_length: Length::new::<millimeter>(100.0),
    ///     axial_coil_overhang: Length::new::<millimeter>(0.0),
    ///     skew_angle: 0.0,
    ///     iron_fill_factor: 1.0,
    ///     material: Arc::new(Material::default()),
    ///     pole_pairs: 3,
    ///     air_gap: Box::new(air_gap),
    ///     flux_barrier: None, // No flux barrier at initialization
    /// }.try_into().expect("valid inputs");
    /// assert!(rot_core.flux_barrier().is_none());
    ///
    /// // A compatible flux barrier
    /// let fb_comp = Star1FluxBarrier {
    ///     air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
    ///     yoke_leakage_path_width: Length::new::<millimeter>(1.0),
    ///     relief_path_air_gap_width: Length::new::<millimeter>(4.0),
    ///     magnet_space_width: Length::new::<millimeter>(10.0),
    ///    magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<millimeter>(2.0)),
    ///     glue_gap: Length::new::<millimeter>(0.0),
    ///     magnet_material: None,
    ///     cache: None,
    /// };
    /// assert!(rot_core.set_flux_barrier(Some(Box::new(fb_comp))).is_ok());
    /// assert!(rot_core.flux_barrier().is_some());
    ///
    /// // Remove the flux barrier
    /// assert!(rot_core.set_flux_barrier(None).is_ok()); // Cannot fail for None input
    /// assert!(rot_core.flux_barrier().is_none());
    ///
    /// // An incompatible flux barrier
    /// let mut fb_incomp = Star1FluxBarrier {
    ///     air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
    ///     yoke_leakage_path_width: Length::new::<millimeter>(1.0),
    ///     relief_path_air_gap_width: Length::new::<millimeter>(4.0),
    ///     magnet_space_width: Length::new::<millimeter>(30.0), // Too wide for the core width
    ///     magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<
    ///         millimeter,
    ///     >(2.0)),
    ///     glue_gap: Length::new::<millimeter>(0.0),
    ///     magnet_material: None,
    ///     cache: None,
    /// };
    /// assert!(rot_core.set_flux_barrier(Some(Box::new(fb_incomp))).is_err());
    /// assert!(rot_core.flux_barrier().is_none());
    /// ```
    ///
    /// The image below shows a comparison between the flux barrier contours of
    /// `fb_comp` and `fb_incomp`. It is clear to see that the latter is
    /// incompatible to `rot_core` since the flux barriers intersect the shape
    /// contour as well as each other due to the limited width of `rot_core`.
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
    pub fn set_flux_barrier(
        &mut self,
        flux_barrier: Option<Box<dyn crate::prelude::FluxBarrier>>,
    ) -> Result<(), IncompatibleFluxBarrier> {
        let mut air_gap: Box<dyn AirGap> = Box::new(crate::air_gap::PlainAirGap::default());
        std::mem::swap(&mut air_gap, &mut self.air_gap);
        let mut shape = air_gap.combine(self.as_core_ref()).expect(
            "air gap - core combination produced a valid shape during construction of self. This is a bug.",
        );
        std::mem::swap(&mut air_gap, &mut self.air_gap);

        if let Some(mut fb) = flux_barrier {
            let contours = match fb.combine(self.as_core_ref()) {
                Ok(c) => c,
                Err(cause) => {
                    return Err(IncompatibleFluxBarrier {
                        flux_barrier: fb,
                        cause,
                    });
                }
            };

            for hole in contours {
                if let Err(cause) = shape.add_hole(hole) {
                    return Err(IncompatibleFluxBarrier {
                        flux_barrier: fb,
                        cause: cause.into(),
                    });
                }
            }
            self.flux_barrier = Some(fb);
        } else {
            self.flux_barrier = None;
        }

        self.shape = shape;
        return Ok(());
    }
}

impl super::ext::private::Sealed for RotCore {}

impl CoreExt for RotCore {
    fn air_gap(&self) -> &dyn AirGap {
        return &*self.air_gap;
    }

    fn flux_barrier(&self) -> Option<&dyn FluxBarrier> {
        return self.flux_barrier.as_ref().map(|v| &**v);
    }

    fn air_gap_width(&self) -> Length {
        return self.air_gap_radius() * TAU;
    }

    fn axial_length(&self) -> Length {
        return self.axial_length;
    }

    fn yoke_height(&self) -> Length {
        return (self.air_gap_radius() - self.yoke_radius()).abs() - self.tooth_height();
    }

    fn iron_fill_factor(&self) -> f64 {
        return self.iron_fill_factor;
    }

    fn material(&self) -> &Arc<Material> {
        return &self.material;
    }

    fn pole_pairs(&self) -> u16 {
        return self.pole_pairs;
    }

    fn num_segments(&self) -> usize {
        return self.air_gap.num_segments(self.into());
    }

    fn skew_angle(&self) -> f64 {
        return self.skew_angle;
    }

    fn lin_or_rot(&self) -> LinOrRot {
        return LinOrRot::Rot;
    }

    fn axial_coil_overhang(&self) -> Length {
        return self.axial_coil_overhang;
    }

    fn shape<'a>(&'a self) -> &'a Shape {
        return &self.shape;
    }

    fn as_core_ref(&self) -> super::CoreRef<'_> {
        return self.into();
    }

    fn pole_coverage(&self, surface_magnet_assembly: Option<&MagnetAssembly>) -> f64 {
        if let Some(assembly) = surface_magnet_assembly {
            let single_magnet_coverage = crate::magnets::pole_coverage_angle(
                std::iter::once(&*assembly.magnet().shape()),
                self.air_gap_radius.get::<meter>(),
                Length::new::<meter>(0.0),
            );
            return 2.0
                * self.pole_pairs() as f64
                * assembly.num_tangential() as f64
                * single_magnet_coverage
                / TAU;
        } else {
            if let Some(flux_barrier) = &self.flux_barrier {
                return flux_barrier.pole_coverage(self.as_core_ref());
            } else {
                return 0.5;
            }
        }
    }
}

/**
Builder struct for [`RotCore`].

This struct can be (fallibly) converted into a[`RotCore`] via its [`TryFrom`] /
[`TryInto`] implementation or via [`RotCore::new`]. The conversion fails if one
of the field values is not inside the value range given on the individual field
docstrings.

The serialized representation of a [`RotCore`] is equivalent to that of this
struct. When deserializing a [`RotCore`], the serialized representation is first
deserialized into a [`RotCoreBuilder`] which is then converted via [`TryFrom`].

See the docstring of [`RotCore`] for examples.
 */
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct RotCoreBuilder {
    /// Air gap radius of the core. Must be positive and not equal to
    /// [`RotCoreBuilder::yoke_radius`] (`0 m <= air_gap_radius !=
    /// yoke_radius`).
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub air_gap_radius: Length,
    /// Yoke radius of the core. Must be positive and not equal to
    /// [`RotCoreBuilder::air_gap_radius`] (`0 m <= yoke_radius !=
    /// air_gap_radius`).
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub yoke_radius: Length,
    /// Axial length of the core. This dimension is invisible when using the
    /// typical cross-section view of a core because it goes into the image
    /// plane. Must be positive (`axial_length >= 0 m`).
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub axial_length: Length,
    /// If the core holds a winding, this specifies the axial overhang of both
    /// sides. See [`CoreExt::axial_coil_overhang`] for details. Must be
    /// positive (`axial_coil_overhang >= 0 m`).
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub axial_coil_overhang: Length,
    /// Skew angle of the core. See [`CoreExt::skew_angle`] for details.
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_angle"))]
    pub skew_angle: f64,
    /// Magnetic cores are often build from stacked sheets of ferromagnetic
    /// lamination, which are connected by glue. The gap between the sheets
    /// reduces the effective magnetic conductivity, see
    /// [`CoreExt::iron_length`]. This effect can be modeled by setting this
    /// factor somewhere between 0 and 1 (`0 <= iron_fill_factor <= 1`). Typical
    /// values are usually between 0.9 and 1.
    pub iron_fill_factor: f64,
    /// Material used for the core.
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_arc_link"))]
    pub material: Arc<Material>,
    /// Number of pole pairs of the core.
    pub pole_pairs: u16,
    /// Definition of the air gap shape. See the docstring of [`AirGap`] for
    /// details.
    pub air_gap: Box<dyn AirGap>,
    /// Definition of the flux barrier geometry, if the core has any. See the
    /// docstring of [`FluxBarrier`] for more. Setting this field to `None`
    /// means that the core has no flux barriers. This field can also be set
    /// after the creation of a [`RotCore`] with [`RotCore::set_flux_barrier`].
    /// This field can be omitted when deserializing, in which case it is set to
    /// `None`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub flux_barrier: Option<Box<dyn FluxBarrier>>,
}

impl TryFrom<RotCoreBuilder> for RotCore {
    type Error = crate::error::Error;

    fn try_from(builder: RotCoreBuilder) -> Result<Self, Self::Error> {
        let zero = Length::new::<meter>(0.0);
        compare_variables!(val zero <= builder.air_gap_radius)?;
        compare_variables!(val zero <= builder.yoke_radius)?;
        compare_variables!(builder.air_gap_radius != builder.yoke_radius)?;
        compare_variables!(val zero <= builder.axial_length)?;
        compare_variables!(val zero <= builder.axial_coil_overhang)?;
        compare_variables!(0.0 <= builder.iron_fill_factor <= 1.0)?;

        let mut this = RotCore {
            air_gap_radius: builder.air_gap_radius,
            yoke_radius: builder.yoke_radius,
            axial_length: builder.axial_length,
            iron_fill_factor: builder.iron_fill_factor,
            material: builder.material,
            pole_pairs: builder.pole_pairs,
            skew_angle: builder.skew_angle,
            axial_coil_overhang: builder.axial_coil_overhang,
            // Placeholder
            air_gap: builder.air_gap.clone(),
            // Placeholder
            flux_barrier: None,
            // Placeholder
            shape: Shape::from_outer(BoundingBox::new(0.0, 1.0, 0.0, 1.0).into())?,
        };

        let mut ag = builder.air_gap;
        this.shape = AirGap::combine(&mut *ag, this.as_core_ref())?;
        this.air_gap = ag;

        // Check if core and flux barrier are compatible
        if let Some(mut fb) = builder.flux_barrier {
            let contours = FluxBarrier::combine(&mut *fb, this.as_core_ref())?;
            for contour in contours {
                this.shape.add_hole(contour)?;
            }
            this.flux_barrier = Some(fb);
        }

        return Ok(this);
    }
}
