use std::f64::consts::FRAC_PI_2;
use std::{path::PathBuf, sync::Arc};

use cairo_viewport::{BoundingBox, SideLength, Viewport};
use stem_core::prelude::*;
use stem_slot::semi_trapezoid::SemiTrapezoidWidthsAndHeightsBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    plot_flux_barrier_comparison()?;
    plot_spoke1_flux_barrier()?;
    plot_slotted_with_and_without_flux_barrier()?;
    plot_v1r_flux_barrier()?;
    plot_v2r_flux_barrier()?;
    return Ok(());
}

fn plot_flux_barrier_comparison() -> Result<(), Box<dyn std::error::Error>> {
    let distance = 0.01;

    let spoke1_core = create_spoke1_flux_barrier()?;
    let v1r_core = create_v1r_flux_barrier()?;
    let v2r_core = create_v2r_flux_barrier()?;

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&format!("docs/img/rot_flux_barrier_comparison.svg"));

    let ag_radius = spoke1_core.air_gap_radius().get::<meter>();
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

        spoke1_core.drawable().draw(cr)?;

        cr.translate(distance + 2.0 * ag_radius, 0.0);
        v1r_core.drawable().draw(cr)?;

        cr.translate(distance + 2.0 * ag_radius, 0.0);
        v2r_core.drawable().draw(cr)?;

        return Ok(());
    })?;
    return Ok(());
}

fn plot_spoke1_flux_barrier() -> Result<(), Box<dyn std::error::Error>> {
    let distance = 0.03;

    let fb = Spoke1FluxBarrier {
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(3.0),
        magnet_space_width: Length::new::<millimeter>(10.0),
        height_split: Spoke1HeightSplit::ReliefPathWidth(Length::new::<millimeter>(1.0)),
        glue_gap: Length::new::<millimeter>(0.2),
        magnet_material: Some(Default::default()),
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
        flux_barrier: Some(Box::new(fb)),
    }
    .try_into()?;

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&format!("docs/img/lin_and_rot_core_spoke1.svg"));

    let ag_radius = rot_core.air_gap_radius().get::<meter>();
    let width = lin_core.width().get::<meter>();
    let xshift = width + distance + ag_radius;
    let yshift = 0.5 * lin_core.height().get::<meter>();
    let bb = BoundingBox::new(
        0.0,
        xshift + ag_radius,
        -ag_radius + yshift,
        ag_radius + yshift,
    );

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        lin_core.drawable().draw(cr)?;
        for m in lin_core.interior_magnets(true) {
            m.into_drawable().draw(cr)?;
        }

        cr.translate(width + distance + ag_radius, yshift);

        rot_core.drawable().draw(cr)?;
        for m in rot_core.interior_magnets(true) {
            m.into_drawable().draw(cr)?;
        }

        return Ok(());
    })?;
    return Ok(());
}

fn plot_v1r_flux_barrier() -> Result<(), Box<dyn std::error::Error>> {
    let rot_core = create_v1r_flux_barrier()?;

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&format!("docs/img/rot_core_v1r.svg"));

    let ag_radius = rot_core.air_gap_radius().get::<meter>();
    let xshift = ag_radius;
    let bb = BoundingBox::new(0.0, xshift + ag_radius, -ag_radius, ag_radius);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(400));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        cr.translate(ag_radius, 0.0);

        rot_core.drawable().draw(cr)?;
        for m in rot_core.interior_magnets(true) {
            m.into_drawable().draw(cr)?;
        }

        return Ok(());
    })?;
    return Ok(());
}

fn plot_v2r_flux_barrier() -> Result<(), Box<dyn std::error::Error>> {
    let rot_core = create_v2r_flux_barrier()?;

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&format!("docs/img/rot_core_v2r.svg"));

    let ag_radius = rot_core.air_gap_radius().get::<meter>();
    let xshift = ag_radius;
    let bb = BoundingBox::new(0.0, xshift + ag_radius, -ag_radius, ag_radius);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(400));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        cr.translate(ag_radius, 0.0);

        rot_core.drawable().draw(cr)?;
        for m in rot_core.interior_magnets(true) {
            m.into_drawable().draw(cr)?;
        }

        return Ok(());
    })?;
    return Ok(());
}

fn plot_slotted_with_and_without_flux_barrier() -> Result<(), Box<dyn std::error::Error>> {
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

fn create_spoke1_flux_barrier() -> Result<RotCore, Box<dyn std::error::Error>> {
    let fb = Spoke1FluxBarrier {
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(3.0),
        magnet_space_width: Length::new::<millimeter>(10.0),
        height_split: Spoke1HeightSplit::ReliefPathWidth(Length::new::<millimeter>(1.0)),
        glue_gap: Length::new::<millimeter>(0.2),
        magnet_material: Some(Default::default()),
        cache: None,
    };

    RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(40.0),
        yoke_radius: Length::new::<millimeter>(19.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 3,
        skew_angle: 0.0,
        air_gap: Box::new(PlainAirGap::default()),
        flux_barrier: Some(Box::new(fb)),
    }
    .try_into()
    .map_err(From::from)
}

fn create_v1r_flux_barrier() -> Result<RotCore, Box<dyn std::error::Error>> {
    let fb = V1rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(3.0),
        relief_path_air_gap_width: Length::new::<millimeter>(2.0),
        relief_path_length: Length::new::<millimeter>(4.0),
        relief_path_width: Length::new::<millimeter>(2.0),
        opening_angle: FRAC_PI_2,
        magnet_space_width: Length::new::<millimeter>(6.0),
        magnet_space_height: Length::new::<millimeter>(13.0),
        glue_gap: Length::new::<millimeter>(0.2),
        leakage_path_width: Length::new::<millimeter>(1.0),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
    };

    RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(40.0),
        yoke_radius: Length::new::<millimeter>(19.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 3,
        skew_angle: 0.0,
        air_gap: Box::new(PlainAirGap::default()),
        flux_barrier: Some(Box::new(fb)),
    }
    .try_into()
    .map_err(From::from)
}

fn create_v2r_flux_barrier() -> Result<RotCore, Box<dyn std::error::Error>> {
    let fb = V2rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(3.0),
        relief_path_air_gap_width: Length::new::<millimeter>(2.0),
        relief_path_length: Length::new::<millimeter>(4.0),
        relief_path_width: Length::new::<millimeter>(2.0),
        opening_angle: FRAC_PI_2,
        magnet_space_width: Length::new::<millimeter>(6.0),
        magnet_space_height: Length::new::<millimeter>(13.0),
        glue_gap: Length::new::<millimeter>(0.2),
        leakage_path_width: Length::new::<millimeter>(1.0),
        magnet_material: Some(Arc::new(Material::default())),
        q_axis_fillet: None,
        cache: None,
    };

    RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(40.0),
        yoke_radius: Length::new::<millimeter>(19.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 3,
        skew_angle: 0.0,
        air_gap: Box::new(PlainAirGap::default()),
        flux_barrier: Some(Box::new(fb)),
    }
    .try_into()
    .map_err(From::from)
}
