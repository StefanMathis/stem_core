use std::f64::consts::TAU;
use std::{path::PathBuf, sync::Arc};

use cairo_viewport::{BoundingBox, SideLength, Viewport};
use planar_geo::Transformation;
use planar_geo::draw::*;
use stem_core::prelude::*;
use stem_slot::planar_geo::prelude::Composite;
use stem_slot::semi_trapezoid::SemiTrapezoidWithoutSlopesBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let air_gap =
        PlainAirGap::new(Length::new::<millimeter>(4.0), 0.8, 1, 24, false).expect("valid");
    let core_plain: RotCore = RotCoreBuilder {
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
