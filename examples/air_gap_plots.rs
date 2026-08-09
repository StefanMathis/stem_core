use std::{path::PathBuf, sync::Arc};

use cairo_viewport::{BoundingBox, SideLength, Viewport};
use planar_geo::Transformation;
use planar_geo::draw::*;
use stem_core::prelude::*;
use stem_slot::semi_trapezoid::SemiTrapezoidWidthsAndHeightsBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    plot_plain()?;
    plot_slotted()?;
    plot_straight_indents()?;
    return Ok(());
}

fn plot_plain() -> Result<(), Box<dyn std::error::Error>> {
    let distance = 0.02;
    let magnet_thickness = 0.005;

    let lin_core: LinCore = LinCoreBuilder {
        height: Length::new::<millimeter>(20.0),
        width: Length::new::<millimeter>(150.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 3,
        air_gap: Box::new(PlainAirGap::default()),
        flux_barrier: None,
    }
    .try_into()?;

    let rot_core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(40.0),
        yoke_radius: Length::new::<millimeter>(19.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 3,
        skew_angle: 0.0,
        air_gap: Box::new(PlainAirGap::default()),
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

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&format!("docs/img/lin_and_rot_core_plain.svg"));

    let ag_radius = rot_core.air_gap_radius().get::<meter>();
    let width = lin_core.width().get::<meter>();
    let xshift = width + distance + ag_radius;
    let yshift = 0.5 * lin_core.height().get::<meter>();
    let bb = BoundingBox::new(
        0.0,
        xshift + magnet_thickness + ag_radius,
        -ag_radius + yshift,
        ag_radius + yshift,
    );

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
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

fn plot_slotted() -> Result<(), Box<dyn std::error::Error>> {
    let distance = 0.02;
    let magnet_thickness = 0.005;

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
        pole_pairs: 3,
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

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&format!("docs/img/lin_and_rot_core_slotted.svg"));

    let ag_radius = rot_core.air_gap_radius().get::<meter>();
    let width = lin_core.width().get::<meter>();
    let xshift = width + distance + ag_radius;
    let yshift = 0.5 * lin_core.height().get::<meter>();
    let bb = BoundingBox::new(
        0.0,
        xshift + magnet_thickness + ag_radius,
        -ag_radius + yshift,
        ag_radius + yshift,
    );

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
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

fn plot_straight_indents() -> Result<(), Box<dyn std::error::Error>> {
    let distance = 0.02;
    let magnet_thickness = 0.005;

    let air_gap = StraightIndentsAirGap::new(
        1.try_into().unwrap(),
        Length::new::<millimeter>(10.0),
        Length::new::<millimeter>(2.0),
        2,
    );

    let lin_core: LinCore = LinCoreBuilder {
        height: Length::new::<millimeter>(20.0),
        width: Length::new::<millimeter>(150.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 3,
        air_gap: Box::new(air_gap.clone()),
        flux_barrier: None,
    }
    .try_into()?;

    let rot_core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(40.0),
        yoke_radius: Length::new::<millimeter>(19.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 3,
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

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&format!("docs/img/lin_and_rot_core_straight_indents.svg"));

    let ag_radius = rot_core.air_gap_radius().get::<meter>();
    let width = lin_core.width().get::<meter>();
    let xshift = width + distance + ag_radius;
    let yshift = 0.5 * lin_core.height().get::<meter>();
    let bb = BoundingBox::new(
        0.0,
        xshift + magnet_thickness + ag_radius,
        -ag_radius + yshift,
        ag_radius + yshift,
    );

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
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
