use std::{path::PathBuf, sync::Arc};

use cairo_viewport::{BoundingBox, SideLength, Viewport};
use planar_geo::Transformation;
use planar_geo::draw::*;
use stem_core::prelude::*;
use stem_slot::semi_trapezoid::SemiTrapezoidWidthsAndHeightsBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    plot_comparison()?;
    plot_plain()?;
    plot_slotted()?;
    plot_straight_indents()?;
    return Ok(());
}

fn plot_comparison() -> Result<(), Box<dyn std::error::Error>> {
    let distance = 0.01;

    let plain_air_gap: RotCore = RotCoreBuilder {
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
    .try_into()?;

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

    let air_gap = SlottedAirGap::new(15, false, CarterFactorModel::Bin12, Box::new(slot));
    let slotted_air_pap: RotCore = RotCoreBuilder {
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
    .try_into()?;

    let air_gap = StraightIndentsAirGap::new(
        1.try_into()?,
        Length::new::<millimeter>(10.0),
        Length::new::<millimeter>(2.0),
        2,
    );

    let straight_indents_air_pap: RotCore = RotCoreBuilder {
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
    .try_into()?;

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&format!("docs/img/rot_air_gap_comparison.svg"));

    let ag_radius = plain_air_gap.air_gap_radius().get::<meter>();
    let bb = BoundingBox::new(
        -(ag_radius + 0.001),
        5.0 * ag_radius + 2.0 * distance + 0.001,
        -(ag_radius + 0.001),
        ag_radius + 0.001,
    );

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        plain_air_gap.drawable().draw(cr)?;

        cr.translate(distance + 2.0 * ag_radius, 0.0);
        slotted_air_pap.drawable().draw(cr)?;

        cr.translate(distance + 2.0 * ag_radius, 0.0);
        straight_indents_air_pap.drawable().draw(cr)?;

        return Ok(());
    })?;
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
    .try_into()?;

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

    // =========================================================================
    // Plot with air gap winding and surface magnets

    let ag = PlainAirGap::new(0, Length::new::<millimeter>(3.0), 0.8, 12, true)?;
    let rot_core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(40.0),
        yoke_radius: Length::new::<millimeter>(19.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 3,
        skew_angle: 0.0,
        air_gap: Box::new(ag),
        flux_barrier: None,
    }
    .try_into()?;

    let magnet = ArcParallelMagnet::with_const_thickness(
        rot_core.axial_length(),
        rot_core.air_gap_radius(),
        SideHeightOrThickness::Thickness(Length::new::<millimeter>(4.0)),
        AngleOrWidth::Angle(0.3 * std::f64::consts::FRAC_PI_2),
        Arc::new(Material::default()),
    )?;
    let mag_assembly = MagnetAssembly::new(magnet, 1.try_into()?, 2.try_into()?);

    let ag_radius = rot_core.air_gap_radius().get::<meter>();
    let bb = BoundingBox::new(
        -(ag_radius + 0.005),
        3.0 * ag_radius + 2.0 * distance + 0.005,
        -(ag_radius + 0.005),
        ag_radius + 0.005,
    );

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&format!("docs/img/magnets_and_winding_plain.svg"));

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        rot_core.drawable().draw(cr)?;
        for w in rot_core.winding_zones(&CoilLayout::Single) {
            w.into_drawable().draw(cr)?;
        }

        cr.translate(distance + 2.0 * ag_radius, 0.0);
        rot_core.drawable().draw(cr)?;
        for m in rot_core.surface_magnets(&mag_assembly, true) {
            m.into_drawable().draw(cr)?;
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
        opening_width: Length::new::<millimeter>(0.0),
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

    let air_gap = SlottedAirGap::new(12, false, CarterFactorModel::Bin12, Box::new(slot));

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
    .try_into()?;

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

    // =========================================================================

    let distance = 0.01;

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

    let air_gap = SlottedAirGap::new(9, false, CarterFactorModel::Bin12, Box::new(slot.clone()));
    let lin_core_tooth_middle: LinCore = LinCoreBuilder {
        height: Length::new::<millimeter>(20.0),
        width: Length::new::<millimeter>(120.0),
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

    let air_gap: SlottedAirGap =
        SlottedAirGap::new(9, true, CarterFactorModel::Bin12, Box::new(slot.clone()));
    let lin_core_slot_middle: LinCore = LinCoreBuilder {
        height: Length::new::<millimeter>(20.0),
        width: Length::new::<millimeter>(120.0),
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

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&format!(
        "docs/img/lin_slotted_core_slot_vs_tooth_middle.svg"
    ));

    let width = lin_core_tooth_middle.width().get::<meter>();
    let xshift = width + distance;
    let height = lin_core_tooth_middle.height().get::<meter>();
    let delta = 0.001;
    let bb = BoundingBox::new(-delta, xshift + width + delta, -delta, 1.4 * height + delta);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        lin_core_tooth_middle.drawable().draw(cr)?;
        let text = Text {
            text: "starts_in_slot_middle = false".to_string(),
            anchor: Anchor::Center,
            fixed_anchor_offset: [0.0, 0.0],
            scaled_anchor_offset: [0.5 * width, 1.2 * height],
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            font_size: 12.0,
            angle: 0.0,
        };
        text.draw(cr)?;

        cr.translate(xshift, 0.0);
        lin_core_slot_middle.drawable().draw(cr)?;
        let text = Text {
            text: "starts_in_slot_middle = true".to_string(),
            anchor: Anchor::Center,
            fixed_anchor_offset: [0.0, 0.0],
            scaled_anchor_offset: [0.5 * width, 1.2 * height],
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            font_size: 12.0,
            angle: 0.0,
        };
        text.draw(cr)?;

        return Ok(());
    })?;

    return Ok(());
}

fn plot_straight_indents() -> Result<(), Box<dyn std::error::Error>> {
    let distance = 0.02;
    let magnet_thickness = 0.005;

    let air_gap = StraightIndentsAirGap::new(
        1.try_into()?,
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
    .try_into()?;

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
