/*!
This module provides the [`PlainAirGap`] struct which provides a smooth air gap
contour as shown in the image below.
 */
#![cfg_attr(feature = "doc-images", doc = "")]
#![cfg_attr(
    feature = "doc-images",
    doc = "![Plain linear and rotary core][lin_and_rot_core_plain.svg]"
)]
#![cfg_attr(feature = "doc-images",
cfg_attr(all(),
doc = ::embed_doc_image::embed_image!("lin_and_rot_core_plain.svg", "docs/img/lin_and_rot_core_plain.svg"),
))]
#![cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/*!
This struct implements the [`AirGap`] trait and can therefore be used to build
magnetic cores. See the struct docstring for more.
*/

use std::f64::consts::PI;

use crate::planar_geo;
use compare_variables::compare_variables;
use stem_magnet::assembly::MagnetAssembly;
use stem_slot::{coil_layout::CoilLayout, prelude::stem_material::prelude::*};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use planar_geo::prelude::*;

use crate::{
    air_gap::AirGap,
    core::{CoreExt, CoreRef},
    error::Error,
    magnets::{EvenlyDistributedMagnets, Magnets},
    winding_zones::{WindingZones, WindingZonesEqSpaced},
};

fn deserialize_winding_coverage<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = f64::deserialize(deserializer)?;
    Ok(value.clamp(0.0, 1.0))
}

/**
A plain air gap without any features.

This is a very simple air gap contour which is basically just a straight line
when applied to a [`LinCore`](crate::core::LinCore) or a circle when applied to
a [`RotCore`](crate::core::RotCore).
 */
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Linear and rotary core with a plain air gap][lin_and_rot_core_plain]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image(
        "lin_and_rot_core_plain",
        "docs/img/lin_and_rot_core_plain.svg"
    )
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**

_This image was produced with `examples/air_gap_plots.rs`._

Despite this simplicity, the plain air gap allows for mounting either an air gap
winding or surface magnets (but not both at the same time). In the image below,
the left core has a single-layer air gap winding, whereas the right core is
equipped with surface magnets on its six poles.
 */
#[doc = ""]
#[cfg_attr(
    feature = "doc-images",
    doc = "![Plain air gap with surface winding and magnets][magnets_and_winding_plain]"
)]
#[cfg_attr(
    feature = "doc-images",
    embed_doc_image::embed_doc_image(
        "magnets_and_winding_plain",
        "docs/img/magnets_and_winding_plain.svg"
    )
)]
#[cfg_attr(
    not(feature = "doc-images"),
    doc = "**Doc images not enabled**. Compile docs with
    `cargo doc --features 'doc-images'` and Rust version >= 1.54."
)]
/**
_This image was produced with `examples/air_gap_plots.rs`._

The space occupied by the air gap winding is defined by the
[`PlainAirGap::air_gap_winding_height`] and [`PlainAirGap::winding_coverage`],
see the method docstrings for details.

A [`PlainAirGap`] can be either segmented or continuously skewed, meaning that
[`AirGap::num_segments`] can be any `usize` for it. For details on the concept
of skewing / segmentation, see also [`CoreExt::num_segments`].

# Fields

All fields of this struct can be read out via their accessor function. Please
see the accessor docstring for details on the particular field.
- [`air_gap_winding_height`](PlainAirGap::air_gap_winding_height)
- [`winding_coverage`](PlainAirGap::winding_coverage)
- [`num_segments`](PlainAirGap::num_segments)
- [`starts_in_slot_middle`](PlainAirGap::starts_in_slot_middle)
- [`slots`](PlainAirGap::slots)

# Constructors

This struct offers three constructors:
- [`PlainAirGap::new`] allows specifying all fields directly (performing value
range checks on some).
- [`PlainAirGap::with_num_segments`] is useful if the core is not meant to be
wound, in which case all arguments to [`PlainAirGap::new`] except `num_segments`
are irrelevant anyway.
- [`PlainAirGap::default`] offers the most basic constructor implemented via the
[`Default`] trait. All values are set to sensible defaults:

```ignore
impl Default for PlainAirGap {
    fn default() -> Self {
        Self {
            num_segments: 0,
            air_gap_winding_height: Length::new::<meter>(0.0),
            winding_coverage: 0.0,
            slots: 0,
            starts_in_slot_middle: true,
        }
    }
}
```

# Serialization and deserialization

A [`PlainAirGap`] can be deserialized directly from the arguments to
[`PlainAirGap::new`]. Note that the same invariants apply:
`air_gap_winding_height` must not be smaller than zero and `winding_coverage`
will be clamped to be between 0 and 1.

```
use stem_core::prelude::*;
use yaml_serde;

let str = indoc::indoc! {"
air_gap_winding_height: 10 mm
winding_coverage: 1.5 # Will be clamped to 0 ... 1
num_segments: 2
starts_in_slot_middle: true
slots: 12
"};

let ag: PlainAirGap = yaml_serde::from_str(&str).expect("valid dimensions");
assert_eq!(ag.winding_coverage(), 1.0);
```

Any of these fields can be omitted, in which case the value from the
[PlainAirGap::default] implementation takes its place.

```
use stem_core::prelude::*;
use yaml_serde;

let str = indoc::indoc! {"
air_gap_winding_height: 10 mm
"};

let ag: PlainAirGap = yaml_serde::from_str(&str).expect("valid dimensions");
assert_eq!(ag.winding_coverage(), 0.0);
```
 */
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PlainAirGap {
    #[cfg_attr(feature = "serde", serde(default = "usize::default"))]
    num_segments: usize,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(
        feature = "serde",
        serde(
            deserialize_with = "super::deserialize_nonnegative_length",
            default = "super::zero_length"
        )
    )]
    air_gap_winding_height: Length,
    #[cfg_attr(
        feature = "serde",
        serde(
            deserialize_with = "deserialize_winding_coverage",
            default = "f64::default"
        )
    )]
    winding_coverage: f64,
    #[cfg_attr(feature = "serde", serde(default = "bool::default"))]
    starts_in_slot_middle: bool,
    #[cfg_attr(feature = "serde", serde(default = "u16::default"))]
    slots: u16,
}

impl PlainAirGap {
    /**
    Creates a new [`PlainAirGap`] from valid input data.

    The creation fails if `air_gap_winding_height` is negative.
    `winding_coverage` is clamped between 0 and 1. All other data is straight
    fed into the [`PlainAirGap`] struct.

    # Examples

    ```
    use stem_core::prelude::*;

    // Valid input data
    assert!(PlainAirGap::new(2, Length::new::<millimeter>(1.0), 0.5, 12, true).is_ok());

    // Winding coverage gets clamped
    let ag = PlainAirGap::new(2, Length::new::<millimeter>(1.0), 2.0, 12, true).expect("valid input data");
    assert_eq!(ag.winding_coverage(), 1.0);

    // Negative winding height -> creation fails
    assert!(PlainAirGap::new(2, Length::new::<millimeter>(-1.0), 0.5, 12, true).is_err());
    ```
     */
    pub fn new(
        num_segments: usize,
        air_gap_winding_height: Length,
        winding_coverage: f64,
        slots: u16,
        starts_in_slot_middle: bool,
    ) -> Result<Self, Error> {
        let zero_length = Length::new::<meter>(0.0);
        compare_variables!(air_gap_winding_height >= zero_length)?;
        let winding_coverage = winding_coverage.clamp(0.0, 1.0);
        return Ok(Self {
            air_gap_winding_height,
            winding_coverage,
            num_segments,
            slots,
            starts_in_slot_middle,
        });
    }

    /**
    Creates a new [`PlainAirGap`] where all values except `num_segments` are set
    to their default values (see [`PlainAirGap`] docstring).

    This method allows creating a [`PlainAirGap`] where only the
    [`PlainAirGap::num_segments`] is specified. All other fields are set to
    their default values, meaning that the resulting air gap cannot hold a
    winding. Hence, this method is essentially an alternative to the
    [`Default`] implementation of [`PlainAirGap`] if `num_segments` should not
    be zero.

    # Examples

    ```
    use stem_core::prelude::*;

    let ag = PlainAirGap::with_num_segments(2);
    assert_eq!(ag.air_gap_winding_height().get::<meter>(), 0.0);
    assert_eq!(ag.winding_coverage(), 0.0);
    ```
     */
    pub fn with_num_segments(num_segments: usize) -> Self {
        Self {
            air_gap_winding_height: Length::new::<meter>(0.0),
            winding_coverage: 0.0,
            num_segments,
            slots: 0,
            starts_in_slot_middle: true,
        }
    }

    /// Returns the height of the air gap winding space.
    ///
    /// This parameter shows how much the air gap winding extends into the air
    /// gap itself. It therefore must not be negative. If it is zero, the
    /// available winding space is also zero and the core cannot be wound in
    /// practice. Therefore, [`PlainAirGap::slots`] then also returns 0.
    ///
    /// The image below shows a double-layer winding with the (hatched) layers
    /// arranged horizontally next to each other
    /// ([`CoilLayout::DoubleHorizontal`]).
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
    pub fn air_gap_winding_height(&self) -> Length {
        return self.air_gap_winding_height;
    }

    /// Returns the proportional coverage of the air gap surface by the air gap
    /// winding space.
    ///
    /// This parameter is between zero and one. If it is zero, the
    /// available winding space is also zero and the core cannot be wound in
    /// practice (even if [`PlainAirGap::slots`] is larger than zero). If it is
    /// one, the entire air gap surface is covered by coils if the core is
    /// wound. Therefore, [`PlainAirGap::slots`] then also returns 0.
    ///
    /// As shown in the image below, this value is the ratio betwen the "slot"
    /// (coil group) width and the [`CoreExt::slot_pitch`]. The depicted winding
    /// is a double-layer winding with the (hatched) layers arranged
    /// horizontally next to each other ([`CoilLayout::DoubleHorizontal`]).
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
    pub fn winding_coverage(&self) -> f64 {
        return self.winding_coverage;
    }

    /// Whether the air gap surface starts in the middle of a slot or inbetween
    /// two slots.
    ///
    /// The image below shows the influence of this parameter on the positioning
    /// of the individual winding layers for a double layer winding:
    #[doc = ""]
    #[cfg_attr(
        feature = "doc-images",
        doc = "![Effect of the starts_in_slot_middle parameter][cad_plain_air_gap_winding]"
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
    /// If the "slot" cannot be separated horizontally (e.g. in case of a
    /// ([`CoilLayout::Single`])), the layers will protrude outside the air gap
    /// for a linear core. This is obviously not desirable, which is why
    /// this parameter should generally only be `true` for coil layouts
    /// which can be separated horizontally
    /// ([`CoilLayout::DoubleHorizontal`], [`CoilLayout::Quadruple`]).
    /// For a rotary core, this is not the case, as there the parameter only
    /// influences whether the first slot is positioned on the x-axis or not.
    pub fn starts_in_slot_middle(&self) -> bool {
        return self.starts_in_slot_middle;
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl AirGap for PlainAirGap {
    fn num_segments(&self, _: CoreRef<'_>) -> usize {
        return self.num_segments;
    }

    fn winding_zones(&self, core: CoreRef<'_>, coil_layout: &CoilLayout) -> WindingZones {
        match core {
            CoreRef::Lin(lin) => WindingZones::WindingZonesEqSpacedLin(WindingZonesEqSpaced::<
                Contour,
                true,
            >::from_air_gap_winding(
                lin.air_gap_length(),
                lin.slots(),
                self.air_gap_winding_height(),
                self.winding_coverage(),
                coil_layout,
                self.starts_in_slot_middle(),
                true,
            )),
            CoreRef::Rot(rot) => WindingZones::WindingZonesEqSpacedRot(WindingZonesEqSpaced::<
                Contour,
                false,
            >::from_air_gap_winding(
                rot.air_gap_length(),
                rot.slots(),
                self.air_gap_winding_height(),
                self.winding_coverage(),
                coil_layout,
                self.starts_in_slot_middle(),
                rot.is_outer(),
            )),
        }
    }

    fn slots(&self, _: CoreRef<'_>) -> u16 {
        let m = u16::from(
            self.air_gap_winding_height > super::zero_length() && self.winding_coverage > 0.0,
        );
        return m * self.slots;
    }

    fn surface_magnets(
        &self,
        magnet_assembly: &MagnetAssembly,
        core: CoreRef<'_>,
        split: bool,
    ) -> Magnets {
        match core {
            CoreRef::Lin(_) => EvenlyDistributedMagnets::<true>::from_magnet_assembly(
                core.poles().into(),
                core.air_gap_length(),
                magnet_assembly,
                split,
                true, // Value is ignored anyway if LIN = true
                core.d_axis_offset(),
            )
            .into(),
            CoreRef::Rot(core_rot) => EvenlyDistributedMagnets::<false>::from_magnet_assembly(
                core.poles().into(),
                core.air_gap_length(),
                magnet_assembly,
                split,
                core_rot.is_outer(),
                core.d_axis_offset(),
            )
            .into(),
        }
    }

    fn combine(&mut self, core: CoreRef<'_>) -> Result<Shape, Error> {
        let shape = match core {
            CoreRef::Lin(core) => {
                let contour = Contour::rectangle(
                    [0.0, 0.0],
                    [core.width().get::<meter>(), core.height().get::<meter>()],
                );
                Shape::new(vec![contour])?
            }
            CoreRef::Rot(core) => {
                // Check if the air gap winding fits the core
                if core.is_outer() {
                    let air_gap_radius = core.air_gap_radius();
                    compare_variables!(self.air_gap_winding_height <= air_gap_radius)?;
                }

                // Create the air gap circle
                let air_gap = ArcSegment::from_center_radius_start_sweep_angle(
                    [0.0, 0.0],
                    core.air_gap_radius().get::<meter>(),
                    0.0,
                    2.0_f64 * PI,
                )?
                .into();

                // Create the yoke circle
                let yoke = ArcSegment::from_center_radius_start_sweep_angle(
                    [0.0, 0.0],
                    core.yoke_radius().get::<meter>(),
                    0.0,
                    2.0_f64 * PI,
                )?
                .into();

                super::combine_air_gap_and_yoke_to_shape(air_gap, yoke, core.is_outer())?
            }
        };
        return Ok(shape);
    }

    fn slot_opening_factor(&self, core: CoreRef<'_>, mech_ordinal: i32) -> f64 {
        let slot_pitch = core.slot_pitch();
        return super::slot_opening_factor(
            slot_pitch,
            slot_pitch * self.winding_coverage,
            self.slots,
            mech_ordinal,
        );
    }

    fn carter_factor(&self, _core: CoreRef<'_>, _air_gap_width: Length) -> f64 {
        return 1.0;
    }

    fn slot(&self, _core: CoreRef<'_>) -> Option<&dyn stem_slot::slot::Slot> {
        return None;
    }
}
