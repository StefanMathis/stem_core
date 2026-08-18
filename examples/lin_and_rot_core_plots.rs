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
    plot_lin_core_air_gap_and_slotted_winding()?;
    plot_lin_and_rot_core_surface_magnets()?;
    plot_lin_and_rot_core_interior_magnets()?;
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
    .try_into()?;

    let air_gap = SlottedAirGap::new(12, false, CarterFactorModel::Bin12, Box::new(slot.clone()));
    let flux_barrier = Spoke1FluxBarrier {
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(4.0),
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height_or_relief_path_width: Spoke1HeightSplit::ReliefPathWidth(
            Length::new::<millimeter>(2.0),
        ),
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
    let air_gap_plain = PlainAirGap::default();

    let flux_barrier = Spoke1FluxBarrier {
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(4.0),
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height_or_relief_path_width: Spoke1HeightSplit::ReliefPathWidth(
            Length::new::<millimeter>(2.0),
        ),
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
    .try_into()?;

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
    .try_into()?;

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
    .try_into()?;

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

fn plot_lin_core_air_gap_and_slotted_winding() -> Result<(), Box<dyn std::error::Error>> {
    let width = 0.15;
    let height = 0.018;
    let air_gap_height = 0.005;

    let plain_air_gap = PlainAirGap {
        num_segments: 0,
        air_gap_winding_height: Length::new::<meter>(air_gap_height),
        winding_coverage: 0.8,
        starts_in_slot_middle: false,
        slots: 12,
    };

    let plain_lin_core: LinCore = LinCoreBuilder {
        height: Length::new::<meter>(height),
        width: Length::new::<meter>(width),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        air_gap: Box::new(plain_air_gap),
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
    .try_into()?;

    let air_gap = SlottedAirGap::new(12, false, CarterFactorModel::Bin12, Box::new(slot.clone()));

    let slotted_lin_core: LinCore = LinCoreBuilder {
        height: Length::new::<meter>(height),
        width: Length::new::<meter>(width),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    }
    .try_into()?;

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&format!(
        "docs/img/lin_core_air_gap_and_slotted_winding.svg"
    ));

    let shift = width + 0.01;

    let bb = BoundingBox::new(0.0, shift + width, -air_gap_height, height);
    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        plain_lin_core.drawable().draw(cr)?;
        for c in plain_lin_core.winding_zones(&CoilLayout::DoubleVertical) {
            let drawable: Drawable = c.into_drawable();
            drawable.draw(cr)?;
        }

        let mut slotted: Drawable = slotted_lin_core.drawable().into();
        slotted.translate([shift, 0.0]);
        slotted.draw(cr)?;
        for c in slotted_lin_core.winding_zones(&CoilLayout::DoubleVertical) {
            let mut drawable: Drawable = c.into_drawable();
            drawable.translate([shift, 0.0]);
            drawable.draw(cr)?;
        }

        return Ok(());
    })?;
    return Ok(());
}

fn plot_lin_and_rot_core_surface_magnets() -> Result<(), Box<dyn std::error::Error>> {
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

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&format!("docs/img/lin_and_rot_core_surface_magnets.svg"));

    let ag_radius = rot_core.air_gap_radius().get::<meter>();
    let width = lin_core.width().get::<meter>();
    let xshift = width + distance + ag_radius;
    let yshift = 0.5 * lin_core.height().get::<meter>();
    let shift = [xshift, yshift];
    let bb = BoundingBox::new(
        0.0,
        xshift + magnet_thickness + ag_radius,
        -ag_radius - magnet_thickness + yshift,
        ag_radius + magnet_thickness + yshift,
    );

    let lin_core_magnet = MagnetAssembly::new(
        BreadLoafMagnet::with_center_thickness(
            lin_core.axial_length(),
            Length::new::<millimeter>(10.0),
            Length::new::<meter>(0.75 * magnet_thickness),
            Length::new::<meter>(magnet_thickness),
            Arc::new(Material::default()),
        )?,
        1.try_into()?,
        2.try_into()?,
    );

    let rot_core_magnet = MagnetAssembly::new(
        ArcParallelMagnet::with_const_thickness(
            lin_core.axial_length(),
            rot_core.air_gap_radius(),
            SideHeightOrThickness::Thickness(Length::new::<meter>(magnet_thickness)),
            0.4.into(),
            Arc::new(Material::default()),
        )?,
        1.try_into()?,
        2.try_into()?,
    );

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        lin_core.drawable().draw(cr)?;
        for c in lin_core.surface_magnets(&lin_core_magnet, true) {
            let drawable: Drawable = c.into_drawable();
            drawable.draw(cr)?;
        }

        let mut drawable: Drawable = rot_core.drawable().into();
        drawable.translate(shift);
        drawable.draw(cr)?;
        for c in rot_core.surface_magnets(&rot_core_magnet, true) {
            let mut drawable: Drawable = c.into_drawable();
            drawable.translate(shift);
            drawable.draw(cr)?;
        }
        return Ok(());
    })?;
    return Ok(());
}

fn plot_lin_and_rot_core_interior_magnets() -> Result<(), Box<dyn std::error::Error>> {
    let distance = 0.02;
    let magnet_thickness = 0.005;

    let fb = Spoke1FluxBarrier {
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(0.0),
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height_or_relief_path_width: Spoke1HeightSplit::ReliefPathWidth(
            Length::new::<millimeter>(0.0),
        ),
        glue_gap: Length::new::<millimeter>(0.5),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
    };

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
        flux_barrier: Some(Box::new(fb.clone())),
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
        flux_barrier: Some(Box::new(fb.clone())),
    }
    .try_into()?;

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&format!("docs/img/lin_and_rot_core_interior_magnets.svg"));

    let ag_radius = rot_core.air_gap_radius().get::<meter>();
    let width = lin_core.width().get::<meter>();
    let xshift = width + distance + ag_radius;
    let yshift = 0.5 * lin_core.height().get::<meter>();
    let shift = [xshift, yshift];
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

        lin_core.drawable().draw(cr)?;
        for c in lin_core.interior_magnets(true) {
            let drawable: Drawable = c.into_drawable();
            drawable.draw(cr)?;
        }

        let mut drawable: Drawable = rot_core.drawable().into();
        drawable.translate(shift);
        drawable.draw(cr)?;
        for c in rot_core.interior_magnets(true) {
            let mut drawable: Drawable = c.into_drawable();
            drawable.translate(shift);
            drawable.draw(cr)?;
        }
        return Ok(());
    })?;
    return Ok(());
}
