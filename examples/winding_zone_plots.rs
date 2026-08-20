use std::f64::consts::TAU;
use std::{path::PathBuf, sync::Arc};

use cairo_viewport::{BoundingBox, SideLength, Viewport};
use planar_geo::Transformation;
use planar_geo::draw::*;
use stem_core::prelude::*;
use stem_slot::planar_geo::prelude::Composite;
use stem_slot::semi_trapezoid::SemiTrapezoidWithoutSlopesBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    plot_slotted_and_air_gap_rot()?;
    plot_slotted_and_air_gap_lin_with_zone_pos()?;
    return Ok(());
}

fn plot_slotted_and_air_gap_lin_with_zone_pos() -> Result<(), Box<dyn std::error::Error>> {
    let slot = RectangularSlot::new(
        Length::new::<millimeter>(10.0),
        Length::new::<millimeter>(4.0),
        Length::new::<millimeter>(15.0),
        Length::new::<millimeter>(2.0),
        true,
    )?;

    let core_plain: LinCore = LinCoreBuilder {
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
    .try_into()?;

    let core_slotted: LinCore = LinCoreBuilder {
        height: Length::new::<millimeter>(25.0),
        width: Length::new::<millimeter>(50.0),
        axial_length: Length::new::<millimeter>(1.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Default::default()),
        pole_pairs: 1,
        air_gap: Box::new(SlottedAirGap {
            slots: 3,
            starts_in_slot_middle: false,
            carter_factor_model: CarterFactorModel::Bin12,
            slot: Box::new(slot),
        }),
        flux_barrier: None,
    }
    .try_into()?;

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&format!("docs/img/winding_zones_with_zone_pos.svg"));

    let bb = BoundingBox::new(-1.0e-3, 111.0e-3, -1.0e-3, 26.0e-3);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        cr.translate(0.0, 0.01);

        core_plain.drawable().draw(cr)?;
        for w in core_plain.winding_zones(&CoilLayout::DoubleVertical) {
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

        cr.translate(core_plain.width().get::<meter>() + 0.01, -0.01);

        core_slotted.drawable().draw(cr)?;
        for w in core_slotted.winding_zones(&CoilLayout::DoubleVertical) {
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

        return Ok(());
    })?;

    return Ok(());
}

fn plot_slotted_and_air_gap_rot() -> Result<(), Box<dyn std::error::Error>> {
    let plain_air_gap = PlainAirGap {
        num_segments: 0,
        air_gap_winding_height: Length::new::<millimeter>(4.0),
        winding_coverage: 0.8,
        starts_in_slot_middle: false,
        slots: 24,
    };

    let core_plain: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(18.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        skew_angle: 0.0,
        air_gap: Box::new(plain_air_gap),
        flux_barrier: None,
    }
    .try_into()?;

    let slot: SemiTrapezoidSlot = SemiTrapezoidWithoutSlopesBuilder {
        bottom_width: Length::new::<millimeter>(4.0),
        opening_width: Length::new::<millimeter>(2.0),
        height: Length::new::<millimeter>(20.0),
        opening_height: Length::new::<millimeter>(2.0),
        slot_angle: -TAU / 24.0,
        bottom_radius: Length::new::<millimeter>(2.0),
        top_radius: Length::new::<millimeter>(1.0),
        opening_radius: Length::new::<millimeter>(0.0),
        consider_tooth_tip_leakage: true,
    }
    .try_into()
    .unwrap();
    let air_gap_slotted = SlottedAirGap::new(24, false, CarterFactorModel::Bin12, Box::new(slot));
    let core_slotted: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(18.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        skew_angle: 0.0,
        air_gap: Box::new(air_gap_slotted),
        flux_barrier: None,
    }
    .try_into()
    .unwrap();

    let shift = 0.12;

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&format!("docs/img/winding_zones.svg"));

    let bb = BoundingBox::new(-60.0e-3, shift + 60.0e-3, -60.0e-3, 60.0e-3);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        core_plain.drawable().draw(cr)?;
        for (idx, w) in core_plain
            .winding_zones(&CoilLayout::DoubleVertical)
            .enumerate()
        {
            let text = Text {
                text: idx.to_string(),
                anchor: Anchor::Center,
                fixed_anchor_offset: [0.0, 0.0],
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
            w.into_drawable().draw(cr)?;
            text.draw(cr)?;
        }

        let mut drawable: Drawable = core_slotted.drawable().into();
        drawable.translate([shift, 0.0]);
        drawable.draw(cr)?;
        for (idx, mut w) in core_slotted
            .winding_zones(&CoilLayout::DoubleVertical)
            .enumerate()
        {
            w.translate([shift, 0.0]);
            let text = Text {
                text: idx.to_string(),
                anchor: Anchor::Center,
                fixed_anchor_offset: [0.0, 0.0],
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
            w.into_drawable().draw(cr)?;
            text.draw(cr)?;
        }

        return Ok(());
    })?;

    return Ok(());
}
