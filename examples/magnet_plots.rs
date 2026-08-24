use std::f64::consts::FRAC_PI_2;
use std::{path::PathBuf, sync::Arc};

use cairo_viewport::{BoundingBox, SideLength, Viewport};
use planar_geo::Transformation;
use planar_geo::draw::*;
use stem_core::prelude::*;
use stem_slot::planar_geo::prelude::Composite;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    surface_magnets_plot()?;
    surface_and_interior_magnets_plot()?;
    return Ok(());
}

fn surface_and_interior_magnets_plot() -> Result<(), Box<dyn std::error::Error>> {
    let air_gap = PlainAirGap {
        num_segments: 0,
        air_gap_winding_height: Length::new::<millimeter>(4.0),
        winding_coverage: 0.8,
        starts_in_slot_middle: false,
        slots: 24,
    };

    let fb = V1rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(5.0),
        relief_path_air_gap_width: Length::new::<millimeter>(4.0),
        relief_path_length: Length::new::<millimeter>(0.0),
        relief_path_width: Length::new::<millimeter>(2.0),
        opening_angle: FRAC_PI_2,
        magnet_space_width: Length::new::<millimeter>(5.0),
        magnet_space_height: Length::new::<millimeter>(30.0),
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
    let rot_surface_magnets = MagnetAssembly::new(magnet, 1.try_into()?, 2.try_into()?);

    let fp = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&format!("docs/img/surface_and_interior_magnets.svg"));

    let bb = BoundingBox::new(-60.0e-3, 60.0e-3, -60.0e-3, 60.0e-3);

    let roman_letters = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII"];

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(600));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        rot_core.drawable().draw(cr)?;
        for (idx, mut m) in rot_core
            .surface_magnets(&rot_surface_magnets, false)
            .enumerate()
        {
            m.line_reflection([0.0, 0.0], [1.0, 0.0]);
            let text = Text {
                text: idx.to_string(),
                anchor: Anchor::Center,
                fixed_anchor_offset: [0.0, 0.0],
                scaled_anchor_offset: m.shape.centroid(),
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                font_size: 16.0,
                angle: 0.0,
            };
            m.into_drawable().draw(cr)?;
            text.draw(cr)?;
        }

        for (idx, mut m) in rot_core.interior_magnets(false).enumerate() {
            m.line_reflection([0.0, 0.0], [1.0, 0.0]);
            let text = Text {
                text: roman_letters[idx].to_string(),
                anchor: Anchor::Center,
                fixed_anchor_offset: [0.0, 0.0],
                scaled_anchor_offset: m.shape.centroid(),
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                font_size: 16.0,
                angle: 0.0,
            };
            m.into_drawable().draw(cr)?;
            text.draw(cr)?;
        }

        return Ok(());
    })?;

    return Ok(());
}

fn surface_magnets_plot() -> Result<(), Box<dyn std::error::Error>> {
    let air_gap = PlainAirGap {
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
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    }
    .try_into()?;
    let magnet = ArcParallelMagnet::with_const_thickness(
        core_plain.axial_length(),
        core_plain.air_gap_radius(),
        SideHeightOrThickness::Thickness(Length::new::<millimeter>(4.0)),
        AngleOrWidth::Angle(0.7 * FRAC_PI_2 / 2.0),
        Arc::new(Material::default()),
    )?;
    let surface_magnets_plain = MagnetAssembly::new(magnet, 1.try_into()?, 2.try_into()?);

    let air_gap = StraightIndentsAirGap {
        num_segments: 1.try_into()?,
        indent_width: Length::new::<millimeter>(20.5),
        indent_depth: Length::new::<millimeter>(2.0),
        indents_per_pole: 3,
    };

    let core_straight_indents: RotCore = RotCoreBuilder {
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
    let magnet = BlockMagnet::new(
        core_straight_indents.axial_length(),
        Length::new::<millimeter>(20.0),
        Length::new::<millimeter>(4.0),
        Length::new::<millimeter>(1.0),
        Arc::new(Material::default()),
    )?;
    let surface_magnets_straight_indents =
        MagnetAssembly::new(magnet, 1.try_into()?, 3.try_into()?);

    let shift = 0.12;

    let fp =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&format!("docs/img/surface_magnets.svg"));

    let bb = BoundingBox::new(-60.0e-3, shift + 60.0e-3, -60.0e-3, 60.0e-3);

    let view = Viewport::from_bounding_box(&bb, SideLength::Long(800));
    view.write_to_file(&fp, |cr| {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.paint()?;

        core_plain.drawable().draw(cr)?;
        for (idx, mut m) in core_plain
            .surface_magnets(&surface_magnets_plain, false)
            .enumerate()
        {
            m.line_reflection([0.0, 0.0], [1.0, 0.0]);
            let text = Text {
                text: idx.to_string(),
                anchor: Anchor::Center,
                fixed_anchor_offset: [0.0, 0.0],
                scaled_anchor_offset: m.shape.centroid(),
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                font_size: 12.0,
                angle: 0.0,
            };
            m.into_drawable().draw(cr)?;
            text.draw(cr)?;
        }

        cr.translate(shift, 0.0);

        core_straight_indents.drawable().draw(cr)?;
        for (idx, mut m) in core_straight_indents
            .surface_magnets(&surface_magnets_straight_indents, false)
            .enumerate()
        {
            m.line_reflection([0.0, 0.0], [1.0, 0.0]);
            let text = Text {
                text: idx.to_string(),
                anchor: Anchor::Center,
                fixed_anchor_offset: [0.0, 0.0],
                scaled_anchor_offset: m.shape.centroid(),
                color: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                font_size: 12.0,
                angle: 0.0,
            };
            m.into_drawable().draw(cr)?;
            text.draw(cr)?;
        }

        return Ok(());
    })?;

    return Ok(());
}
