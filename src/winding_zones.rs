/*!
A module providing iterators for positioning winding zones inside magnetic
cores.

A winding [`Zone`] is the space where one coil of a winding is located. A "slot"
is a group of zones layered next to each other as defined by a [`CoilLayout`].
If a core is [slotted](crate::air_gap::SlottedAirGap), this group is located
inside an actual [`Slot`], whereas e.g. for an
[air gap winding](crate::air_gap::PlainAirGap), the coils are located on top of
the air gap surface of the core:

 */
#![cfg_attr(feature = "doc-images", doc = "")]
#![cfg_attr(
    feature = "doc-images",
    doc = "![Winding zone positioning (plain air gap and slotted air gap)][winding_zones_with_zone_pos.svg]"
)]
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("winding_zones_with_zone_pos.svg", "docs/img/winding_zones_with_zone_pos.svg"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/*!
_This image was produced with `examples/winding_zone_plots.rs`._

The [`Contour`]s of the winding zones are positioned relative to the core via
the [`WindingZones`] iterator, which in turn is created by
[`CoreExt::winding_zones`](crate::core::CoreExt::winding_zones). The positioning
depends on the [`AirGap`](crate::air_gap::AirGap) of the core, see
[`AirGap::winding_zones`](crate::air_gap::AirGap::winding_zones).
[`WindingZones`] returns an [`PositionedZoneContour`] struct, which contains the
positioned contour as well as the [`Zone`] indices shown in the image above.
[`PositionedZoneContour`] therefore provides the geometrical constraints for a
winding: For example, it tells how much space is available for the lower layer
in slot 23 of a double-layer winding.

[`WindingZones`] itself is an enum wrapping a bunch of predefined iterators
(such as [`WindingZonesEqSpaced`] which themselves implement
`Iterator<Item=PositionedZoneContour`>). It is possible to define custom
iterators for custom [`AirGap`](crate::air_gap::AirGap)s via the
[`WindingZones::Other`] escape hatch, which allows using any iterator which
returns [`PositionedZoneContour`]s.

`examples/winding_zone_plots.rs` demonstrates how to utilize the
[`WindingZones`] iterator for creating the above image. The following snippet in
particular shows how to to draw the [`PositionedZoneContour`]s.

```ignore
// "cr" is a cairo::Context
for w in core.winding_zones(&CoilLayout::DoubleVertical) {
    w.as_drawable().draw(cr)?;
    let slot_text = Text {
        text: format!("Slot: {}", w.zone.slot),
        anchor: Anchor::Center,
        fixed_anchor_offset: [0.0, -7.0],
        scaled_anchor_offset: w.contour.centroid(),
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        font_size: 12.0,
        angle: 0.0,
    };
    slot_text.draw(cr)?;

    let layer_text = Text {
        text: format!("Layer: {}", w.zone.layer),
        anchor: Anchor::Center,
        fixed_anchor_offset: [0.0, 7.0],
        scaled_anchor_offset: w.contour.centroid(),
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
        font_size: 12.0,
        angle: 0.0,
    };
    layer_text.draw(cr)?;
}
```
 */

use crate::planar_geo;
use planar_geo::DEFAULT_EPSILON;
use planar_geo::prelude::{Polysegment, ToBoundingBox};
use planar_geo::segment::ArcSegment;
use planar_geo::{Transformation, contour::Contour};
use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, TAU};
use stem_slot::prelude::*;
use stem_slot::{coil_layout::CoilLayout, slot::Slot};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "cairo")]
use stem_slot::planar_geo::draw::{Drawable, DrawableRef};

/// An index for a coil / "zone" of a winding.
///
/// The coils of a winding are placed inside "slots". Depending on the
/// [`CoilLayout`] of a winding, multiple coils may share one slot and occupy
/// different "layers" within that slot. This struct is an index to a particular
/// winding zone defined by [`slot`](Zone::slot) and [`layer`](Zone::layer)
/// index which can contain a coil. The following image shows the winding zones
/// for an air gap winding and a winding mounted on a slotted core:
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
///
/// _This image was produced with `examples/winding_zone_plots.rs`._
///
/// [`Zone`] implements [`Ord`]: A zone is said to be greater than another one
/// if its [`slot`](Zone::slot) index is larger. If the [`slot`](Zone::slot)
/// indices are equal, the zone with the larger [`layer`](Zone::layer) index is
/// greater.
///
/// # Examples
///
/// ```
/// use stem_core::winding_zones::Zone;
///
/// let zone_a = Zone {slot: 0, layer: 0};
/// let zone_b = Zone {slot: 1, layer: 0};
/// let zone_c = Zone {slot: 0, layer: 1};
/// assert!(zone_a < zone_b);
/// assert!(zone_a < zone_c);
/// assert!(zone_c < zone_b);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Zone {
    /// Slot index of the zone.
    pub slot: u16,
    /// Layer index of the zone.
    pub layer: u16,
}

impl Zone {
    /// Returns a new [`Zone`].
    pub fn new(slot: u16, layer: u16) -> Self {
        return Self { slot, layer };
    }
}

impl From<Zone> for [u16; 2] {
    fn from(zone: Zone) -> Self {
        return [zone.slot, zone.layer];
    }
}

impl From<(u16, u16)> for Zone {
    fn from(value: (u16, u16)) -> Self {
        return Zone::new(value.0, value.1);
    }
}

impl From<[u16; 2]> for Zone {
    fn from(value: [u16; 2]) -> Self {
        return Zone::new(value[0], value[1]);
    }
}

impl PartialOrd for Zone {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(self.cmp(other));
    }
}

impl Ord for Zone {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.slot.cmp(&other.slot) {
            core::cmp::Ordering::Equal => return self.layer.cmp(&other.layer),
            ord => return ord,
        }
    }
}

impl std::fmt::Display for Zone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "zone: slot {}, layer {} ", self.slot, self.layer)
    }
}

/// [`Contour`] and [`Zone`] index of a winding zone.
///
/// This struct is returned by the [`WindingZones`] iterator and contains the
/// contour of the zone positioned relative to the magnetic core which created
/// [`WindingZones`] via
/// [`CoreExt::winding_zones`](crate::core::CoreExt::winding_zones). In
/// addition, it also provides the [`Zone`] index.
///
/// This struct implements the [`Transformation`] trait. The trait methods are
/// applied to [`PositionedZoneContour::contour`] using the implementation
/// of that trait for [`Contour`].
///
/// See the [module documentation](crate::winding_zones) for examples.
#[derive(Debug, Clone)]
pub struct PositionedZoneContour {
    /// Positioned contour of the zone.
    pub contour: Contour,
    /// Index of the zone.
    pub zone: Zone,
}

impl PositionedZoneContour {
    /// Converts [`PositionedZoneContour::contour`] into a [`Drawable`] using
    /// the default [`stem_slot::SLOT_STYLE`].
    #[cfg(feature = "cairo")]
    pub fn into_drawable(self) -> Drawable {
        Drawable::new(self.contour, stem_slot::SLOT_STYLE)
    }

    /// Converts [`PositionedZoneContour::contour`] into a [`Drawable`] using
    /// the provided `style`.
    #[cfg(feature = "cairo")]
    pub fn into_drawable_with_style(self, style: planar_geo::draw::Style) -> Drawable {
        Drawable::new(self.contour, style)
    }

    /// Wraps [`PositionedZoneContour::contour`] into a [`DrawableRef`] using
    /// the default [`stem_slot::SLOT_STYLE`].
    #[cfg(feature = "cairo")]
    pub fn as_drawable<'a>(&'a self) -> DrawableRef<'a> {
        return DrawableRef::new(&self.contour, stem_slot::SLOT_STYLE);
    }

    /// Wraps [`PositionedZoneContour::contour`] into a [`DrawableRef`] using
    /// the provided `style`.
    #[cfg(feature = "cairo")]
    pub fn as_drawable_with_style<'a>(&'a self, style: planar_geo::draw::Style) -> DrawableRef<'a> {
        return DrawableRef::new(&self.contour, style);
    }
}

impl From<PositionedZoneContour> for Contour {
    fn from(value: PositionedZoneContour) -> Self {
        value.contour
    }
}

impl From<(Contour, Zone)> for PositionedZoneContour {
    fn from(value: (Contour, Zone)) -> Self {
        Self {
            contour: value.0,
            zone: value.1,
        }
    }
}

#[cfg(feature = "cairo")]
impl From<PositionedZoneContour> for Drawable {
    fn from(value: PositionedZoneContour) -> Self {
        value.into_drawable()
    }
}

impl Transformation for PositionedZoneContour {
    fn translate(&mut self, shift: [f64; 2]) {
        self.contour.translate(shift);
    }

    fn rotate(&mut self, center: [f64; 2], angle: f64) {
        self.contour.rotate(center, angle);
    }

    fn scale(&mut self, factor: f64) {
        self.contour.scale(factor);
    }

    fn line_reflection(&mut self, start: [f64; 2], stop: [f64; 2]) -> () {
        self.contour.line_reflection(start, stop);
    }
}

impl ToBoundingBox for PositionedZoneContour {
    fn bounding_box(&self) -> planar_geo::prelude::BoundingBox {
        self.contour.bounding_box()
    }
}

/**
An iterator over the [`PositionedZoneContour`]s of a magnetic core where the
individual slots are distributed over the air gap surface.

This struct is essentially a builder for a [`WindingZones`] iterator which
groups the zone contours of the same slot together and ensures an equidistant
distribution of the slots over the air gap surface. It can be used for both
linear (`LIN = true`) and rotary cores (`LIN = false`) and provides a bunch of
constructors for either defining it from scratch or for particular
[`AirGap`](crate::air_gap::AirGap) types.

The following image demonstrates how the iterator works using the example of a
linear core with three slots, an [air gap winding](crate::air_gap::PlainAirGap)
on the left side and a [slotted air gap](crate::air_gap::SlottedAirGap) on the
right side. The [`WindingZonesEqSpaced::new`] constructor receives a "prototype"
`zones` argument consisting of the zone contours (shown together with their
coordinate system in red). These zones are then copied into new coordinate
systems which are distributed evenly along the air gap surface, starting from
the left. In case the core begins and ends in the middle of a slot, the layers
are distributed accordingly as shown for the right-side core.
*/
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![How WindingZonesEqSpaced distributes the winding zones][cad_winding_zones_eq_spaced]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image(
        "cad_winding_zones_eq_spaced",
        "docs/img/cad_winding_zones_eq_spaced.svg"
    )
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

# Examples

```
use stem_core::prelude::*;
use planar_geo::prelude::*;

// Dummy contours defining the lower and the upper layer of a winding.
let lower = Contour::new(Polysegment::new());
let upper = Contour::new(Polysegment::new());

let mut wz = WindingZonesEqSpaced::<Contour, true>::new(
    Length::new::<millimeter>(100.0),
    vec![lower, upper],
    12,
    false
);

// Two layers and twelve slots -> Total number of elements should be 24.
assert_eq!(wz.slots() * wz.layers(), 24);
assert_eq!(wz.count(), 24);
```
 */
#[derive(Debug, Clone)]
pub struct WindingZonesEqSpaced<T: Transformation + Clone, const LIN: bool> {
    slots: u16,
    air_gap_length: Length,
    zones: Vec<T>,
    starts_in_slot_middle: bool,
    index: usize,
}

impl<T: Transformation + Clone, const LIN: bool> WindingZonesEqSpaced<T, LIN> {
    /// Creates a new [`WindingZonesEqSpaced`].
    ///
    /// This is the "raw" constructor for a [`WindingZonesEqSpaced`] which can
    /// be used if the constructors with an higher abstraction level such as
    /// [`WindingZonesEqSpaced::from_slot`] and
    /// [`WindingZonesEqSpaced::from_air_gap_winding`] are not applicable. These
    /// constructors usually create `zones` and then call
    /// [`WindingZonesEqSpaced::new`]. See the
    /// [struct-level](WindingZonesEqSpaced) documentation for details.
    ///
    /// - `air_gap_length`: The cross-section length of the air gap surface
    ///   along which the zones will be distributed. See
    ///   [`CoreExt::air_gap_length`](crate::core::CoreExt::air_gap_length).
    /// - `zones`: The geometric entities to be distributed. Each element is
    ///   interpreted as a layer, i.e. `zones.len() == self.layers()`.
    /// - `slots`: Number of slots along the air gap. The `zones` will be
    ///   distributed `slots` times along the `air_gap_length`.
    /// - `starts_in_slot_middle`: If true, the first slot starts at the
    ///   reference point of the core (which is the left side for a
    ///   [`LinCore`](crate::core::LinCore) and the x-axis for a
    ///   [`RotCore`](crate::core::RotCore). If false, it starts at an offset of
    ///   `air_gap_length / (2*slots)`.
    ///
    /// In the following example, `slots` is 3 for both the left side showing
    /// an [air gap winding](crate::air_gap::PlainAirGap) and for the right side
    /// representing a [slotted air gap](crate::air_gap::SlottedAirGap). The
    /// `zones` argument is a vector of two contours as shown in the image. The
    /// `air_gap_length` is identical for both and `starts_in_slot_middle` is
    /// false for the left and true for the right side.
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![How WindingZonesEqSpaced distributes the winding zones][cad_winding_zones_eq_spaced]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "cad_winding_zones_eq_spaced",
            "docs/img/cad_winding_zones_eq_spaced.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    ///
    /// # Examples
    /// ```
    /// use stem_core::prelude::*;
    /// use planar_geo::prelude::*;
    ///
    /// // Dummy contours defining the lower and the upper layer of a winding.
    /// let lower = Contour::new(Polysegment::new());
    /// let upper = Contour::new(Polysegment::new());
    ///
    /// // Left side
    /// let mut left = WindingZonesEqSpaced::<Contour, true>::new(
    ///     Length::new::<millimeter>(100.0),
    ///     vec![lower.clone(), upper.clone()],
    ///     3,
    ///     false
    /// );
    ///
    /// // Two layers and three slots -> Total number of elements should be 6.
    /// assert_eq!(left.slots() * left.layers(), 6);
    /// assert_eq!(left.count(), 6);
    ///
    /// // Right side
    /// let mut right = WindingZonesEqSpaced::<Contour, true>::new(
    ///     Length::new::<millimeter>(100.0),
    ///     vec![lower, upper],
    ///     3,
    ///     true
    /// );
    ///
    /// // Two layers and three slots -> Total number of elements should be 6.
    /// assert_eq!(right.slots() * right.layers(), 6);
    /// assert_eq!(right.count(), 6);
    /// ```
    pub fn new(
        air_gap_length: Length,
        zones: Vec<T>,
        slots: u16,
        starts_in_slot_middle: bool,
    ) -> Self {
        return Self {
            slots,
            air_gap_length,
            zones,
            starts_in_slot_middle,
            index: 0,
        };
    }

    /// Returns a "dummy" iterator which will only yield `None`.
    ///
    /// This constructor is useful for creating an "empty" iterator (e.g. for
    /// implementing
    /// [`CoreExt::winding_zones`](crate::core::CoreExt::winding_zones) for a
    /// non-windable air gap). It calls [`WindingZonesEqSpaced::new`] with
    /// `slots` being zero and `zones` being an empty vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use stem_core::prelude::*;
    /// use planar_geo::prelude::*;
    ///
    /// let mut wz = WindingZonesEqSpaced::<Contour, true>::no_slots();
    /// assert!(wz.next().is_none());
    /// ```
    pub fn no_slots() -> Self {
        return Self {
            slots: 0,
            air_gap_length: Length::new::<meter>(0.0),
            zones: Vec::new(),
            starts_in_slot_middle: true,
            index: 0,
        };
    }

    /// Returns the number of layers within a single slot.
    ///
    /// When iterated over, `self` will return `self.layers() * self.slots()`
    /// items.
    pub fn layers(&self) -> usize {
        return self.zones.len();
    }

    /// Returns the number of slots.
    ///
    /// When iterated over, `self` will return `self.layers() * self.slots()`
    /// items.
    pub fn slots(&self) -> usize {
        return self.slots.into();
    }

    /// Resets the iterator.
    ///
    /// After "resetting" the iterator, the next yielded item is again the first
    /// zone (`Zone {slot: 0, layer: 0}`);
    ///
    /// # Examples
    ///
    /// ```
    /// use stem_core::prelude::*;
    /// use planar_geo::prelude::*;
    ///
    /// // Dummy contours defining the lower and the upper layer of a winding.
    /// let lower = Contour::new(Polysegment::new());
    /// let upper = Contour::new(Polysegment::new());
    ///
    /// let mut wz = WindingZonesEqSpaced::<Contour, true>::new(
    ///     Length::new::<millimeter>(100.0),
    ///     vec![lower, upper],
    ///     12,
    ///     false
    /// );
    ///
    /// // Start iterating
    /// assert_eq!(wz.next().unwrap().zone, Zone {slot: 0, layer: 0});
    /// assert_eq!(wz.next().unwrap().zone, Zone {slot: 0, layer: 1});
    /// assert_eq!(wz.next().unwrap().zone, Zone {slot: 1, layer: 0});
    /// // ...
    ///
    /// // Now reset the iterator
    /// wz.reset();
    /// assert_eq!(wz.next().unwrap().zone, Zone {slot: 0, layer: 0});
    /// ```
    pub fn reset(&mut self) {
        self.index = 0;
    }
}

impl<T: Transformation + Clone + ToBoundingBox, const LIN: bool> WindingZonesEqSpaced<T, LIN> {
    fn next_priv(&mut self) -> Option<(T, Zone)> {
        let layers = self.layers();

        // Check for iterator exhaustion (also covers the case of the contour
        // vector being empty)
        if self.index >= (self.slots as usize) * layers {
            return None;
        }

        let current_layer = self.index.rem_euclid(layers);
        let current_slot = (self.index / layers) as f64;
        self.index = self.index + 1;

        // Cannot panic, because the index is the remainder of a division by the
        // total number of layers and therefore is always in bounds
        let mut geom = self.zones[current_layer].clone();

        if LIN {
            // If all vertices of the contour are negative, shift it to the
            // end of the core. This can only happen for the first slot in
            // case the slot starts in the tooth middle
            let factor = if current_slot == 0.0
                && self.starts_in_slot_middle
                && geom.bounding_box().xmax() <= DEFAULT_EPSILON
            {
                1.0
            } else {
                1.0 / f64::from(self.slots)
                    * (current_slot + 0.5 * (!self.starts_in_slot_middle) as u32 as f64)
            };
            geom.translate([self.air_gap_length.get::<meter>() * factor, 0.0]);
        } else {
            geom.translate([0.0, self.air_gap_length.get::<meter>() / TAU]);
            let angle = -TAU
                * (current_slot + 0.5 * (!self.starts_in_slot_middle) as u32 as f64) as f64
                / self.slots as f64
                + FRAC_PI_2;
            geom.rotate([0.0, 0.0], angle);
        }
        return Some((
            geom,
            Zone {
                slot: current_slot as u16,
                layer: current_layer as u16,
            },
        ));
    }
}

impl<const LIN: bool> WindingZonesEqSpaced<Polysegment, LIN> {
    /// Returns an iterator over the [`Slot::outline`]s distributed evenly along
    /// the `air_gap_length`.
    ///
    /// This constructor is specifically used for creating the
    /// [`SlottedAirGap`](crate::air_gap::SlottedAirGap) shape. It uses the
    /// `slot` to get the [`Slot::outline`], flips it vertically for an inner
    /// rotary core and then forwards the resulting [`Polysegment`] into the
    /// `zones` argument of [`WindingZonesEqSpaced::new`] along with the
    /// other arguments. See [`WindingZonesEqSpaced::new`] for more details
    /// and examples.
    pub fn from_slot<S: Slot + ?Sized>(
        air_gap_length: Length,
        slots: u16,
        slot: &S,
        starts_in_slot_middle: bool,
        outer_core: bool,
    ) -> Self {
        let air_gap_length = if LIN {
            air_gap_length
        } else {
            use stem_material::uom::typenum::P2;
            let air_gap_radius = air_gap_length / TAU;

            let mod_radius = 0.5
                * (4.0 * air_gap_radius.powi(P2::new()) - slot.opening_width().powi(P2::new()))
                    .sqrt();
            mod_radius * TAU
        };

        let mut slot_outline: Polysegment = slot.outline().into_owned();

        if !LIN && !outer_core {
            slot_outline.line_reflection([0.0, 0.0], [1.0, 0.0])
        };

        return Self {
            slots,
            air_gap_length,
            zones: vec![slot_outline],
            starts_in_slot_middle,
            index: 0,
        };
    }
}

impl<const LIN: bool> WindingZonesEqSpaced<Contour, LIN> {
    /// Returns an iterator over the [`Slot::layer_contours`]s distributed
    /// evenly along the `air_gap_length`.
    ///
    /// This constructor creates the `zones` argument of
    /// [`WindingZonesEqSpaced::new`] with `slot.layer_contours(coil_layout)`,
    /// flips it vertically for an inner rotary core and and then forwards the
    /// resulting [`Vec<Contour>`] into the `zones` argument of
    /// [`WindingZonesEqSpaced::new`] along with the other arguments. See
    /// [`WindingZonesEqSpaced::new`] for more details and examples.
    pub fn from_slot<S: Slot + ?Sized>(
        air_gap_length: Length,
        slots: u16,
        slot: &S,
        coil_layout: &CoilLayout,
        starts_in_slot_middle: bool,
        outer_core: bool,
    ) -> Self {
        let mut zones = slot.layer_contours(coil_layout);

        let air_gap_length = if LIN {
            air_gap_length
        } else {
            use stem_material::uom::typenum::P2;
            let air_gap_radius = air_gap_length / TAU;

            if slot.is_open() && coil_layout == &CoilLayout::SingleFilled {
                // Won't panic, since the zones vector will always have
                // one entry if coil_layout == &CoilLayout::Single
                let mut contour = Contour::new(Polysegment::new());
                std::mem::swap(&mut contour, &mut zones[0]);

                // Last element of the contour is the straight slot opening
                // segment => replace it with an arc
                let mut slot_seg = Polysegment::from(contour);
                if let Some(opening_seg) = slot_seg.pop_back() {
                    if let Ok(a) = ArcSegment::from_start_stop_radius(
                        opening_seg.start(),
                        opening_seg.stop(),
                        air_gap_radius.get::<meter>(),
                        outer_core,
                        false,
                    ) {
                        slot_seg.push_back(a.into());
                    } else {
                        slot_seg.push_back(opening_seg.into());
                    }
                }
                let mut contour = Contour::new(slot_seg);
                std::mem::swap(&mut contour, &mut zones[0]);
            }

            let mod_radius = 0.5
                * (4.0 * air_gap_radius.powi(P2::new()) - slot.opening_width().powi(P2::new()))
                    .sqrt();
            mod_radius * TAU
        };
        if !LIN && !outer_core {
            zones
                .iter_mut()
                .for_each(|c| c.line_reflection([0.0, 0.0], [1.0, 0.0]));
        };

        return Self {
            slots,
            air_gap_length,
            zones,
            starts_in_slot_middle,
            index: 0,
        };
    }

    /// Returns an iterator over the [`PositionedZoneContour`]s of an air gap
    /// winding distributed evenly along the `air_gap_length`.
    ///
    /// This method creates the air gap winding zone [`Contour`]s from the
    /// `air_gap_winding_height`, `winding_coverage` and `coil_layout` as shown
    /// in the image below (for a rotary core, the zones are curved instead):
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Air gap winding dimensions][cad_plain_air_gap_winding]"
    )]
    #[cfg_attr(
        feature = "doc-images",
        embed_doc_image::embed_doc_image(
            "cad_plain_air_gap_winding",
            "docs/img/cad_plain_air_gap_winding.svg"
        )
    )]
    #[cfg_attr(
        not(feature = "doc-images"),
        doc = "**Doc images not enabled**. Compile docs with
        `cargo doc --features 'doc-images'` and Rust version >= 1.54."
    )]
    ///
    /// The origin of the created segments is at the air gap surface. If
    /// `outer_core` is true, they are flipped.
    ///
    /// The resulting [`Vec<Contour>`] is then provided as `zones` argument to
    /// [`WindingZonesEqSpaced::new`] along with the other arguments. See
    /// [`WindingZonesEqSpaced::new`] for more details and examples.
    ///
    /// # Examples
    ///
    /// ```
    /// use stem_core::prelude::*;
    /// use planar_geo::prelude::*;
    ///
    /// let coil_layout = CoilLayout::DoubleVertical;
    /// let slots = 12;
    ///
    /// let wz = WindingZonesEqSpaced::<Contour, true>::from_air_gap_winding(
    ///     Length::new::<millimeter>(100.0),
    ///     slots,
    ///     Length::new::<millimeter>(6.0),
    ///     0.8,
    ///     &coil_layout,
    ///     false,
    ///     true,
    /// );
    ///
    /// // Two layers and three slots -> Total number of elements should be 24.
    /// let num_contours = wz.count();
    /// assert_eq!(num_contours, usize::from(slots * coil_layout.layers()));
    /// assert_eq!(num_contours, 24);
    /// ```
    pub fn from_air_gap_winding(
        air_gap_length: Length,
        slots: u16,
        air_gap_winding_height: Length,
        winding_coverage: f64,
        coil_layout: &CoilLayout,
        starts_in_slot_middle: bool,
        outer_core: bool,
    ) -> Self {
        let mut zones = Vec::with_capacity(coil_layout.layers() as usize);
        let ag_height = if outer_core {
            -air_gap_winding_height.get::<meter>()
        } else {
            air_gap_winding_height.get::<meter>()
        };

        if LIN {
            let half_slot_width =
                winding_coverage.clamp(0.0, 1.0) * 0.5 * air_gap_length.get::<meter>()
                    / slots as f64;
            match coil_layout {
                CoilLayout::Single | CoilLayout::SingleFilled => zones.push(Contour::rectangle(
                    [-half_slot_width, 0.0],
                    [half_slot_width, ag_height],
                )),
                CoilLayout::DoubleVertical => {
                    zones.push(Contour::rectangle(
                        [-half_slot_width, 0.0],
                        [half_slot_width, 0.5 * ag_height],
                    ));
                    zones.push(Contour::rectangle(
                        [-half_slot_width, 0.5 * ag_height],
                        [half_slot_width, ag_height],
                    ));
                }
                CoilLayout::DoubleHorizontal => {
                    zones.push(Contour::rectangle(
                        [-half_slot_width, 0.0],
                        [0.0, ag_height],
                    ));
                    zones.push(Contour::rectangle([0.0, 0.0], [half_slot_width, ag_height]));
                }
                CoilLayout::Quadruple => {
                    zones.push(Contour::rectangle(
                        [-half_slot_width, 0.0],
                        [0.0, 0.5 * ag_height],
                    ));
                    zones.push(Contour::rectangle(
                        [-half_slot_width, 0.5 * ag_height],
                        [0.0, ag_height],
                    ));
                    zones.push(Contour::rectangle(
                        [0.0, 0.5 * ag_height],
                        [half_slot_width, ag_height],
                    ));
                    zones.push(Contour::rectangle(
                        [0.0, 0.0],
                        [half_slot_width, 0.5 * ag_height],
                    ));
                }
                CoilLayout::MultiVertical(layers) => {
                    let layer_height = ag_height / (*layers) as f64;
                    for layer in 0..(*layers) {
                        zones.push(Contour::rectangle(
                            [-half_slot_width, layer as f64 * layer_height],
                            [half_slot_width, (layer + 1) as f64 * layer_height],
                        ));
                    }
                }
            }
        } else {
            let angle = winding_coverage.clamp(0.0, 1.0) * TAU / slots as f64;
            let agr = air_gap_length.get::<meter>() / TAU;
            match coil_layout {
                CoilLayout::Single | CoilLayout::SingleFilled => {
                    let mut ps = Polysegment::with_capacity(4);
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        agr,
                        -0.5 * angle + FRAC_PI_2,
                        angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        agr + ag_height,
                        0.5 * angle + FRAC_PI_2,
                        -angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    zones.push(ps.into());
                }
                CoilLayout::DoubleVertical => {
                    let radicand = (agr + ag_height).powi(2) + agr.powi(2);
                    let mean_radius = radicand.signum() * FRAC_1_SQRT_2 * radicand.sqrt();

                    // Lower layer
                    let mut ps = Polysegment::with_capacity(4);
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        agr,
                        -0.5 * angle + FRAC_PI_2,
                        angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        mean_radius,
                        0.5 * angle + FRAC_PI_2,
                        -angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    zones.push(ps.into());

                    // Upper layer
                    let mut ps = Polysegment::with_capacity(4);
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        mean_radius,
                        -0.5 * angle + FRAC_PI_2,
                        angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        agr + ag_height,
                        0.5 * angle + FRAC_PI_2,
                        -angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    zones.push(ps.into());
                }
                CoilLayout::DoubleHorizontal => {
                    // Right layer
                    let mut ps = Polysegment::with_capacity(4);
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        agr,
                        FRAC_PI_2,
                        0.5 * angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        agr + ag_height,
                        FRAC_PI_2 + 0.5 * angle,
                        -0.5 * angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    zones.push(ps.into());

                    // Left layer
                    let mut ps = Polysegment::with_capacity(4);
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        agr,
                        -0.5 * angle + FRAC_PI_2,
                        0.5 * angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        agr + ag_height,
                        FRAC_PI_2,
                        -0.5 * angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    zones.push(ps.into());
                }
                CoilLayout::Quadruple => {
                    let radicand = (agr + ag_height).powi(2) + agr.powi(2);
                    let mean_radius = radicand.signum() * FRAC_1_SQRT_2 * radicand.sqrt();

                    // First layer
                    let mut ps = Polysegment::with_capacity(4);
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        agr,
                        FRAC_PI_2,
                        0.5 * angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        mean_radius,
                        0.5 * angle + FRAC_PI_2,
                        -0.5 * angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    zones.push(ps.into());

                    // Second layer
                    let mut ps = Polysegment::with_capacity(4);
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        mean_radius,
                        FRAC_PI_2,
                        0.5 * angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        agr + ag_height,
                        0.5 * angle + FRAC_PI_2,
                        -0.5 * angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    zones.push(ps.into());

                    // Third layer
                    let mut ps = Polysegment::with_capacity(4);
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        mean_radius,
                        -0.5 * angle + FRAC_PI_2,
                        0.5 * angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        agr + ag_height,
                        FRAC_PI_2,
                        -0.5 * angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    zones.push(ps.into());

                    // Fourth layer
                    let mut ps = Polysegment::with_capacity(4);
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        agr,
                        -0.5 * angle + FRAC_PI_2,
                        0.5 * angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                        [0.0, 0.0],
                        mean_radius,
                        FRAC_PI_2,
                        -0.5 * angle,
                    ) {
                        ps.push_back(arc.into());
                    }
                    zones.push(ps.into());
                }
                CoilLayout::MultiVertical(layers) => {
                    let layer_height = ag_height / (*layers) as f64;
                    for layer in 0..(*layers) {
                        let first_radius = agr + layer as f64 * layer_height;
                        let second_radius = agr + (layer + 1) as f64 * layer_height;

                        let mut ps = Polysegment::with_capacity(4);
                        if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                            [0.0, 0.0],
                            first_radius,
                            -0.5 * angle + FRAC_PI_2,
                            angle,
                        ) {
                            ps.push_back(arc.into());
                        }
                        if let Ok(arc) = ArcSegment::from_center_radius_start_sweep_angle(
                            [0.0, 0.0],
                            second_radius,
                            0.5 * angle + FRAC_PI_2,
                            -angle,
                        ) {
                            ps.push_back(arc.into());
                        }
                        zones.push(ps.into());
                    }
                }
            }
            zones.iter_mut().for_each(|c| c.translate([0.0, -agr]));
        }

        return Self {
            slots,
            air_gap_length,
            zones,
            starts_in_slot_middle,
            index: 0,
        };
    }
}

impl<const LIN: bool> Iterator for WindingZonesEqSpaced<Polysegment, LIN> {
    type Item = Polysegment;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_priv().map(|t| t.0)
    }
}

impl<const LIN: bool> Iterator for WindingZonesEqSpaced<Contour, LIN> {
    type Item = PositionedZoneContour;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_priv().map(From::from)
    }
}

/**
An iterator over the [`PositionedZoneContour`]s of a magnetic core.

If a magnetic core provides space for the coils of a winding ("zones"), this
iterator returns the [`Contour`]s describing that space for each [`Zone`] of the
winding. The [`Zone`]s are strictly monotonic increasing:

```
use std::sync::Arc;
use stem_core::prelude::*;

let core: LinCore = LinCoreBuilder {
    height: Length::new::<millimeter>(15.0),
    width: Length::new::<millimeter>(50.0),
    axial_length: Length::new::<millimeter>(1.0),
    axial_coil_overhang: Length::new::<millimeter>(0.0),
    skew_angle: 0.0,
    iron_fill_factor: 1.0,
    material: Arc::new(Default::default()),
    pole_pairs: 1,
    air_gap: Box::new(PlainAirGap {
        num_segments: 0,
        air_gap_winding_height: Length::new::<millimeter>(8.0),
        winding_coverage: 0.7,
        starts_in_slot_middle: false,
        slots: 3,
    }),
    flux_barrier: None,
}
.try_into().expect("valid dimensions");

// Single-layer winding
let mut wz_single = core.winding_zones(&CoilLayout::Single);
assert_eq!(wz_single.next().unwrap().zone, Zone {slot: 0, layer: 0});
assert_eq!(wz_single.next().unwrap().zone, Zone {slot: 1, layer: 0});
assert_eq!(wz_single.next().unwrap().zone, Zone {slot: 2, layer: 0});
assert!(wz_single.next().is_none());

// Double-layer winding
let mut wz_double = core.winding_zones(&CoilLayout::DoubleVertical);
assert_eq!(wz_double.next().unwrap().zone, Zone {slot: 0, layer: 0});
assert_eq!(wz_double.next().unwrap().zone, Zone {slot: 0, layer: 1});
assert_eq!(wz_double.next().unwrap().zone, Zone {slot: 1, layer: 0});
assert_eq!(wz_double.next().unwrap().zone, Zone {slot: 1, layer: 1});
assert_eq!(wz_double.next().unwrap().zone, Zone {slot: 2, layer: 0});
assert_eq!(wz_double.next().unwrap().zone, Zone {slot: 2, layer: 1});
assert!(wz_double.next().is_none());
```

As shown in the example above, this iterator is not meant to be constructed
directly, but should instead be created with the
[`CoreExt::winding_zones`](crate::core::CoreExt::winding_zones) method. All
predefined specialized iterators (such as [`WindingZonesEqSpaced`]) can be
converted into a [`WindingZones`] via its [`From`] implementation.

When implementing
[`AirGap::winding_zones`](crate::air_gap::AirGap::winding_zones) (which drives
[`CoreExt::winding_zones`](crate::core::CoreExt::winding_zones)), the
[`WindingZones::from_iter`] method can be used for wrapping a custom
iterator. The custom iterator should provide the zones in the same ascending
order as shown in the example (i.e. the nth + 1 [`PositionedZoneContour::zone`]
should be greater than the nth [`PositionedZoneContour::zone`] according to the
[`Ord`] implementation for [`Zone`]). See [`Zone`] for details.
 */
pub struct WindingZones(WindingZonesInner);

impl WindingZones {
    /// Creates a `WindingZones` from a custom iterator.
    ///
    /// The iterator must yield [`PositionedZoneContour`]s in strictly
    /// increasing [`Zone`] order. See the [`Zone`] documentation for
    /// details.
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: Iterator<Item = PositionedZoneContour> + 'static,
    {
        Self(WindingZonesInner::Other(Box::new(iter)))
    }
}

enum WindingZonesInner {
    /// A wrapper variant around [`WindingZonesEqSpaced`] for a linear core.
    WindingZonesEqSpacedLin(WindingZonesEqSpaced<Contour, true>),
    /// A wrapper variant around [`WindingZonesEqSpaced`] for a rotary core.
    WindingZonesEqSpacedRot(WindingZonesEqSpaced<Contour, false>),
    Other(Box<dyn Iterator<Item = PositionedZoneContour>>),
}

impl From<WindingZonesInner> for WindingZones {
    fn from(value: WindingZonesInner) -> Self {
        Self(value)
    }
}

impl Iterator for WindingZones {
    type Item = PositionedZoneContour;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            WindingZonesInner::WindingZonesEqSpacedLin(i) => i.next(),
            WindingZonesInner::WindingZonesEqSpacedRot(i) => i.next(),
            WindingZonesInner::Other(i) => i.next(),
        }
    }
}

impl From<WindingZonesEqSpaced<Contour, true>> for WindingZones {
    fn from(value: WindingZonesEqSpaced<Contour, true>) -> Self {
        WindingZonesInner::WindingZonesEqSpacedLin(value).into()
    }
}

impl From<WindingZonesEqSpaced<Contour, false>> for WindingZones {
    fn from(value: WindingZonesEqSpaced<Contour, false>) -> Self {
        WindingZonesInner::WindingZonesEqSpacedRot(value).into()
    }
}

impl From<Box<dyn Iterator<Item = PositionedZoneContour>>> for WindingZones {
    fn from(value: Box<dyn Iterator<Item = PositionedZoneContour>>) -> Self {
        WindingZonesInner::Other(value).into()
    }
}
