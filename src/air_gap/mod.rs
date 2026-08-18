/*!
This module defines the [`AirGap`] trait used to customize the air gap contour
of magnetic cores such as [`LinCore`](crate::core::LinCore) and
[`RotCore`](crate::core::RotCore).

Besides the aforementioned trait, this module also reexports some implementors
of the [`AirGap`] trait as well as auxiliary types and functions:
- [`PlainAirGap`] (reexported from the [`plain`] module) defines a smooth air
gap contour.
- [`SlottedAirGap`] (reexported from the [`slotted`] module) uses [`Slot`] trait
objects to define a grooved ("slotted") air gap. The module also provides the
[`CarterFactorModel`] enum which is used when creating a [`SlottedAirGap`].
- [`StraightIndentsAirGap`] "flattens" the core at its poles to provide mounting
points for magnets with a straight surface such as
[`BlockMagnet`](stem_magnet::block::BlockMagnet)s. These mounting points can be
raised or sunken into the core surface. [`PolygonAirGapBuilder`] is a helper
struct to easily create a polygonal rotary air gap surface from a
[`StraightIndentsAirGap`].

See the [trait documentation](AirGap) for details.
 */

use std::f64::consts::PI;

use dyn_clone::DynClone;
use planar_geo::{polysegment::Polysegment, shape::Shape};
use stem_magnet::assembly::MagnetAssembly;
use stem_slot::prelude::*;

use crate::{
    core::{CoreExt, CoreRef},
    error::Error,
    magnets::Magnets,
    winding_zones::{WindingZones, WindingZonesEqSpaced},
};

pub mod plain;
pub mod slotted;
pub mod straight_indents;

pub use plain::PlainAirGap;
pub use slotted::{CarterFactorModel, SlottedAirGap};
pub use straight_indents::{PolygonAirGapBuilder, StraightIndentsAirGap};

/**
A trait to define the air gap contour of a magnetic core.

This trait is used to create the `air_gap` trait objects for the core
builder structs [`LinCoreBuilder`](crate::core::LinCoreBuilder) and
[`RotCoreBuilder`](crate::core::RotCoreBuilder). The following image shows three
different cores where all parameters except for `air_gap` are identical:
 */
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Different air gaps, same core otherwise][rot_air_gap_comparison]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image(
        "rot_air_gap_comparison",
        "docs/img/rot_air_gap_comparison.svg"
    )
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

_This image was produced with `examples/air_gap_plots.rs`. From left to right:
a [`PlainAirGap`], a [`SlottedAirGap`] and a [`StraightIndentsAirGap`]._

Besides the visual appearance, the air gap defines many more important features
of the core via the [`AirGap`] trait methods: Whether a winding or air gap
surface magnets can be mounted on the core, how large the magnetically effective
air gap width is, whether the core has [`Slot`]s and so on. Please see the
individual trait methods for more information.

The trait methods are not meant to be called by user code. Instead, all of them
(except [`AirGap::combine`]) are used to implement the [`CoreExt`] methods of
the same name. For example, the [`CoreExt::slots`] method is implemented like
this:

```ignore
fn slots(&self) -> u16 {
    return self.air_gap().slots(self.as_core_ref());
}
```

Generally speaking, the documentation of the [`CoreExt`] method therefors
focuses on the _usage_ of that specific method, whereas the [`AirGap`] method
docstring explains how to _implement_ it for custom air gap types. If the latter
have examples, they are just there to show how the method is supposed to work,
not how to use them in user code.

The [`AirGap::combine`] method is used in
[`LinCore::new`](crate::core::LinCore::new) and
[`RotCore::new`](crate::core::RotCore::new) to create the core shape and to
determine whether an air gap is compatible to the core at all. See the trait
method documentation for examples.

This design pattern replaces an earlier one where types like `RotCorePlain`
and `LinCoreSlotted` existed, which was clearly OOP-inspired. This older pattern
had the general issue that each new air gap shape required defining two new
types (one for a linear and one for a rotary core) and that it was difficult to
share code between those. The new pattern follows the "composition over
inheritance" philosophy of Rust, by treating the air gap as a property of the
core.
 */
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait AirGap: DynClone + Sync + Send + std::fmt::Debug + std::any::Any {
    /// Combines `self` with the `core` and returns the resulting cross-section
    /// shape.
    ///
    /// This method is used inside [`LinCore::new`](crate::core::LinCore::new) /
    /// [`RotCore::new`](crate::core::RotCore::new) to generate the core
    /// shape. It therefore functions as a general hook to
    /// check the compatibility of the [`AirGap`] with the core. For example,
    /// when combining an [`SlottedAirGap`] with a
    /// [`LinCore`](crate::core::LinCore), the latter must be high enough to
    /// accomodate the slot. But even if the shape creation succeeds, an
    /// [`AirGap`] might still be incompatible to a core: If for example the
    /// [`air_gap_winding_height`](PlainAirGap::air_gap_winding_height) of a
    /// [`PlainAirGap`] is larger than inner air gap radius of a
    /// [`RotCore`](crate::core::RotCore), the winding does not fit inside
    /// the core. Invariants like these can also be checked within this
    /// method.
    ///
    /// Some implementors of [`AirGap`] might also be generally incompatible to
    /// either a [`LinCore`](crate::core::LinCore) or a
    /// [`RotCore`](crate::core::RotCore). In this case, this method should
    /// return [`Error::IncompatibleToLinCore`] or
    /// [`Error::IncompatibleToRotCore`], where the `&'static str` represents
    /// the type name.
    ///
    /// It might be useful to cache data created during the combination within
    /// `self` (e.g. geometric data which is expensive to calculate but useful
    /// for other [`AirGap`] methods). Therefore, this method takes a
    /// `&mut self` reference so that data can be stored within `self`.
    ///
    /// # Examples
    ///
    /// The following examples shows how [`AirGap::combine`] is used within
    /// [`LinCore::new`](crate::core::LinCore::new) /
    /// [`RotCore::new`](crate::core::RotCore::new) on a principal level.
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// use stem_core::prelude::*;
    ///
    /// fn fake_new(mut air_gap: Box<dyn AirGap>) -> Result<LinCore, stem_core::error::Error> {
    ///     // Create the core with a placeholder air gap (will be replaced
    ///     // after a successfull combine call). In the actual implementation,
    ///     // the LinCore struct is assembled directly from the LinCoreBuilder
    ///     // fields instead of using try_into.
    ///     let core: LinCore = LinCoreBuilder {
    ///         height: Length::new::<millimeter>(20.0),
    ///         width: Length::new::<millimeter>(100.0),
    ///         axial_length: Length::new::<millimeter>(100.0),
    ///         axial_coil_overhang: Length::new::<millimeter>(0.0),
    ///         skew_angle: 0.0,
    ///         iron_fill_factor: 1.0,
    ///         material: Arc::new(Material::default()),
    ///         pole_pairs: 2,
    ///         air_gap: Box::new(PlainAirGap::default()),
    ///         flux_barrier: None,
    ///     }.try_into()?;
    ///
    ///     // In the real new method, the shape is cached within LinCore
    ///     let _shape = air_gap.combine(core.as_core_ref())?;
    ///     
    ///     return Ok(core);
    /// }
    ///
    /// // Air gap which is compatible to the core because its indents have
    /// // a length of 20 mm, there is one of them per pole and the core has
    /// // four poles -> all indents cover 80 mm in total, which is smaller than
    /// // the core width of 100 mm
    /// let comp_ag = StraightIndentsAirGap {
    ///     num_segments: 1.try_into().expect("is not zero"),
    ///     indent_width: Length::new::<millimeter>(20.0),
    ///     indent_depth: Length::new::<millimeter>(2.0),
    ///     indents_per_pole: 1,
    /// };
    ///
    /// assert!(fake_new(Box::new(comp_ag)).is_ok());
    ///
    /// // Air gap is not compatible because it has two indents per pole, but
    /// // the indent length is still 20 mm -> All indents cover 160 mm in total,
    /// // which is larger than the core width of 100 mm.
    /// let incomp_ag = StraightIndentsAirGap {
    ///     num_segments: 1.try_into().expect("is not zero"),
    ///     indent_width: Length::new::<millimeter>(20.0),
    ///     indent_depth: Length::new::<millimeter>(2.0),
    ///     indents_per_pole: 2,
    /// };
    /// assert!(fake_new(Box::new(incomp_ag)).is_err());
    /// ```
    fn combine(&mut self, core: CoreRef<'_>) -> Result<Shape, Error>;

    /// Returns the discretization / number of segments of the core.
    ///
    /// This method implements
    /// [`CoreExt::num_segments`]. Depending on `self`, a core may be composed
    /// of multiple individual segments against each other as defined by the
    /// [`CoreExt::skew_angle`](crate::core::CoreExt::skew_angle). This affects
    /// the [`skew_factor`](crate::core::skew_factor) of the core, which can be
    /// used to suppress/ unwanted magnetic harmonics.  See the
    /// docstrings of [`CoreExt::skew_angle`](crate::core::CoreExt::skew_angle)
    /// and [`skew_factor`](crate::core::skew_factor) for details.
    /// If this value is zero, the core is continuously skewed. If it is one,
    /// the component is not skewed at all, as it consists of a single straight
    /// (non-twisted) segment.
    ///
    /// Some [`AirGap`]s might not be discretizable; an example would be the
    /// [`SlottedAirGap`] type. If the air gap can however be skewed, this
    /// method should return 0. If skewing is also not possible, it should
    /// return 1. In return, some air gaps like [`StraightIndentsAirGap`] might
    /// be discretizable, but not continuously skewable. In the case of
    /// [`StraightIndentsAirGap`], this is ensured by the constructor.
    fn num_segments(&self, core: CoreRef<'_>) -> usize;

    /// Returns the number of slots.
    ///
    /// This method implements [`CoreExt::slots`]. If the number of slots is
    /// zero, slot is not windable. This number corresponds to the "slots"
    /// property of a winding, not necessarily to a physical [`Slot`]. For
    /// example, a plain air gap has no slot, but still can be windable.
    fn slots(&self, core: CoreRef<'_>) -> u16;

    /**
    Returns the slot opening factor for the harmonic with the specified
    `mech_ordinal`.

    This method implements [`CoreExt::slot_opening_factor`]. When determining
    the electric loading / induction distribution along the air gap, analytical
    methods assume that the whole electric loading produced by a particular slot
    is concentrated in its center at the air gap. For real core and winding
    geometries, this is obviously not the case. For the example of a
    [`SlottedAirGap`], the electric loading is distributed along the slot,
    opening whereas a wound [`PlainAirGap`] distributes the load along the
    entire air gap surface covered by coils. For further information, see
    standard electric machines literature like e.g.
    [\[1\]](#air_gap_slot_opening_factor_1), section 1.2.3.3.

    The effect of this distribution on a particular harmonic can be calculated
    with the "slot opening factor" ξ which is defined as:

    `ξ = sin(k) / k`

    with `k = mech_ordinal * slot_opening_width / slot_pitch * PI / slots`
    [\[1\]](#air_gap_slot_opening_factor_1), eq. (1.2.62). The mechanical
    ordinal is related to the electrical ordinal via:

    `mech_ordinal = el_ordinal * pole_pairs`

    Multiplying the absolute of this factor with the corresponding harmonic
    amplitude for the idealized case returns the actual harmonic amplitude.

    [\[1\]](#air_gap_slot_opening_factor_1), eq. (1.2.62) is implemented in
    the free [`slot_opening_factor`] function. It is recommended to use this
    function when implementing an [`AirGap`] unless there is a good reason to
    use a custom formula. See the implementations of [`PlainAirGap`] and
    [`SlottedAirGap`] for examples on how to utilize [`slot_opening_factor`] for
    implementing this method.

    # Literature
    <a id="air_gap_slot_opening_factor_1">\[1\]</a>
    Müller, G., Vogt, K. and Ponick, B.: Berechnung elektrischer Maschinen,
    6th edition, Wiley-VCH, 2008

    # Examples

    ```
    use std::f64::consts::FRAC_PI_2;
    use std::sync::Arc;

    use approxim::assert_abs_diff_eq;

    use stem_core::prelude::*;

    // 80 % of a slot pitch is covered by coils
    let air_gap = PlainAirGap {
        num_segments: 0,
        air_gap_winding_height: Length::new::<millimeter>(1.0),
        winding_coverage: 0.8,
        starts_in_slot_middle: true,
        slots: 36,
    };
    let core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(18.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        skew_angle: 0.0,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    }
    .try_into()
    .unwrap();

    // First electrical / second mechanical harmonic
    assert_abs_diff_eq!(core.air_gap().slot_opening_factor(core.as_core_ref(), 2), 0.996754, epsilon = 1e-6);

    // Superharmonics produced by the winding like the electrical 5th and 7th one.
    assert_abs_diff_eq!(core.air_gap().slot_opening_factor(core.as_core_ref(), 10), 0.920725, epsilon = 1e-6);
    assert_abs_diff_eq!(core.air_gap().slot_opening_factor(core.as_core_ref(), 14), 0.848221, epsilon = 1e-6);
    ```
     */
    fn slot_opening_factor(&self, core: CoreRef<'_>, mech_ordinal: i32) -> f64;

    /// Returns the Carter factor of `self` for the given `core`.
    ///
    /// This method implements
    /// [`CoreExt::carter_factor`] for the different possible air gap types. If
    /// the air gap contour is (approximately) smooth or the air gap cannot
    /// be wound in the first place, this method should simply return 1 for
    /// any input. Otherwise, the returned value can depend on core geometry
    /// and air gap width, see for example [`CarterFactorModel`]. It should
    /// be equal to or larger than 1 to represent the virtual "increase" of
    /// the `air_gap_width` due to the non-smooth surface.
    fn carter_factor(&self, core: CoreRef<'_>, air_gap_width: Length) -> f64;

    /// Returns a reference to the [`Slot`] of the air gap, if it has one.
    ///
    /// This method implements [`CoreExt::slot`].
    fn slot(&self, _core: CoreRef<'_>) -> Option<&dyn Slot>;

    /// Returns an iterator over the surface magnet shapes for the given
    /// `magnet_assembly` and `core`.
    ///
    /// This method implements
    /// [`CoreExt::surface_magnets`] for the different possible air gap types.
    /// For example, for a [`PlainAirGap`] and a rotary core, the magnets
    /// are arranged on the circular air gap surface of the core, whereas
    /// they are positioned in the indent middle for a
    /// [`StraightIndentsAirGap`].
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Surface magnets for a plain and an indent air gap][surface_magnets]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image("surface_magnets", "docs/img/surface_magnets.svg")
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    /// _This image was produced with `examples/surface_magnets.rs`._
    ///
    /// When implementing this method for custom [`AirGap`]s, the following
    /// rules should be followed:
    /// - If no magnets can be placed on the air gap surface, an empty iterator
    /// should be returned (for example
    /// `Magnets::Other(Box::new([].into_iter()))`.
    /// - The iterator should return `n`
    /// [`PositionedMagnetShape`](crate::magnets::PositionedMagnetShape)
    /// elements, where `magnet_assembly.num_tangential() * (1 + split) *
    /// core.poles()`.
    /// - The shapes must not overlap each other or the
    /// [`core.shape()`](CoreExt::Shape). This can be checked using
    /// [`CoreExt::assembly_check`].
    /// - Although not strictly required by [`CoreExt::assembly_check`], the
    /// magnet shapes should not "hover" over the core shape, but instead be
    /// attached to it.
    /// - Since there is only one type of magnet assembly located on the air gap
    /// surface by definition,
    /// [`PositionedMagnetShape::magnet_idx`](crate::magnets::PositionedMagnetShape::magnet_idx)
    /// should always be zero.
    /// - If `split` is true, each magnet should be separated in its north and
    /// south shape using
    /// [`Magnet::north_south_shapes`](stem_magnet::magnet::Magnet::north_south_shapes).
    /// Otherwise, the whole magnet shape should be returned. When returning the
    /// shapes for a negative pole, the shapes need to be adjusted for polarity
    /// (see [`PositionedMagnetShape::is_north`](crate::magnets::PositionedMagnetShape::is_north)).
    ///
    /// The [`crate::magnets`] module contains some predefined iterators
    /// to simplify the implementation of this method, see e.g. the source code
    /// of [`PlainAirGap::surface_magnets`] for an example.
    fn surface_magnets(
        &self,
        magnet_assembly: &MagnetAssembly,
        core: CoreRef<'_>,
        split: bool,
    ) -> Magnets {
        // Dummy implementation, to be overwritten.
        return crate::magnets::EvenlyDistributedMagnets::<true>::from_magnet_assembly(
            0,
            Length::new::<millimeter>(0.0),
            magnet_assembly,
            split,
            true,
            core.d_axis_offset(),
        )
        .into();
    }

    /// Returns an iterator over the
    /// [`PositionedZoneContour`](core::winding_zones::PositionedZoneContour)s
    /// for the given `coil_layout`.
    ///
    /// This method implements [`CoreExt::winding_zones`] for the different
    /// possible air gap types. For example, for a [`PlainAirGap`], the
    /// winding zone contours are located inside the air gap itself, whereas
    /// those of a [`SlottedAirGap`] are situated within the slots as shown
    /// in the image below:
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Winding zones for a slotted and a plain air gap][winding_zones]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image("winding_zones", "docs/img/winding_zones.svg")
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    /// _This image was produced with `examples/winding_zones.rs`._
    ///
    /// When implementing this method for custom [`AirGap`]s, the following
    /// rules should be followed:
    /// - If the core is not windable, an empty iterator should be returned
    /// (which can for example be constructed from
    /// [`WindingZonesEqSpaced::no_slots`](crate::magnets::WindingZonesEqSpaced)).
    /// - The iterator should return `n`
    /// [`PositionedZoneContour`](core::winding_zones::PositionedZoneContour)
    /// elements, where `n = coil_layout.layers() * self.slots()`
    /// - Each element should have a unique [`Zone`](core::winding_zones::Zone)
    /// index. In sum, all returned elements should cover all possible layer
    /// / slot combinations resulting from `coil_layout` and `self.slots()`.
    /// - The zones must not overlap each other or the
    /// [`core.shape()`](CoreExt::Shape). This can be checked using
    /// [`CoreExt::assembly_check`].
    /// - Although not strictly required by [`CoreExt::assembly_check`], the
    /// zone contours should not "hover" over the core shape, but instead be
    /// attached to it.
    ///
    /// The [`crate::winding_zones`] module contains some predefined iterators
    /// to simplify the implementation of this method, see e.g. the source code
    /// of [`PlainAirGap::winding_zones`] for an example.
    fn winding_zones(&self, _core: CoreRef<'_>, _coil_layout: &CoilLayout) -> WindingZones {
        // Dummy implementation, to be overwritten.
        WindingZones::WindingZonesEqSpacedLin(WindingZonesEqSpaced::<
            planar_geo::prelude::Contour,
            true,
        >::no_slots())
        .into()
    }

    /// Returns the tooth height of the core.
    ///
    /// This method implements
    /// [`CoreExt::tooth_height`](crate::core::CoreExt::tooth_height) for the
    /// different possible air gap types. The default implementation returns
    /// [`Slot::height`] for [`AirGap::slot`], if the latter is `Some`, and
    /// a length of zero meter otherwise. If a particular [`AirGap`] implementor
    /// defines the tooth height differently, this method can be overwritten.
    ///
    /// See [`CoreExt::tooth_height`](crate::core::CoreExt::tooth_height) for
    /// examples.
    fn tooth_height(&self, core: CoreRef<'_>) -> Length {
        self.slot(core)
            .map_or(Length::new::<meter>(0.0), |s| s.height())
    }

    /// Returns the tooth width at a specific height, measured from the air gap.
    ///
    /// This method implements
    /// [`CoreExt::tooth_width_at`](crate::core::CoreExt::tooth_width_at) for
    /// the different possible air gap types. If [`AirGap::slot`] is `None`,
    /// the default implementation returns a length of zero meter. If a slot
    /// does in fact exist, its width at the given `height` is calculated
    /// using [`Slot::width_at`] and the resulting value is then used to
    /// determine the tooth width, i.e. the width of the space between two
    /// slots at that particular `height`. If a particular [`AirGap`]
    /// implementor defines the tooth width differently, this method can be
    /// overwritten.
    ///
    /// See [`CoreExt::tooth_width_at`](crate::core::CoreExt::tooth_width_at)
    /// for examples.
    fn tooth_width_at(&self, core: CoreRef<'_>, height: Length) -> Length {
        let slot = match self.slot(core) {
            Some(s) => s,
            None => return Length::new::<meter>(0.0),
        };

        if height < Length::new::<meter>(0.0) {
            return Length::new::<meter>(0.0);
        }

        match core {
            CoreRef::Lin(lin_core) => {
                return lin_core.width() / self.slots(core) as f64 - slot.width_at(height);
            }
            CoreRef::Rot(rot_core) => {
                let width = slot.width_at(height).get::<meter>();
                let origin_height = if rot_core.is_outer() {
                    (rot_core.origin_offset_core_to_slot() + height).get::<meter>()
                } else {
                    (rot_core.origin_offset_core_to_slot() - height).get::<meter>()
                };
                let radius = (origin_height.powi(2) + (0.5 * width).powi(2)).sqrt();
                return Length::new::<meter>(
                    stem_slot::slot::semi_regular_polygon_side_length(
                        width,
                        radius,
                        2 * usize::from(self.slots(core)),
                    )
                    .unwrap(),
                );
            }
        }
    }

    /// Returns a calculator for determining the current displacement
    /// coefficients for different current frequencies.
    ///
    /// This method implements
    /// [`CoreExt::current_displacement_coefficients`].
    /// If an air gap supports windings created from massive conductors
    /// (e.g. squirrel cage windings), the latter may be subject to
    /// non-negligible current displacement affecting both the effective
    /// electrical resistance and inductance. See
    /// [`CurrentDisplacementCalculator`] for a detailed explanation of the
    /// effect and its calculation.
    ///
    /// This method is only used in the special case where the conductor is
    /// (partially) surrounded by the core (currently only the case for a
    /// [`SlottedAirGap`]). Hence, its default implementation creates a
    /// calculator which simply returns [`CurrentDisplacementCoefficients`]
    /// which are 1 for any input (i.e., no current displacement effects take
    /// place). This method should only be overwritten if the [`AirGap`] contour
    /// is expected to cause notable current displacement and [`SlottedAirGap`]
    /// is insufficient for that particular use case.
    fn current_displacement_coefficients(
        &self,
        _core: CoreRef<'_>,
    ) -> CurrentDisplacementCalculator {
        return CurrentDisplacementCalculator::from_slice_dims([].into_iter());
    }
}

dyn_clone::clone_trait_object!(AirGap);

/**
Returns the slot opening factor for the harmonic with the specified
`mech_ordinal`.

When determining the electric loading / induction distribution along the air
gap, analytical methods assume that the whole electric loading produced by
a particular slot is concentrated in its center at the air gap. For real
core and winding geometries, this is obviously not the case. For the example
of a [`SlottedAirGap`], the electric loading is distributed along the slot,
opening whereas a wound [`PlainAirGap`] distributes the load along the
entire air gap surface covered by coils. For further information, see
standard electric machines literature like e.g.
[\[1\]](#air_gap_slot_opening_factor_1), section 1.2.3.3.

The effect of this distribution on a particular harmonic can be calculated
with the "slot opening factor" ξ which is defined as:

`ξ = sin(k) / k`

with `k = mech_ordinal * slot_opening_width / slot_pitch * PI / slots`
[\[1\]](#air_gap_slot_opening_factor_1), eq. (1.2.62). The mechanical
ordinal is related to the electrical ordinal via:

`mech_ordinal = el_ordinal * pole_pairs`

Multiplying the absolute of this factor with the corresponding harmonic
amplitude for the idealized case returns the actual harmonic amplitude.

The mechanical ordinal can be specified as an integer (as one would expect), but
also as a float. This enables calculating the continuous graph of ξ over the
ordinals.

# Literature
<a id="air_gap_slot_opening_factor_1">\[1\]</a>
Müller, G., Vogt, K. and Ponick, B.: Berechnung elektrischer Maschinen,
6th edition, Wiley-VCH, 2008

# Examples

```
use approxim::assert_abs_diff_eq;

use stem_core::air_gap::slot_opening_factor;
use stem_core::prelude::*;

let slot_pitch = Length::new::<millimeter>(10.0);

// Special (theoretical) case of the current load being concentrated in the slot
// middle
assert_abs_diff_eq!(
    slot_opening_factor(slot_pitch, Length::new::<millimeter>(0.0), 36, 1),
    1.0,
    epsilon = 1e-6
);
assert_abs_diff_eq!(
    slot_opening_factor(slot_pitch, Length::new::<millimeter>(0.0), 36, 10),
    1.0,
    epsilon = 1e-6
);

// Special case of the current load being distributed along the entire slot
// pitch
assert_abs_diff_eq!(
    slot_opening_factor(slot_pitch, slot_pitch, 36, 1),
    0.998731,
    epsilon = 1e-6
);
assert_abs_diff_eq!(
    slot_opening_factor(slot_pitch, slot_pitch, 36, 10),
    0.877822,
    epsilon = 1e-6
);

// Slot opening of 2 mm
assert_abs_diff_eq!(
    slot_opening_factor(slot_pitch, Length::new::<millimeter>(2.0), 36, 1),
    0.9999492,
    epsilon = 1e-6
);
assert_abs_diff_eq!(
    slot_opening_factor(slot_pitch, Length::new::<millimeter>(2.0), 36, 10),
    0.9949307,
    epsilon = 1e-6
);
```
 */
pub fn slot_opening_factor<I: Into<f64>>(
    slot_pitch: Length,
    slot_opening_width: Length,
    slots: u16,
    mech_ordinal: I,
) -> f64 {
    let mech_ordinal: f64 = mech_ordinal.into();
    let k = mech_ordinal * (slot_opening_width / slot_pitch).get::<ratio>() * PI / f64::from(slots);

    // Avoid division of 0/0. This is physically correct, see [1], eq. (1.2.63).
    if k == 0.0 {
        return 1.0;
    } else {
        return k.sin() / k;
    }
}

/// Helper function to combine stator and rotor segment_chain into a shape.
fn combine_air_gap_and_yoke_to_shape(
    air_gap: Polysegment,
    yoke: Polysegment,
    is_outer: bool,
) -> Result<Shape, Error> {
    let (outer, inner) = if is_outer {
        (yoke, air_gap)
    } else {
        (air_gap, yoke)
    };
    return Shape::new(vec![outer.into(), inner.into()]).map_err(From::from);
}

fn zero_length() -> Length {
    Length::new::<meter>(0.0)
}

fn deserialize_nonnegative_length<'de, D>(deserializer: D) -> Result<Length, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = deserialize_quantity(deserializer)?;
    let zero_length = zero_length();
    if let Err(err) = compare_variables::compare_variables!(value >= zero_length) {
        return Err(serde::de::Error::custom(err));
    }
    Ok(value)
}
