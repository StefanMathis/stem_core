use std::{f64::consts::PI, num::NonZero};

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

/// Used as default constructor for field
/// [`PlainAirGap::air_gap_winding_height`]
fn zero_length() -> Length {
    return Length::new::<meter>(0.0);
}

/**
If you don't care about adding a winding, but care about core segmentation -> PlainAirGap::with_num_segments
If you don't care about adding a winding and core segmentation -> PlainAirGap::default
 */
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PlainAirGap {
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "deserialize_quantity", default = "zero_length")
    )]
    air_gap_winding_height: Length,
    winding_coverage: f64,
    num_segments: usize,
    starts_in_slot_middle: bool,
    slots: u16,
}

impl PlainAirGap {
    pub fn new(
        air_gap_winding_height: Length,
        winding_coverage: f64,
        num_segments: usize,
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

    pub fn with_num_segments(num_segments: NonZero<usize>) -> Self {
        Self {
            air_gap_winding_height: Length::new::<meter>(0.0),
            winding_coverage: 0.0,
            num_segments: num_segments.into(),
            slots: 0,
            starts_in_slot_middle: true,
        }
    }

    pub fn air_gap_winding_height(&self) -> Length {
        return self.air_gap_winding_height;
    }

    pub fn winding_coverage(&self) -> f64 {
        return self.winding_coverage;
    }

    pub fn num_segments(&self) -> usize {
        return self.num_segments;
    }

    pub fn starts_in_slot_middle(&self) -> bool {
        return self.starts_in_slot_middle;
    }
}

impl Default for PlainAirGap {
    fn default() -> Self {
        Self {
            air_gap_winding_height: Length::new::<meter>(0.0),
            winding_coverage: 0.0,
            num_segments: 1,
            slots: 1,
            starts_in_slot_middle: true,
        }
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
        return self.slots;
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
            )
            .into(),
            CoreRef::Rot(core_rot) => EvenlyDistributedMagnets::<false>::from_magnet_assembly(
                core.poles().into(),
                core.air_gap_length(),
                magnet_assembly,
                split,
                core_rot.is_outer(),
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
}
