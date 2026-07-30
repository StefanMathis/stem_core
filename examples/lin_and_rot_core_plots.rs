use std::f64::consts::PI;
use std::{path::PathBuf, sync::Arc};

use cairo_viewport::bounding_box::ToBoundingBox;
use cairo_viewport::{BoundingBox, SideLength, Viewport};
use planar_geo::Transformation;
use planar_geo::draw::*;
use stem_core::prelude::*;
use stem_slot::semi_trapezoid::{
    SemiTrapezoidWidthsAndHeightsBuilder, SemiTrapezoidWithoutSlopesBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    plot_lin_core()?;
    plot_rot_core()?;
    plot_lin_and_rot_core_common_image()?;
    return Ok(());
}

fn plot_lin_core() -> Result<(), Box<dyn std::error::Error>> {
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
    .try_into()
    .unwrap();

    let air_gap = SlottedAirGap::new(12, false, CarterFactorModel::Bin12, Box::new(slot.clone()));
    let flux_barrier = Star1FluxBarrier {
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(4.0),
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<
            millimeter,
        >(2.0)),
        glue_gap: Length::new::<millimeter>(0.0),
        magnet_material: None,
        cache: None,
    };

    let lin_core: LinCore = LinCoreBuilder {
        height: Length::new::<millimeter>(30.0),
        width: Length::new::<millimeter>(150.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        air_gap: Box::new(air_gap),
        flux_barrier: Some(Box::new(flux_barrier)),
    }
    .try_into()?;

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&format!("docs/img/lin_core.svg"));

    let mut bb = lin_core.shape().bounding_box();
    bb.scale(1.01);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(400));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        lin_core.drawable().draw(cr)?;

        return Ok(());
    })?;
    return Ok(());
}

fn plot_rot_core() -> Result<(), Box<dyn std::error::Error>> {
    let air_gap_plain = PlainAirGap::new(Length::new::<meter>(0.0), 0.0, 1, 0, true)?;

    let flux_barrier = Star1FluxBarrier {
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(4.0),
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<
            millimeter,
        >(2.0)),
        glue_gap: Length::new::<millimeter>(0.0),
        magnet_material: None,
        cache: None,
    };

    let inner_core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(53.0),
        yoke_radius: Length::new::<millimeter>(19.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 3,
        skew_angle: 0.0,
        air_gap: Box::new(air_gap_plain),
        flux_barrier: Some(Box::new(flux_barrier)),
    }
    .try_into()
    .unwrap();

    let slot: SemiTrapezoidSlot = SemiTrapezoidWithoutSlopesBuilder {
        bottom_width: Length::new::<millimeter>(9.0),
        opening_width: Length::new::<millimeter>(2.0),
        height: Length::new::<millimeter>(20.0),
        opening_height: Length::new::<millimeter>(2.0),
        slot_angle: 10.0 * PI / 180.0,
        bottom_radius: Length::new::<millimeter>(2.0),
        top_radius: Length::new::<millimeter>(1.0),
        opening_radius: Length::new::<millimeter>(0.0),
        consider_tooth_tip_leakage: true,
    }
    .try_into()
    .unwrap();

    let air_gap_slotted = SlottedAirGap::new(36, false, CarterFactorModel::Bin12, Box::new(slot));

    let outer_core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(90.0),
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

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&format!("docs/img/rot_core.svg"));

    let mut bb = outer_core.shape().bounding_box();
    bb.scale(1.01);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(400));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        outer_core.drawable().draw(cr)?;
        inner_core.drawable().draw(cr)?;

        return Ok(());
    })?;
    return Ok(());
}

fn plot_lin_and_rot_core_common_image() -> Result<(), Box<dyn std::error::Error>> {
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
    .try_into()
    .expect("valid slot");

    let air_gap = SlottedAirGap::new(12, false, CarterFactorModel::Bin12, Box::new(slot.clone()));

    let lin_core: LinCore = LinCoreBuilder {
        height: Length::new::<millimeter>(20.0),
        width: Length::new::<millimeter>(150.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 3,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    }
    .try_into()?;

    let air_gap = SlottedAirGap::new(15, false, CarterFactorModel::Bin12, Box::new(slot));
    let rot_core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(40.0),
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
    .try_into()
    .unwrap();

    let mut drawables: Vec<Drawable> = Vec::with_capacity(2);
    let lin_core_drawable: Drawable = lin_core.drawable().into();
    drawables.push(lin_core_drawable);

    let mut rot_core_drawable: Drawable = rot_core.drawable().into();
    rot_core_drawable.translate([
        (lin_core.width() + rot_core.air_gap_radius()).get::<meter>() * 1.1,
        0.5 * lin_core.height().get::<meter>(),
    ]);
    drawables.push(rot_core_drawable);

    let fp =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&format!("docs/img/lin_and_rot_core.svg"));

    let mut bb = BoundingBox::from_bounded_entities(drawables.as_slice().iter()).unwrap();
    bb.scale(1.01);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(400));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        for drawable in drawables {
            drawable.draw(cr)?;
        }

        return Ok(());
    })?;
    return Ok(());
}
