use std::f64::consts::FRAC_PI_2;
use std::{path::PathBuf, sync::Arc};

use cairo_viewport::{BoundingBox, SideLength, Viewport};
use stem_core::prelude::*;
use stem_slot::semi_trapezoid::SemiTrapezoidWidthsAndHeightsBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let slot: SemiTrapezoidSlot = SemiTrapezoidWidthsAndHeightsBuilder {
        bottom_width: Length::new::<millimeter>(6.76),
        bottom_side_width: Length::new::<millimeter>(6.76),
        top_side_width: Length::new::<millimeter>(8.0),
        top_width: Length::new::<millimeter>(1.5),
        opening_width: Length::new::<millimeter>(1.5),
        bottom_height: Length::new::<millimeter>(0.0),
        side_height: Length::new::<millimeter>(6.79 - 0.75 - 0.5),
        top_height: Length::new::<millimeter>(0.5),
        opening_height: Length::new::<millimeter>(0.75),
        bottom_radius: Length::new::<millimeter>(0.0),
        bottom_side_radius: Length::new::<millimeter>(0.0),
        top_radius: Length::new::<millimeter>(0.0),
        top_side_radius: Length::new::<millimeter>(0.0),
        opening_radius: Length::new::<millimeter>(0.0),
        consider_tooth_tip_leakage: true,
    }
    .try_into()?;

    let barrier = V1rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(4.5),
        relief_path_air_gap_width: Length::new::<millimeter>(3.5),
        relief_path_length: Length::new::<millimeter>(1.33),
        relief_path_width: Length::new::<millimeter>(4.0),
        opening_angle: FRAC_PI_2,
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height: Length::new::<millimeter>(20.0),
        glue_gap: Length::new::<millimeter>(0.2),
        leakage_path_width: Length::new::<millimeter>(1.0),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
    };

    let air_gap = SlottedAirGap::new(28, false, CarterFactorModel::Bin12, Box::new(slot));
    let core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(54.4),
        yoke_radius: Length::new::<millimeter>(19.0),
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

    let shift = 0.12;

    let mut core_fb = core.clone();
    core_fb.set_flux_barrier(Some(Box::new(barrier)))?;

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&format!("docs/img/slotted_core_with_and_without_fb.svg"));

    let bb = BoundingBox::new(-60.0e-3, shift + 60.0e-3, -60.0e-3, 60.0e-3);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        core.drawable().draw(cr)?;
        cr.translate(shift, 0.0);
        core_fb.drawable().draw(cr)?;

        return Ok(());
    })?;

    return Ok(());
}
