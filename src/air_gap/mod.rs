use std::f64::consts::PI;

use dyn_clone::DynClone;
use planar_geo::{polysegment::Polysegment, shape::Shape};
use stem_magnet::assembly::MagnetAssembly;
use stem_slot::prelude::*;

use crate::{core::CoreRef, error::Error, magnets::Magnets, winding_zones::WindingZones};

pub mod plain;
pub mod slotted;
pub mod straight_indents;

pub use plain::PlainAirGap;
pub use slotted::{CarterFactorModel, SlottedAirGap};
pub use straight_indents::{AirGapPolygonBuilder, StraightIndentsAirGap};

/**
TODO: Explain that all methods which take a core as second arg are not meant to
be used standalone, but instead are called from the corresponding [`CoreExt`]
methods, using core.air_gap() as first, core as second arg. Hence, these methods
basically implement the [`CoreExt`] methods.

See docstring of CoreExt for in-depth discussion.
 */
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait AirGap: DynClone + Sync + Send + std::fmt::Debug + std::any::Any {
    /// Returns the discretization / number of segments of the core.
    ///
    /// Depending on `self`, a core may be composed of multiple
    /// individual segments  against each other as defined by the
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

    /// Returns an iterator over the surface magnet shapes for the given
    /// `magnet_assembly` and `core`.
    ///
    /// This method implements [`CoreExt::surface_magnets`] for the different
    /// possible air gap types. For example, for a [`PlainAirGap`] and a rotary
    /// core, the magnets are arranged on the circular air gap surface of the
    /// core, whereas they are positioned in the indent middle for a
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
    ///
    /// The [`crate::magnets`] module contains some predefined iterators
    /// to simplify the implementation of this method, see e.g. the source code
    /// of [`PlainAirGap::surface_magnets`] for an example.
    fn surface_magnets(
        &self,
        magnet_assembly: &MagnetAssembly,
        core: CoreRef<'_>,
        split: bool,
    ) -> Magnets;

    /// Returns an iterator over the
    /// [`PositionedZoneContour`](core::winding_zones::PositionedZoneContour)s
    /// for the given `coil_layout`.
    ///
    /// This method implements [`CoreExt::winding_zones`] for the different
    /// possible air gap types. For example, for a [`PlainAirGap`], the winding
    /// zone contours are located inside the air gap itself, whereas those
    /// of a [`SlottedAirGap`] are situated within the slots as shown in the
    /// image below:
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
    fn winding_zones(&self, core: CoreRef<'_>, coil_layout: &CoilLayout) -> WindingZones;

    /// Return the number of slots. If zero, slot is not windable. This number
    /// corresponds to the "slots" property of a winding, not necessarily to a
    /// physical [`Slot`]. For example, a plain air gap has no slot, but still
    /// can be windable.
    fn slots(&self, core: CoreRef<'_>) -> u16;

    /// Combines `self` with the `core` and returns the resulting cross-section
    /// shape.
    ///
    /// This method is used when creating a [`LinCore`] / [`RotCore`] out of a
    /// [`LinCoreBuilder`] / [`RotCoreBuilder`]. It therefore functions as a
    /// general hook to check the compatibility of the [`AirGap`] with the core.
    /// For example, when combining an [`SlottedAirGap`] with a [`LinCore`], the
    /// latter must be high enough to accomodate the slot (otherwise the shape
    /// creation will fail). But even if the shape creation succeeds, an
    /// [`AirGap`] might still be incompatible to a core: If for example the
    /// [`air_gap_winding_height`](PlainAirGap::air_gap_winding_height) of a
    /// [`PlainAirGap`] is larger than inner air gap radius of a [`RotCore`],
    /// the winding does not fit inside the core.
    ///
    /// Some implementors of [`AirGap`] might also be generally incompatible to
    /// either a [`LinCore`] or a [`RotCore`]. In this case, this method should
    /// return [`Error::IncompatibleToLinCore`] or
    /// [`Error::IncompatibleToRotCore`], where the `&'static str` represents
    /// the type name.
    ///
    /// TODO: Example for successfull and failing combination
    /// TODO: Slotted core image (linear core?)
    fn combine(&mut self, core: CoreRef<'_>) -> Result<Shape, Error>;

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

    use approx::assert_abs_diff_eq;

    use stem_core::prelude::*;

    // 80 % of a slot pitch is covered by coils
    let air_gap = PlainAirGap::new(Length::new::<millimeter>(0.0), 0.8, 1, 36, true).unwrap();
    let core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(18.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        skew_angle: 0.0,
        air_gap: Box::new(PlainAirGap::default()),
        flux_barrier: Some(Box::new(fb)),
    }
    .try_into()
    .unwrap();

    // First electrical / second mechanical harmonic
    assert_abs_diff_eq!(core.air_gap().slot_opening_factor(core.as_core_ref(), 2), 0.212, epsilon = 1e-6);

    // Superharmonics produced by the winding like the electrical 5th and 7th one.
    assert_abs_diff_eq!(core.air_gap().slot_opening_factor(core.as_core_ref(), 10), 0.212, epsilon = 1e-6);
    assert_abs_diff_eq!(core.air_gap().slot_opening_factor(core.as_core_ref(), 14), 0.212, epsilon = 1e-6);
    ```
     */
    fn slot_opening_factor(&self, core: CoreRef<'_>, mech_ordinal: i32) -> f64;

    // =========================================================================

    /**
    Return the cross section area of a winding zone. In case of a slotted air gap, this is the windable slot area.
     */
    fn zone_area(&self, _core: CoreRef<'_>) -> Area {
        Area::new::<square_meter>(0.0)
    }

    fn tooth_height(&self, _core: CoreRef<'_>) -> Length {
        return Length::new::<meter>(0.0);
    }

    fn tooth_width_at(&self, _core: CoreRef<'_>, _height: Length) -> Length {
        return Length::new::<meter>(0.0);
    }

    fn carter_factor(&self, _core: CoreRef<'_>, _air_gap_width: Length) -> f64 {
        return 1.0;
    }

    fn slot(&self, _core: CoreRef<'_>) -> Option<&dyn Slot> {
        return None;
    }

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
use approx::assert_abs_diff_eq;

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
