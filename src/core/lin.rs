/*!
This module contains the [`LinCore`] type and its builder struct
[`LinCoreBuilder`]. [`LinCore`] forms the basis for all linear magnetic cores
used in the stem ecosystem. See its docstring for more.
 */

use compare_variables::compare_variables;
use planar_geo::{prelude::BoundingBox, shape::Shape};
use std::sync::Arc;
use stem_magnet::prelude::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use serde_mosaic::{deserialize_arc_link, serialize_arc_link};

use super::CoreExt;
use crate::{
    LinOrRot,
    air_gap::{AirGap, PlainAirGap},
    error::IncompatibleFluxBarrier,
    flux_barrier::FluxBarrier,
};

/**
A magnetic core for a linear electric motor / machine.

Seen from its cross section, a radial flux linear electric motor consists of
effectively two rectangles (stator and rotor) where the rotor slides against the
stator. Therefore, the cross section of the stator / rotor core is effectively
also a rectangle and the extents of it are described by its width, height and
axial length (which in the cross section view goes into the image plane).
Furthermore, the core may have geometric features such as a special air gap
contour or cutouts (flux barriers). The following image shows the cross section
of a slotted core with simple "star" flux barriers. The slots may hold a
winding, the flux barriers may contain magnets (not depicted).
*/
#[doc = ""]
#[cfg_attr(feature = "doc-images", doc = "![Linear core][lin_core]")]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image("lin_core", "docs/img/lin_core.svg")
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

# Building a `LinCore`

A [`LinCore`] is built from a [`LinCoreBuilder`]. If the field values of the
[`LinCoreBuilder`] do not result in a valid core (e.g. if negative dimensions
are given), the conversion fails, as shown in the example below. The field
docstrings of [`LinCoreBuilder`] state the allowed value range for each
parameter. Besides the [`LinCore::new`] constructor, [`TryFrom`] / [`TryInto`]
implementations are also available.

```
use std::sync::Arc;
use stem_core::prelude::*;

// Valid parameters
let air_gap = PlainAirGap::new(Length::new::<meter>(0.0), 0.0, 1, 0, true).expect("valid data");
let builder = LinCoreBuilder {
    height: Length::new::<millimeter>(20.0),
    width: Length::new::<millimeter>(100.0),
    axial_length: Length::new::<millimeter>(100.0),
    axial_coil_overhang: Length::new::<millimeter>(0.0),
    skew_angle: 0.0,
    iron_fill_factor: 1.0,
    material: Arc::new(Material::default()),
    pole_pairs: 2,
    air_gap: Box::new(air_gap),
    flux_barrier: None,
};

let core = LinCore::new(builder).expect("valid inputs");
assert_eq!(core.width().get::<millimeter>(), 100.0);

// Invalid parameters (negative core width).
let air_gap = PlainAirGap::new(Length::new::<meter>(0.0), 0.0, 1, 0, true).expect("valid data");
let builder = LinCoreBuilder {
    height: Length::new::<millimeter>(20.0),
    width: Length::new::<millimeter>(-100.0), // Negative width!
    axial_length: Length::new::<millimeter>(100.0),
    axial_coil_overhang: Length::new::<millimeter>(0.0),
    skew_angle: 0.0,
    iron_fill_factor: 1.0,
    material: Arc::new(Material::default()),
    pole_pairs: 2,
    air_gap: Box::new(air_gap),
    flux_barrier: None,
};

// try_from is equivalent to new
assert!(LinCore::try_from(builder).is_err());
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
height: 20 mm
width: 100 mm
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
        number_segments: 1
        starts_in_slot_middle: true
        slots: 0
"};

let core: LinCore = serde_yaml::from_str(&str).expect("valid dimensions");
assert_eq!(core.width().get::<millimeter>(), 100.0);
```
 */
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "LinCoreBuilder"))]
pub struct LinCore {
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    height: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    width: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    axial_length: Length,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    axial_coil_overhang: Length,
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_angle"))]
    skew_angle: f64,
    iron_fill_factor: f64,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_arc_link",))]
    material: Arc<Material>,
    pole_pairs: u16,
    air_gap: Box<dyn AirGap>,
    flux_barrier: Option<Box<dyn FluxBarrier>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    shape: Shape,
}

impl LinCore {
    /**
    Builds a new [`LinCore`] from a [`LinCoreBuilder`].

    Building a [`LinCore`] can fail if the provided data is invalid (e.g.
    negative dimensions). See the field documentation of [`LinCoreBuilder`] for
    details. In such a case, the resulting error is returned instead.

    This method forwards to the `TryInto<LinCore>` implementation of
    [`LinCoreBuilder`].
     */
    pub fn new(builder: LinCoreBuilder) -> Result<Self, crate::error::Error> {
        builder.try_into()
    }

    /// Returns the height (vertical extents) of the core.
    ///
    /// This value is equivalent to [`LinCoreBuilder::height`] from the builder
    /// struct used to create `self`.
    pub fn height(&self) -> Length {
        return self.height;
    }

    /// Returns the width (horizontal extents) of the core.
    ///
    /// This value is equivalent to [`LinCoreBuilder::width`] from the builder
    /// struct used to create `self`.
    pub fn width(&self) -> Length {
        return self.width;
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
    /// let air_gap = PlainAirGap::new(Length::new::<meter>(0.0), 0.0, 1, 0, true).expect("valid inputs");
    /// let mut lin_core: LinCore = LinCoreBuilder {
    ///     height: Length::new::<millimeter>(20.0),
    ///     width: Length::new::<millimeter>(100.0),
    ///     axial_length: Length::new::<millimeter>(100.0),
    ///     axial_coil_overhang: Length::new::<millimeter>(0.0),
    ///     skew_angle: 0.0,
    ///     iron_fill_factor: 1.0,
    ///     material: Arc::new(Material::default()),
    ///     pole_pairs: 2,
    ///     air_gap: Box::new(air_gap),
    ///     flux_barrier: None, // No flux barrier at initialization
    /// }.try_into().expect("valid inputs");
    /// assert!(lin_core.flux_barrier().is_none());
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
    /// assert!(lin_core.set_flux_barrier(Some(Box::new(fb_comp))).is_ok());
    /// assert!(lin_core.flux_barrier().is_some());
    ///
    /// // Remove the flux barrier
    /// assert!(lin_core.set_flux_barrier(None).is_ok()); // Cannot fail for None input
    /// assert!(lin_core.flux_barrier().is_none());
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
    /// assert!(lin_core.set_flux_barrier(Some(Box::new(fb_incomp))).is_err());
    /// assert!(lin_core.flux_barrier().is_none());
    /// ```
    ///
    /// The image below shows a comparison between the flux barrier contours of
    /// `fb_comp` and `fb_incomp`. It is clear to see that the latter is
    /// incompatible to `lin_core` since the flux barriers intersect the shape
    /// contour as well as each other due to the limited width of `lin_core`.
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Comparison compatible and incompatible flux barrier][lin_core_set_fb]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image("lin_core_set_fb", "docs/img/lin_core_set_fb.svg")
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    pub fn set_flux_barrier(
        &mut self,
        flux_barrier: Option<Box<dyn FluxBarrier>>,
    ) -> Result<(), IncompatibleFluxBarrier> {
        let mut air_gap: Box<dyn AirGap> = Box::new(PlainAirGap::default());
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

impl super::ext::private::Sealed for LinCore {}

impl CoreExt for LinCore {
    fn air_gap(&self) -> &dyn AirGap {
        return &*self.air_gap;
    }

    fn flux_barrier(&self) -> Option<&dyn FluxBarrier> {
        return self.flux_barrier.as_ref().map(|v| &**v);
    }
    fn axial_length(&self) -> Length {
        return self.axial_length;
    }

    fn air_gap_width(&self) -> Length {
        return self.width;
    }

    fn iron_fill_factor(&self) -> f64 {
        return self.iron_fill_factor;
    }

    fn pole_pairs(&self) -> u16 {
        return self.pole_pairs;
    }

    fn material(&self) -> &Arc<Material> {
        return &self.material;
    }

    fn yoke_height(&self) -> Length {
        return self.height - self.air_gap.tooth_height(self.into());
    }

    fn skew_angle(&self) -> f64 {
        return self.skew_angle;
    }

    fn lin_or_rot(&self) -> LinOrRot {
        return LinOrRot::Lin;
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
            let single_magnet_coverage = BoundingBox::from(&*assembly.magnet().shape()).width();
            return 2.0
                * self.pole_pairs() as f64
                * assembly.num_tangential() as f64
                * single_magnet_coverage
                / self.width().get::<meter>();
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
Builder struct for [`LinCore`].

This struct can be (fallibly) converted into a[`LinCore`] via its [`TryFrom`] /
[`TryInto`] implementation or via [`LinCore::new`]. The conversion fails if one
of the field values is not inside the value range given on the individual field
docstrings.

The serialized representation of a [`LinCore`] is equivalent to that of this
struct. When deserializing a [`LinCore`], the serialized representation is first
deserialized into a [`LinCoreBuilder`] which is then converted via [`TryFrom`].

See the docstring of [`LinCore`] for examples.
 */
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct LinCoreBuilder {
    /// Height (vertical extents) of the core. Must be positive (`height >= 0
    /// m`).
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub height: Length,
    /// Width (horizontal extents) of the core. Must be positive (`width >= 0
    /// m`).
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_quantity"))]
    pub width: Length,
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
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_arc_link"))]
    /// Material used for the core.
    pub material: Arc<Material>,
    /// Number of pole pairs of the core.
    pub pole_pairs: u16,
    /// Definition of the air gap shape. See the docstring of [`AirGap`] for
    /// details.
    pub air_gap: Box<dyn AirGap>,
    /// Definition of the flux barrier geometry, if the core has any. See the
    /// docstring of [`FluxBarrier`] for more. Setting this field to `None`
    /// means that the core has no flux barriers. This field can also be set
    /// after the creation of a [`LinCore`] with [`LinCore::set_flux_barrier`].
    /// This field can be omitted when deserializing, in which case it is set to
    /// `None`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub flux_barrier: Option<Box<dyn FluxBarrier>>,
}

impl TryFrom<LinCoreBuilder> for LinCore {
    type Error = crate::error::Error;

    fn try_from(builder: LinCoreBuilder) -> Result<Self, Self::Error> {
        let zero = Length::new::<meter>(0.0);
        compare_variables!(val zero <= builder.height)?;
        compare_variables!(val zero <= builder.width)?;
        compare_variables!(val zero <= builder.axial_length)?;
        compare_variables!(val zero <= builder.axial_coil_overhang)?;
        compare_variables!(0.0 <= builder.iron_fill_factor <= 1.0)?;

        let mut this = LinCore {
            height: builder.height,
            width: builder.width,
            axial_length: builder.axial_length,
            iron_fill_factor: builder.iron_fill_factor,
            material: builder.material,
            pole_pairs: builder.pole_pairs,
            skew_angle: builder.skew_angle,
            axial_coil_overhang: builder.axial_coil_overhang,
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
