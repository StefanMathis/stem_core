use crate::planar_geo;
use planar_geo::prelude::*;
use stem_magnet::assembly::MagnetAssembly;
use stem_slot::{
    coil_layout::CoilLayout, current_displacement::CurrentDisplacementCalculator,
    prelude::stem_material::prelude::*, slot::Slot,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    air_gap::AirGap,
    core::{CoreExt, CoreRef, LinCore, RotCore},
    error::Error,
    magnets::{EvenlyDistributedMagnets, Magnets},
    winding_zones::{WindingZones, WindingZonesEqSpaced},
};

/// Carter factor models for CoreRotSlotted and CoreLinSlotted from a variety of
/// literature sources. The explanation of those models can be found in the
/// literature overview
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CarterFactorModel {
    Bin12,
    MVP08,
    PS62,
}

impl CarterFactorModel {
    pub fn carter_factor(
        &self,
        air_gap_length: Length,
        opening_width: Length,
        slot_pitch: Length,
    ) -> f64 {
        match self {
            Self::Bin12 => {
                let val = f64::from(opening_width / air_gap_length);
                let gamma = val.powi(2) / (5.0 + val);
                return f64::from(slot_pitch / (slot_pitch - gamma * air_gap_length));
            }
            Self::MVP08 => {
                let gamma = opening_width / (5.0 * air_gap_length + opening_width);
                return f64::from(slot_pitch / (slot_pitch - gamma * air_gap_length));
            }
            Self::PS62 => {
                return f64::from(
                    (slot_pitch + 10.0 * air_gap_length)
                        / (slot_pitch - opening_width + 10.0 * air_gap_length),
                );
            }
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SlottedAirGap {
    slots: u16,
    starts_in_slot_middle: bool,
    carter_factor_model: CarterFactorModel,
    slot: Box<dyn Slot>, // Slot of the core
}

impl SlottedAirGap {
    pub fn new(
        slots: u16,
        starts_in_slot_middle: bool,
        carter_factor_model: CarterFactorModel,
        slot: Box<dyn Slot>,
    ) -> Self {
        return Self {
            slots,
            starts_in_slot_middle,
            carter_factor_model,
            slot,
        };
    }

    pub fn starts_in_slot_middle(&self) -> bool {
        return self.starts_in_slot_middle;
    }

    pub fn carter_factor_model(&self) -> &CarterFactorModel {
        return &self.carter_factor_model;
    }

    pub fn slot(&self) -> &dyn Slot {
        return &*self.slot;
    }

    /// Returns the core shape for a linear core, if the combination of `self`
    /// and `core` results in a valid shape.
    fn shape_lin(&self, core: &LinCore) -> Result<Shape, Error> {
        // Returns [right half, left half]
        fn split_vertical(ps: Polysegment) -> impl Iterator<Item = Polysegment> {
            let bb = ps.bounding_box();

            let verts_par = [[0.0, bb.ymin() - 1.0], [0.0, bb.ymax() + 1.0]];
            let vertical_line = Polysegment::from_points(verts_par.as_slice());
            let separated =
                ps.intersection_cut(&vertical_line, DEFAULT_EPSILON, DEFAULT_MAX_RELATIVE);

            return separated.into_iter().rev().filter(|ps| !ps.is_empty());
        }

        let is_open = self.slot.is_open();

        let mut slot_iter = WindingZonesEqSpaced::<Polysegment, true>::from_slot(
            core.air_gap_length(),
            self.slots,
            &*self.slot,
            self.starts_in_slot_middle,
            true,
        );

        if is_open {
            let mut ps = Polysegment::new();

            if let Ok(ls) = LineSegment::new(
                [core.width().get::<meter>(), core.height().get::<meter>()],
                [0.0, core.height().get::<meter>()],
            ) {
                ps.push_back(ls.into());
            }

            let mut halfes = if self.starts_in_slot_middle
                && let Some(first_slot) = slot_iter.next()
            {
                Some(split_vertical(first_slot))
            } else {
                None
            };

            if let Some(mut right_half) = halfes.as_mut().map(|i| i.next()).flatten() {
                ps.append(&mut right_half);
            }

            if !self.starts_in_slot_middle {
                ps.extend_back([0.0, 0.0]);
            }

            for mut slot in slot_iter {
                ps.append(&mut slot);
            }

            if !self.starts_in_slot_middle {
                ps.extend_back([core.width().get::<meter>(), 0.0]);
            }

            if let Some(mut left_half) = halfes.as_mut().map(|i| i.next()).flatten() {
                left_half.translate([core.width().get::<meter>(), 0.0]);
                ps.append(&mut left_half);
            }

            return Shape::try_from(ps).map_err(From::from);
        } else {
            let mut shape = if self.starts_in_slot_middle
                && let Some(first_slot) = slot_iter.next()
            {
                let mut halfes = split_vertical(first_slot);

                let mut ps = Polysegment::new();
                if let Ok(ls) = LineSegment::new(
                    [core.width().get::<meter>(), core.height().get::<meter>()],
                    [0.0, core.height().get::<meter>()],
                ) {
                    ps.push_back(ls.into());
                }
                if let Some(mut left_half) = halfes.next() {
                    ps.append(&mut left_half);
                }
                ps.extend_back([0.0, 0.0]);
                ps.extend_back([core.width().get::<meter>(), 0.0]);
                if let Some(mut right_half) = halfes.next() {
                    right_half.translate([core.width().get::<meter>(), 0.0]);
                    ps.append(&mut right_half);
                }
                Shape::try_from(ps)?
            } else {
                let contour = Contour::rectangle(
                    [0.0, 0.0],
                    [core.width().get::<meter>(), core.height().get::<meter>()],
                );
                Shape::try_from(contour)?
            };

            // Add all of the closed slots
            for slot_contour in slot_iter {
                shape.add_hole(slot_contour.into())?;
            }

            return Ok(shape);
        }
    }

    /// Returns the core shape for a rotary core, if the combination of `self`
    /// and `core` results in a valid shape.
    fn shape_rot(&self, core: &RotCore) -> Result<Shape, Error> {
        let yoke = ArcSegment::circle([0.0, 0.0], core.yoke_radius().get::<meter>())?;
        let air_gap_radius = core.air_gap_radius().get::<meter>();
        let is_open = self.slot.is_open();

        let slot_iter = WindingZonesEqSpaced::<Polysegment, false>::from_slot(
            core.air_gap_length(),
            self.slots,
            &*self.slot,
            self.starts_in_slot_middle,
            core.is_outer(),
        );

        if is_open {
            let mut air_gap = Polysegment::new();
            let mut last_stop_opt = None;

            for mut slot_outline in slot_iter {
                if let Some(last_stop) = last_stop_opt {
                    if let Some(next_start) = slot_outline.front().map(|s| s.start()) {
                        if let Ok(a) = ArcSegment::from_start_stop_center_radius(
                            last_stop,
                            next_start,
                            [0.0, 0.0],
                            air_gap_radius,
                            false,
                        ) {
                            air_gap.push_back(a.into());
                        }
                    }
                }

                last_stop_opt = slot_outline.back().map(|s| s.stop());
                air_gap.append(&mut slot_outline);
            }

            // Close the air gap outline. To close the line, we need to connect
            // from stop to start.
            if let Some(start) = air_gap.front().map(|s| s.start()) {
                if let Some(stop) = air_gap.back().map(|s| s.stop()) {
                    if let Ok(a) = ArcSegment::from_start_stop_center_radius(
                        stop,
                        start,
                        [0.0, 0.0],
                        air_gap_radius,
                        false,
                    ) {
                        air_gap.push_back(a.into());
                    }
                }
            } else {
                // No segment in the air gap -> air gap is a simple circle
                air_gap.push_back(ArcSegment::circle([0.0, 0.0], air_gap_radius)?.into());
            }

            if core.is_outer() {
                return Shape::new(vec![yoke.into(), air_gap.into()]).map_err(From::from);
            } else {
                return Shape::new(vec![air_gap.into(), yoke.into()]).map_err(From::from);
            }
        } else {
            let air_gap = ArcSegment::circle([0.0, 0.0], air_gap_radius)?;
            let mut shape = if core.is_outer() {
                Shape::new(vec![yoke.into(), air_gap.into()])?
            } else {
                Shape::new(vec![air_gap.into(), yoke.into()])?
            };

            // Add all of the closed slots
            for slot_contour in slot_iter {
                shape.add_hole(slot_contour.into())?;
            }
            return Ok(shape);
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl AirGap for SlottedAirGap {
    fn num_segments(&self, _: CoreRef<'_>) -> usize {
        return 0;
    }

    fn tooth_height(&self, _: CoreRef<'_>) -> Length {
        return self.slot.height();
    }

    fn tooth_width_at(&self, core: CoreRef<'_>, height: Length) -> Length {
        if height < Length::new::<meter>(0.0) {
            return Length::new::<meter>(0.0);
        }

        match core {
            CoreRef::Lin(lin_core) => {
                return lin_core.width() / self.slots(core) as f64 - self.slot.width_at(height);
            }
            CoreRef::Rot(rot_core) => {
                let width = self.slot.width_at(height).get::<meter>();
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

    fn winding_zones(&self, core: CoreRef<'_>, coil_layout: &CoilLayout) -> WindingZones {
        match core {
            CoreRef::Lin(_) => WindingZones::WindingZonesEqSpacedLin(WindingZonesEqSpaced::<
                Contour,
                true,
            >::from_slot(
                core.air_gap_length(),
                core.slots(),
                &*self.slot,
                coil_layout,
                self.starts_in_slot_middle(),
                true,
            )),
            CoreRef::Rot(rot) => WindingZones::WindingZonesEqSpacedRot(WindingZonesEqSpaced::<
                Contour,
                false,
            >::from_slot(
                core.air_gap_length(),
                core.slots(),
                &*self.slot,
                coil_layout,
                self.starts_in_slot_middle(),
                rot.is_outer(),
            )),
        }
    }

    fn slots(&self, _: CoreRef<'_>) -> u16 {
        return self.slots;
    }

    fn slot(&self, _core: CoreRef<'_>) -> Option<&dyn Slot> {
        return Some(&*self.slot);
    }

    fn zone_area(&self, _: CoreRef<'_>) -> Area {
        Area::new::<square_meter>(0.0)
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
        match core {
            CoreRef::Lin(lin_core) => self.shape_lin(lin_core),
            CoreRef::Rot(rot_core) => self.shape_rot(rot_core),
        }
    }

    fn current_displacement_coefficients(
        &self,
        _core: CoreRef<'_>,
    ) -> CurrentDisplacementCalculator {
        return self.slot.current_displacement_coefficients(50);
    }

    fn carter_factor(&self, core: CoreRef<'_>, air_gap_length: Length) -> f64 {
        return self.carter_factor_model().carter_factor(
            air_gap_length,
            self.slot().opening_width(),
            core.slot_pitch(),
        );
    }

    fn slot_opening_factor(&self, core: CoreRef<'_>, mech_ordinal: i32) -> f64 {
        let slot_pitch = core.slot_pitch();
        return super::slot_opening_factor(
            slot_pitch,
            self.slot.opening_width(),
            self.slots,
            mech_ordinal,
        );
    }
}
