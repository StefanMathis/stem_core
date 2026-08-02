use std::sync::Arc;

use cairo_viewport::{SideLength, Viewport, bounding_box::ToBoundingBox, compare_or_create};
use stem_core::magnets::PositionedMagnetShape;
use stem_core::prelude::*;
use stem_slot::planar_geo::draw::Drawable;
use stem_slot::planar_geo::prelude::*;

fn create_core(indent_width: Length, indent_depth: Length, indents_per_pole: usize) -> LinCore {
    let air_gap = StraightIndentsAirGap::new(
        1,
        indent_width,
        indent_depth,
        indents_per_pole.try_into().unwrap(),
    );
    return LinCoreBuilder {
        height: Length::new::<millimeter>(25.0),
        width: Length::new::<millimeter>(150.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    }
    .try_into()
    .unwrap();
}
#[test]
fn test_collision_check() {
    let magnet = BreadLoafMagnet::new(
        Length::new::<millimeter>(165.0),
        Length::new::<millimeter>(8.0),
        Length::new::<millimeter>(5.0),
        Length::new::<millimeter>(15.0),
        Arc::new(Default::default()),
    )
    .unwrap();
    let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());

    {
        let core = create_core(
            Length::new::<millimeter>(10.0),
            Length::new::<millimeter>(0.0),
            3,
        );
        assert!(
            core.collision_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
    {
        let core = create_core(
            Length::new::<millimeter>(10.0),
            Length::new::<millimeter>(5.0),
            3,
        );
        assert!(
            core.collision_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
    {
        let core = create_core(
            Length::new::<millimeter>(10.0),
            Length::new::<millimeter>(-5.0),
            3,
        );
        assert!(
            core.collision_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );
    }
    {
        let core = create_core(
            Length::new::<millimeter>(7.0), // Indent too small
            Length::new::<millimeter>(-5.0),
            3,
        );
        assert!(
            core.collision_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_err()
        );
    }
    {
        // Overlapping magnet
        let magnet = BreadLoafMagnet::new(
            Length::new::<millimeter>(165.0),
            Length::new::<millimeter>(20.0),
            Length::new::<millimeter>(5.0),
            Length::new::<millimeter>(15.0),
            Arc::new(Default::default()),
        )
        .unwrap();
        let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());
        let core = create_core(
            Length::new::<millimeter>(7.0),
            Length::new::<millimeter>(-5.0),
            3,
        );
        assert!(
            core.collision_check(
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
fn test_plot_core() {
    {
        let core = create_core(
            Length::new::<millimeter>(10.0),
            Length::new::<millimeter>(0.0),
            3,
        );

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/lin_straight_indents/3_indents_no_depth.png");
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
        let core = create_core(
            Length::new::<millimeter>(10.0),
            Length::new::<millimeter>(5.0),
            3,
        );

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path =
            std::path::Path::new("tests/img/lin_straight_indents/3_indents_positive_depth.png");
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
        let core = create_core(
            Length::new::<millimeter>(10.0),
            Length::new::<millimeter>(-5.0),
            3,
        );

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path =
            std::path::Path::new("tests/img/lin_straight_indents/3_indents_negative_depth.png");
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
fn test_plot_assembly() {
    let magnet = BreadLoafMagnet::new(
        Length::new::<millimeter>(165.0),
        Length::new::<millimeter>(8.0),
        Length::new::<millimeter>(5.0),
        Length::new::<millimeter>(15.0),
        Arc::new(Default::default()),
    )
    .unwrap();
    let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());

    {
        let core = create_core(
            Length::new::<millimeter>(10.0),
            Length::new::<millimeter>(0.0),
            3,
        );
        let drawable = core.drawable();

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

        let path =
            std::path::Path::new("tests/img/lin_straight_indents/assembly_3_indents_no_depth.png");
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
    {
        let core = create_core(
            Length::new::<millimeter>(10.0),
            Length::new::<millimeter>(5.0),
            3,
        );
        let drawable = core.drawable();

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

        let path = std::path::Path::new(
            "tests/img/lin_straight_indents/assembly_3_indents_positive_depth.png",
        );
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
    {
        let core = create_core(
            Length::new::<millimeter>(10.0),
            Length::new::<millimeter>(-5.0),
            3,
        );
        let drawable = core.drawable();

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

        let path = std::path::Path::new(
            "tests/img/lin_straight_indents/assembly_3_indents_negative_depth.png",
        );
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
