use std::sync::Arc;

use cairo_viewport::{SideLength, Viewport, bounding_box::ToBoundingBox, compare_or_create};
use stem_core::{magnets::PositionedMagnetShape, prelude::*};
use stem_slot::planar_geo::{DEFAULT_EPSILON, DEFAULT_MAX_RELATIVE, draw::Drawable};

#[test]
fn test_radii_calc() {
    let air_gap_radius = Length::new::<millimeter>(60.0);
    {
        let air_gap = StraightIndentsAirGap::new(
            1.try_into().unwrap(),
            Length::new::<millimeter>(20.0),
            Length::new::<millimeter>(0.0),
            3.try_into().unwrap(),
        );

        approx::assert_abs_diff_eq!(
            air_gap
                .indent_corner_radius(air_gap_radius, false)
                .get::<meter>(),
            0.06
        );
        approx::assert_abs_diff_eq!(
            air_gap
                .indent_corner_radius(air_gap_radius, true)
                .get::<meter>(),
            0.06
        );

        approx::assert_abs_diff_eq!(
            air_gap
                .indent_center_radius(air_gap_radius, false)
                .get::<meter>(),
            0.05916,
            epsilon = 1e-4
        );
        approx::assert_abs_diff_eq!(
            air_gap
                .indent_center_radius(air_gap_radius, true)
                .get::<meter>(),
            0.05916,
            epsilon = 1e-4
        );

        approx::assert_abs_diff_eq!(
            (air_gap
                .indent_center_radius(air_gap_radius, false)
                .get::<meter>()
                .powi(2)
                + (0.5 * air_gap.indent_width().get::<meter>()).powi(2))
            .sqrt(),
            0.06,
            epsilon = 1e-4
        );
        approx::assert_abs_diff_eq!(
            (air_gap
                .indent_center_radius(air_gap_radius, true)
                .get::<meter>()
                .powi(2)
                + (0.5 * air_gap.indent_width().get::<meter>()).powi(2))
            .sqrt(),
            0.06,
            epsilon = 1e-4
        );
    }
    let air_gap_radius = Length::new::<millimeter>(60.0);
    {
        let air_gap = StraightIndentsAirGap::new(
            1.try_into().unwrap(),
            Length::new::<millimeter>(20.0),
            Length::new::<millimeter>(5.0),
            3.try_into().unwrap(),
        );

        approx::assert_abs_diff_eq!(
            air_gap
                .indent_corner_radius(air_gap_radius, false)
                .get::<meter>(),
            0.05507,
            epsilon = 1e-4
        );
        approx::assert_abs_diff_eq!(
            air_gap
                .indent_corner_radius(air_gap_radius, true)
                .get::<meter>(),
            0.06493,
            epsilon = 1e-4
        );

        approx::assert_abs_diff_eq!(
            air_gap
                .indent_center_radius(air_gap_radius, false)
                .get::<meter>(),
            0.05416,
            epsilon = 1e-4
        );
        approx::assert_abs_diff_eq!(
            air_gap
                .indent_center_radius(air_gap_radius, true)
                .get::<meter>(),
            0.06416,
            epsilon = 1e-4
        );
    }
    {
        let air_gap = StraightIndentsAirGap::new(
            1.try_into().unwrap(),
            Length::new::<millimeter>(20.0),
            Length::new::<millimeter>(-5.0),
            3.try_into().unwrap(),
        );

        approx::assert_abs_diff_eq!(
            air_gap
                .indent_corner_radius(air_gap_radius, false)
                .get::<meter>(),
            0.06493,
            epsilon = 1e-4
        );
        approx::assert_abs_diff_eq!(
            air_gap
                .indent_corner_radius(air_gap_radius, true)
                .get::<meter>(),
            0.05507,
            epsilon = 1e-4
        );

        approx::assert_abs_diff_eq!(
            air_gap
                .indent_center_radius(air_gap_radius, false)
                .get::<meter>(),
            0.06416,
            epsilon = 1e-4
        );
        approx::assert_abs_diff_eq!(
            air_gap
                .indent_center_radius(air_gap_radius, true)
                .get::<meter>(),
            0.05416,
            epsilon = 1e-4
        );
    }
}

fn create_core(
    indent_width: f64,
    indent_depth: f64,
    indents_per_pole: usize,
    outer: bool,
) -> RotCore {
    let (ag, yoke) = if outer { (60.0, 80.0) } else { (80.0, 60.0) };
    RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(ag),
        yoke_radius: Length::new::<millimeter>(yoke),
        axial_length: Length::new::<millimeter>(1.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        skew_angle: 0.0,
        air_gap: Box::new(StraightIndentsAirGap::new(
            1.try_into().unwrap(),
            Length::new::<millimeter>(indent_width),
            Length::new::<millimeter>(indent_depth),
            indents_per_pole.try_into().expect("must not be zero"),
        )),
        flux_barrier: None,
    }
    .try_into()
    .expect("valid magnetic core")
}

#[test]
fn test_assembly_check_block() {
    let air_gap = StraightIndentsAirGap::new(
        1.try_into().unwrap(),
        Length::new::<millimeter>(20.5),
        Length::new::<millimeter>(2.0),
        3,
    );
    let core: RotCore = RotCoreBuilder {
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
    .try_into()
    .expect("valid");
    let magnet = BlockMagnet::new(
        core.axial_length(),
        Length::new::<millimeter>(20.0),
        Length::new::<millimeter>(4.0),
        Length::new::<millimeter>(1.0),
        Arc::new(Material::default()),
    )
    .expect("valid");
    let assembly = MagnetAssembly::new(
        magnet,
        1.try_into().expect("valid"),
        3.try_into().expect("valid"),
    );

    assert!(
        core.assembly_check(
            &CoilLayout::SingleFilled,
            Some(&assembly),
            DEFAULT_EPSILON,
            DEFAULT_MAX_RELATIVE
        )
        .is_ok()
    );
}

#[test]
fn test_assembly_check_inner() {
    let magnet = BreadLoafMagnet::new(
        Length::new::<millimeter>(165.0),
        Length::new::<millimeter>(18.0),
        Length::new::<millimeter>(5.0),
        Length::new::<millimeter>(15.0),
        Arc::new(Default::default()),
    )
    .unwrap();
    let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());

    {
        let core = create_core(20.0, 0.0, 3, false);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
    {
        let core = create_core(20.0, -2.0, 3, false);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
    {
        let core = create_core(20.0, 2.0, 3, false);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
    {
        let core = create_core(20.0, -5.0, 3, false);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
    {
        let core = create_core(20.0, 5.0, 3, false);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
    {
        let core = create_core(17.0, 2.0, 3, false);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_err()
        );
    }
}

#[test]
fn test_assembly_check_outer() {
    let magnet = BreadLoafMagnet::new(
        Length::new::<millimeter>(165.0),
        Length::new::<millimeter>(18.0),
        Length::new::<millimeter>(5.0),
        Length::new::<millimeter>(15.0),
        Arc::new(Default::default()),
    )
    .unwrap();
    let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());

    {
        let core = create_core(20.0, 0.0, 3, true);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
    {
        let core = create_core(20.0, -2.0, 3, true);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
    {
        let core = create_core(20.0, 2.0, 3, true);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
    {
        let core = create_core(20.0, -5.0, 3, true);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
    {
        let core = create_core(20.0, 5.0, 3, true);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
}

#[test]
fn test_plot_inner_assembly() {
    for (idx, indent_depth) in [0.0, 2.0, -2.0, 5.0, -5.0].into_iter().enumerate() {
        let magnet = BreadLoafMagnet::new(
            Length::new::<millimeter>(165.0),
            Length::new::<millimeter>(18.0),
            Length::new::<millimeter>(5.0),
            Length::new::<millimeter>(15.0),
            Arc::new(Default::default()),
        )
        .unwrap();
        let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());

        let core = create_core(20.0, indent_depth, 3, false);
        let drawable: Drawable = core.drawable().into();

        let magnets: Vec<Drawable> = core
            .surface_magnets(&assembly, true)
            .map(PositionedMagnetShape::into_drawable)
            .collect();
        let mag_ref = &magnets;

        let view = Viewport::from_bounded_entities(
            mag_ref
                .iter()
                .map(|m| m.bounding_box())
                .chain(std::iter::once(drawable.bounding_box())),
            SideLength::Long(500),
        )
        .unwrap();

        let string = format!(
            "tests/img/rot_straight_indents/assembly_inner_{}.png",
            idx + 1
        );
        let path = std::path::Path::new(&string);
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in mag_ref {
                    m.draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
}

#[test]
fn test_plot_outer_assembly() {
    for (idx, indent_depth) in [0.0, 2.0, -2.0, 5.0, -5.0].into_iter().enumerate() {
        let magnet = BreadLoafMagnet::new(
            Length::new::<millimeter>(165.0),
            Length::new::<millimeter>(18.0),
            Length::new::<millimeter>(5.0),
            Length::new::<millimeter>(15.0),
            Arc::new(Default::default()),
        )
        .unwrap();
        let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());

        let core = create_core(20.0, indent_depth, 3, true);
        let drawable: Drawable = core.drawable().into();

        let magnets: Vec<Drawable> = core
            .surface_magnets(&assembly, true)
            .map(PositionedMagnetShape::into_drawable)
            .collect();
        let mag_ref = &magnets;

        let view = Viewport::from_bounded_entities(
            mag_ref
                .iter()
                .map(|m| m.bounding_box())
                .chain(std::iter::once(drawable.bounding_box())),
            SideLength::Long(500),
        )
        .unwrap();

        let string = format!(
            "tests/img/rot_straight_indents/assembly_outer_{}.png",
            idx + 1
        );
        let path = std::path::Path::new(&string);
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in mag_ref {
                    m.draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
}

#[test]
fn test_plot_outer_core() {
    {
        let drawable: Drawable = create_core(20.0, 0.0, 3, true).drawable().into();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/rot_straight_indents/outer_1.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let drawable: Drawable = create_core(30.0, 0.0, 2, true).drawable().into();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/rot_straight_indents/outer_2.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let drawable: Drawable = create_core(30.0, 2.0, 2, true).drawable().into();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/rot_straight_indents/outer_3.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let drawable: Drawable = create_core(30.0, -2.0, 2, true).drawable().into();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/rot_straight_indents/outer_4.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let drawable: Drawable = create_core(20.0, -2.0, 3, true).drawable().into();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/rot_straight_indents/outer_5.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
}

#[test]
fn test_plot_inner_core() {
    {
        let drawable: Drawable = create_core(20.0, 0.0, 3, false).drawable().into();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/rot_straight_indents/inner_1.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let drawable: Drawable = create_core(30.0, 0.0, 2, false).drawable().into();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/rot_straight_indents/inner_2.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let drawable: Drawable = create_core(30.0, 2.0, 2, false).drawable().into();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/rot_straight_indents/inner_3.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let drawable: Drawable = create_core(30.0, -2.0, 2, false).drawable().into();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/rot_straight_indents/inner_4.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let drawable: Drawable = create_core(20.0, -2.0, 3, false).drawable().into();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/rot_straight_indents/inner_5.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
}
