use std::{path::PathBuf, sync::Arc};

use cairo_viewport::{BoundingBox, SideLength, Viewport};
use planar_geo::Transformation;
use planar_geo::draw::*;
use stem_core::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_fb_lin_core()?;
    set_fb_rot_core()?;
    return Ok(());
}

fn set_fb_lin_core() -> Result<(), Box<dyn std::error::Error>> {
    let air_gap = PlainAirGap::default();
    let lin_core: LinCore = LinCoreBuilder {
        height: Length::new::<millimeter>(20.0),
        width: Length::new::<millimeter>(100.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        air_gap: Box::new(air_gap),
        flux_barrier: None, // No flux barrier at initialization
    }
    .try_into()?;

    // Define a compatible flux barrier
    let mut fb_comp = Spoke1FluxBarrier {
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

    // Define an incompatible flux barrier
    let mut fb_incomp = Spoke1FluxBarrier {
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(4.0),
        magnet_space_width: Length::new::<millimeter>(30.0), // Too wide for the core width
        magnet_space_height_or_relief_path_width: Spoke1HeightSplit::ReliefPathWidth(
            Length::new::<millimeter>(2.0),
        ),
        glue_gap: Length::new::<millimeter>(0.0),
        magnet_material: None,
        cache: None,
    };

    // Create contours
    let contours_comp = fb_comp.combine(lin_core.as_core_ref())?;
    let drawable_comp: Drawable = lin_core.drawable().into();
    let mut contours_incomp = fb_incomp.combine(lin_core.as_core_ref())?;
    let mut drawable_incomp = drawable_comp.clone();

    // Shift the plot to the right
    let x_shift = 1.1 * lin_core.width().get::<meter>();
    drawable_incomp.translate([x_shift, 0.0]);
    contours_incomp
        .iter_mut()
        .for_each(|c| c.translate([x_shift, 0.0]));

    let style_comp = Style {
        line_color: Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
        background_color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        },
        line_width: 1.0,
        line_style: LineStyle::Solid,
        line_cap: LineCap::Butt,
        line_join: LineJoin::Bevel,
        text: None,
    };
    let mut style_incomp = style_comp.clone();
    style_incomp.line_color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    let contours_comp_ref = contours_comp.as_slice();
    let contours_incomp_ref = contours_incomp.as_slice();
    let fp =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&format!("docs/img/lin_core_set_fb.svg"));

    let bb = BoundingBox::from_bounded_entities(contours_comp_ref.iter()).unwrap();
    let bb = bb.union(&BoundingBox::from_bounded_entities(contours_incomp_ref.iter()).unwrap());
    let bb = bb.union(&BoundingBox::from(&drawable_comp));
    let mut bb = bb.union(&BoundingBox::from(&drawable_incomp));
    bb.scale(1.01);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(400));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;
        cr.scale(1.0, -1.0);
        cr.translate(0.0, -bb.height());

        drawable_comp.draw(&cr)?;
        drawable_incomp.draw(&cr)?;

        for c in contours_comp_ref {
            c.draw(&style_comp, cr)?;
        }

        for c in contours_incomp_ref {
            c.draw(&style_incomp, cr)?;
        }

        return Ok(());
    })?;

    return Ok(());
}

fn set_fb_rot_core() -> Result<(), Box<dyn std::error::Error>> {
    let air_gap = PlainAirGap::default();
    let rot_core: RotCore = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(40.0),
        yoke_radius: Length::new::<millimeter>(15.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 3,
        air_gap: Box::new(air_gap),
        flux_barrier: None, // No flux barrier at initialization
    }
    .try_into()?;

    // Define a compatible flux barrier
    let mut fb_comp = Spoke1FluxBarrier {
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

    // Define an incompatible flux barrier
    let mut fb_incomp = Spoke1FluxBarrier {
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(4.0),
        magnet_space_width: Length::new::<millimeter>(30.0), // Too wide for the core width
        magnet_space_height_or_relief_path_width: Spoke1HeightSplit::ReliefPathWidth(
            Length::new::<millimeter>(2.0),
        ),
        glue_gap: Length::new::<millimeter>(0.0),
        magnet_material: None,
        cache: None,
    };

    // Create contours
    let contours_comp = fb_comp.combine(rot_core.as_core_ref())?;
    let drawable_comp: Drawable = rot_core.drawable().into();
    let mut contours_incomp = fb_incomp.combine(rot_core.as_core_ref())?;
    let mut drawable_incomp = drawable_comp.clone();

    // Shift the plot to the right
    let x_shift = 1.1 * 2.0 * rot_core.air_gap_radius().get::<meter>();
    drawable_incomp.translate([x_shift, 0.0]);
    contours_incomp
        .iter_mut()
        .for_each(|c| c.translate([x_shift, 0.0]));

    let style_comp = Style {
        line_color: Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
        background_color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        },
        line_width: 1.0,
        line_style: LineStyle::Solid,
        line_cap: LineCap::Butt,
        line_join: LineJoin::Bevel,
        text: None,
    };
    let mut style_incomp = style_comp.clone();
    style_incomp.line_color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    let contours_comp_ref = contours_comp.as_slice();
    let contours_incomp_ref = contours_incomp.as_slice();
    let fp =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&format!("docs/img/rot_core_set_fb.svg"));

    let bb = BoundingBox::from_bounded_entities(contours_comp_ref.iter()).unwrap();
    let bb = bb.union(&BoundingBox::from_bounded_entities(contours_incomp_ref.iter()).unwrap());
    let bb = bb.union(&BoundingBox::from(&drawable_comp));
    let mut bb = bb.union(&BoundingBox::from(&drawable_incomp));
    bb.scale(1.01);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(400));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        drawable_comp.draw(&cr)?;
        drawable_incomp.draw(&cr)?;

        for c in contours_comp_ref {
            c.draw(&style_comp, cr)?;
        }

        for c in contours_incomp_ref {
            c.draw(&style_incomp, cr)?;
        }

        return Ok(());
    })?;

    return Ok(());
}
