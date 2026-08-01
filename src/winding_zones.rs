use crate::planar_geo;
use planar_geo::DEFAULT_EPSILON;
use planar_geo::prelude::{Polysegment, ToBoundingBox};
use planar_geo::segment::ArcSegment;
use planar_geo::{Transformation, contour::Contour};
use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, TAU};
use stem_slot::planar_geo::draw::Drawable;
use stem_slot::prelude::*;
use stem_slot::{coil_layout::CoilLayout, slot::Slot};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/**
A "zone" is a position in the zone plan defined by `slot` and `layer`
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Zone {
    pub slot: u16,
    pub layer: u16,
}

impl Zone {
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

#[derive(Debug, Clone)]
pub struct PositionedZoneContour {
    pub contour: Contour,
    pub zone: Zone,
}

impl PositionedZoneContour {
    #[cfg(feature = "cairo")]
    pub fn into_drawable(self) -> Drawable {
        Drawable::new(self.contour, stem_slot::SLOT_STYLE)
    }

    #[cfg(feature = "cairo")]
    pub fn into_drawable_with_style(self, style: planar_geo::draw::Style) -> Drawable {
        Drawable::new(self.contour, style)
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
ccw on a rotary core means counter-clockwise
ccw on a linear core means from left to right
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

    pub fn no_slots() -> Self {
        return Self {
            slots: 0,
            air_gap_length: Length::new::<meter>(0.0),
            zones: Vec::new(),
            starts_in_slot_middle: true,
            index: 0,
        };
    }

    pub fn layers(&self) -> usize {
        return self.zones.len();
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
        let mut contour = self.zones[current_layer].clone();

        if LIN {
            // If all vertices of the contour are negative, shift it to the
            // end of the core. This can only happen for the first slot in
            // case the slot starts in the tooth middle
            let factor = if current_slot == 0.0
                && self.starts_in_slot_middle
                && contour.bounding_box().xmax() <= DEFAULT_EPSILON.sqrt()
            {
                1.0
            } else {
                1.0 / f64::from(self.slots)
                    * (current_slot + 0.5 * (!self.starts_in_slot_middle) as u32 as f64)
            };
            contour.translate([self.air_gap_length.get::<meter>() * factor, 0.0]);
        } else {
            contour.translate([0.0, self.air_gap_length.get::<meter>() / TAU]);
            let angle = -TAU
                * (current_slot + 0.5 * (!self.starts_in_slot_middle) as u32 as f64) as f64
                / self.slots as f64
                + FRAC_PI_2;
            contour.rotate([0.0, 0.0], angle);
        }
        return Some((
            contour,
            Zone {
                slot: current_slot as u16,
                layer: current_layer as u16,
            },
        ));
    }
}

impl<const LIN: bool> WindingZonesEqSpaced<Polysegment, LIN> {
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

        if !outer_core {
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
        if !outer_core {
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

pub enum WindingZones {
    WindingZonesEqSpacedLin(WindingZonesEqSpaced<Contour, true>),
    WindingZonesEqSpacedRot(WindingZonesEqSpaced<Contour, false>),
    Other(Box<dyn Iterator<Item = PositionedZoneContour>>),
}

impl Iterator for WindingZones {
    type Item = PositionedZoneContour;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            WindingZones::WindingZonesEqSpacedLin(i) => i.next(),
            WindingZones::WindingZonesEqSpacedRot(i) => i.next(),
            WindingZones::Other(i) => i.next(),
        }
    }
}

impl From<WindingZonesEqSpaced<Contour, true>> for WindingZones {
    fn from(value: WindingZonesEqSpaced<Contour, true>) -> Self {
        Self::WindingZonesEqSpacedLin(value)
    }
}

impl From<WindingZonesEqSpaced<Contour, false>> for WindingZones {
    fn from(value: WindingZonesEqSpaced<Contour, false>) -> Self {
        Self::WindingZonesEqSpacedRot(value)
    }
}

impl From<Box<dyn Iterator<Item = PositionedZoneContour>>> for WindingZones {
    fn from(value: Box<dyn Iterator<Item = PositionedZoneContour>>) -> Self {
        Self::Other(value)
    }
}
