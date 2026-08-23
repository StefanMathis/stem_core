use std::f64::consts::FRAC_PI_2;
use std::{path::PathBuf, sync::Arc};

use cairo_viewport::{BoundingBox, SideLength, Viewport};
use stem_core::prelude::*;
use stem_slot::semi_trapezoid::SemiTrapezoidWidthsAndHeightsBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    full_core_assembly_plot()?;
    return Ok(());
}

fn full_core_assembly_plot() -> Result<(), Box<dyn std::error::Error>> {
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

    let air_gap = SlottedAirGap::new(28, false, CarterFactorModel::Bin12, Box::new(slot));

    let fb = V1rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(4.0),
        relief_path_air_gap_width: Length::new::<millimeter>(5.0),
        relief_path_length: Length::new::<millimeter>(0.0),
        relief_path_width: Length::new::<millimeter>(1.0),
        opening_angle: FRAC_PI_2,
        magnet_space_width: Length::new::<millimeter>(6.0),
        magnet_space_height: Length::new::<millimeter>(23.0),
        glue_gap: Length::new::<millimeter>(0.2),
        leakage_path_width: Length::new::<millimeter>(1.0),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
    };

    let rot_core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(18.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        skew_angle: 0.0,
        air_gap: Box::new(air_gap),
        flux_barrier: Some(Box::new(fb)),
    }
    .try_into()?;
    let magnet = ArcParallelMagnet::with_const_thickness(
        rot_core.axial_length(),
        rot_core.air_gap_radius(),
        SideHeightOrThickness::Thickness(Length::new::<millimeter>(4.0)),
        AngleOrWidth::Angle(0.5 * FRAC_PI_2 / 2.0),
        Arc::new(Material::default()),
    )?;
    let surface_magnets = MagnetAssembly::new(magnet, 1.try_into()?, 2.try_into()?);

    let fp =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&format!("docs/img/full_core_assembly.svg"));

    let bb = BoundingBox::new(-60.0e-3, 60.0e-3, -60.0e-3, 60.0e-3);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        rot_core.drawable().draw(cr)?;
        for m in rot_core.surface_magnets(&surface_magnets, true) {
            m.into_drawable().draw(cr)?;
        }

        for m in rot_core.interior_magnets(true) {
            m.into_drawable().draw(cr)?;
        }

        for z in rot_core.winding_zones(&CoilLayout::DoubleHorizontal) {
            z.into_drawable().draw(cr)?;
        }

        return Ok(());
    })?;

    return Ok(());
}
