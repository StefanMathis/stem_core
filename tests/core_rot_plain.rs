use std::{f64::consts::PI, sync::Arc};

use cairo_viewport::{Viewport, compare_or_create};
use planar_geo::prelude::ToBoundingBox;
use stem_core::prelude::*;
use stem_slot::planar_geo::draw::Drawable;

#[test]
fn test_plot() {
    let air_gap = PlainAirGap {
        num_segments: 0,
        air_gap_winding_height: Length::new::<millimeter>(10.0),
        winding_coverage: 1.0,
        starts_in_slot_middle: false,
        slots: 12,
    };
    let core = RotCore::try_from(RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(85.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 5,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    })
    .unwrap();

    let drawable = core.drawable();
    let view = Viewport::from_bounded_entity(&drawable, cairo_viewport::SideLength::Long(500));
    let path = std::path::Path::new("tests/img/rot_plain/core.png");

    let callback = |path: &std::path::Path| {
        return view.write_to_file(path, |cr| {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.paint()?;
            drawable.draw(cr)
        });
    };
    assert!(compare_or_create(path, &callback, 0.99).is_ok());
}

#[test]
fn test_read_core_properties() {
    {
        let air_gap = PlainAirGap {
            num_segments: 0,
            air_gap_winding_height: Length::new::<millimeter>(10.0),
            winding_coverage: 1.0,
            starts_in_slot_middle: false,
            slots: 12,
        };
        let core = RotCore::try_from(RotCoreBuilder {
            air_gap_radius: Length::new::<millimeter>(55.0),
            yoke_radius: Length::new::<millimeter>(85.0),
            axial_length: Length::new::<millimeter>(100.0),
            axial_coil_overhang: Length::new::<millimeter>(0.0),
            skew_angle: 0.0,
            iron_fill_factor: 1.0,
            material: Arc::new(Material::default()),
            pole_pairs: 5,
            air_gap: Box::new(air_gap),
            flux_barrier: None,
        })
        .unwrap();

        approxim::assert_abs_diff_eq!(
            core.yoke_height().get::<millimeter>(),
            30.0,
            epsilon = 1e-10
        );
        assert_eq!(core.pole_pairs(), 5);
        assert_eq!(core.slots(), 12);
        approxim::assert_abs_diff_eq!(core.slot_opening_factor(1), 0.9886159, epsilon = 1e-6);
    }
    {
        let air_gap = PlainAirGap {
            num_segments: 0,
            air_gap_winding_height: Length::new::<millimeter>(0.0),
            winding_coverage: 1.0,
            starts_in_slot_middle: false,
            slots: 12,
        };
        let core = RotCore::try_from(RotCoreBuilder {
            air_gap_radius: Length::new::<millimeter>(55.0),
            yoke_radius: Length::new::<millimeter>(85.0),
            axial_length: Length::new::<millimeter>(100.0),
            axial_coil_overhang: Length::new::<millimeter>(0.0),
            skew_angle: 0.0,
            iron_fill_factor: 1.0,
            material: Arc::new(Material::default()),
            pole_pairs: 5,
            air_gap: Box::new(air_gap),
            flux_barrier: None,
        })
        .unwrap();
        assert_eq!(core.slots(), 0);
    }
    {
        let air_gap = PlainAirGap {
            num_segments: 0,
            air_gap_winding_height: Length::new::<millimeter>(10.0),
            winding_coverage: 0.0,
            starts_in_slot_middle: false,
            slots: 12,
        };
        let core = RotCore::try_from(RotCoreBuilder {
            air_gap_radius: Length::new::<millimeter>(55.0),
            yoke_radius: Length::new::<millimeter>(85.0),
            axial_length: Length::new::<millimeter>(100.0),
            axial_coil_overhang: Length::new::<millimeter>(0.0),
            skew_angle: 0.0,
            iron_fill_factor: 1.0,
            material: Arc::new(Material::default()),
            pole_pairs: 5,
            air_gap: Box::new(air_gap),
            flux_barrier: None,
        })
        .unwrap();
        assert_eq!(core.slots(), 0);
    }
}

#[test]
fn test_pole_coverage() {
    let air_gap = PlainAirGap {
        num_segments: 0,
        air_gap_winding_height: Length::new::<millimeter>(10.0),
        winding_coverage: 0.0,
        starts_in_slot_middle: false,
        slots: 12,
    };

    let core = RotCore::try_from(RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(85.0),
        yoke_radius: Length::new::<millimeter>(55.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    })
    .unwrap();

    let magnet = ArcParallelMagnet::with_const_thickness(
        Length::new::<millimeter>(165.0),
        core.air_gap_radius(),
        SideHeightOrThickness::Thickness(Length::new::<millimeter>(10.0)),
        AngleOrWidth::Angle(10.0 / 180.0 * PI),
        Arc::new(Default::default()),
    )
    .unwrap();

    let assembly =
        MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap());

    approxim::assert_abs_diff_eq!(
        core.pole_coverage(Some(&assembly)),
        1.0 / 3.0,
        epsilon = 1e-6
    );
}

#[test]
fn test_create_core() {
    let air_gap = PlainAirGap {
        num_segments: 0,
        air_gap_winding_height: Length::new::<millimeter>(10.0),
        winding_coverage: 1.0,
        starts_in_slot_middle: false,
        slots: 12,
    };

    assert!(
        RotCore::try_from(RotCoreBuilder {
            air_gap_radius: Length::new::<millimeter>(8.0),
            yoke_radius: Length::new::<millimeter>(10.0),
            axial_length: Length::new::<millimeter>(100.0),
            axial_coil_overhang: Length::new::<millimeter>(0.0),
            skew_angle: 0.0,
            iron_fill_factor: 1.0,
            material: Arc::new(Material::default()),
            pole_pairs: 5,
            air_gap: Box::new(air_gap.clone()),
            flux_barrier: None,
        })
        .is_err()
    );

    assert!(
        RotCore::try_from(RotCoreBuilder {
            air_gap_radius: Length::new::<millimeter>(10.0),
            yoke_radius: Length::new::<millimeter>(8.0),
            axial_length: Length::new::<millimeter>(100.0),
            axial_coil_overhang: Length::new::<millimeter>(0.0),
            skew_angle: 0.0,
            iron_fill_factor: 1.0,
            material: Arc::new(Material::default()),
            pole_pairs: 5,
            air_gap: Box::new(air_gap),
            flux_barrier: None,
        })
        .is_ok()
    );
}

#[test]
fn test_plot_air_gap_winding() {
    let air_gap = PlainAirGap {
        num_segments: 0,
        air_gap_winding_height: Length::new::<millimeter>(10.0),
        winding_coverage: 1.0,
        starts_in_slot_middle: false,
        slots: 12,
    };
    let core = RotCore::try_from(RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(85.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 5,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    })
    .unwrap();

    let drawable = core.drawable();
    let bb = drawable.bounding_box();

    let view = Viewport::from_bounding_box(&bb, cairo_viewport::SideLength::Long(500));

    {
        let path = std::path::Path::new("tests/img/rot_plain/wdg_double_vertical.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .winding_zones(&CoilLayout::DoubleVertical)
                    .map(PositionedZoneContour::into_drawable)
                {
                    d.draw(cr)?;
                }

                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.99).is_ok());
    }
    {
        let path = std::path::Path::new("tests/img/rot_plain/wdg_double_horizontal.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .winding_zones(&CoilLayout::DoubleHorizontal)
                    .map(PositionedZoneContour::into_drawable)
                {
                    d.draw(cr)?;
                }

                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.99).is_ok());
    }
    {
        let path = std::path::Path::new("tests/img/rot_plain/wdg_multi_vert.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .winding_zones(&CoilLayout::MultiVertical(3))
                    .map(PositionedZoneContour::into_drawable)
                {
                    d.draw(cr)?;
                }

                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.99).is_ok());
    }
}

#[test]
fn test_plot_air_gap_winding_slot_middle() {
    let air_gap = PlainAirGap {
        num_segments: 0,
        air_gap_winding_height: Length::new::<millimeter>(10.0),
        winding_coverage: 0.9,
        starts_in_slot_middle: true,
        slots: 12,
    };

    let core = RotCore::try_from(RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(85.0),
        yoke_radius: Length::new::<millimeter>(55.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 5,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    })
    .unwrap();

    let drawable = core.drawable();
    let mut bb = drawable.bounding_box();
    bb.try_set_xmin(bb.xmin() - 0.01);
    bb.try_set_ymin(bb.ymin() - 0.01);
    bb.try_set_xmax(bb.xmax() + 0.01);
    bb.try_set_ymax(bb.ymax() + 0.01);

    let view = Viewport::from_bounding_box(&bb, cairo_viewport::SideLength::Long(500));

    {
        let path =
            std::path::Path::new("tests/img/rot_plain/wdg_double_horizontal_slot_middle.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .winding_zones(&CoilLayout::DoubleHorizontal)
                    .map(PositionedZoneContour::into_drawable)
                {
                    d.draw(cr)?;
                }

                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.99).is_ok());
    }

    {
        let path = std::path::Path::new("tests/img/rot_plain/wdg_quadruple_slot_middle.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .winding_zones(&CoilLayout::Quadruple)
                    .map(PositionedZoneContour::into_drawable)
                {
                    d.draw(cr)?;
                }

                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.99).is_ok());
    }
}

#[test]
fn test_plot_air_gap_winding_zero_coverage() {
    let air_gap = PlainAirGap {
        num_segments: 0,
        air_gap_winding_height: Length::new::<millimeter>(10.0),
        winding_coverage: 0.0,
        starts_in_slot_middle: false,
        slots: 12,
    };

    let core = RotCore::try_from(RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(85.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 5,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    })
    .unwrap();

    let drawable = core.drawable();
    let mut bb = drawable.bounding_box();
    bb.try_set_ymin(bb.ymin() - 0.01);

    let view = Viewport::from_bounding_box(&bb, cairo_viewport::SideLength::Long(500));

    {
        let path = std::path::Path::new("tests/img/rot_plain/wdg_zero_coverage.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .winding_zones(&CoilLayout::DoubleHorizontal)
                    .map(PositionedZoneContour::into_drawable)
                {
                    d.draw(cr)?;
                }

                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.99).is_ok());
    }
}

#[test]
fn test_plot_inner_air_gap_arc_parallel_magnet() {
    let air_gap = PlainAirGap::default();

    let core = RotCore::try_from(RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(85.0),
        yoke_radius: Length::new::<millimeter>(55.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    })
    .unwrap();

    let magnet = ArcParallelMagnet::with_const_thickness(
        Length::new::<millimeter>(165.0),
        core.air_gap_radius(),
        SideHeightOrThickness::Thickness(Length::new::<millimeter>(10.0)),
        AngleOrWidth::Angle(10.0 / 180.0 * PI),
        Arc::new(Default::default()),
    )
    .unwrap();
    let assembly =
        MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap());

    let drawable = core.drawable();
    let mut bb = drawable.bounding_box();
    bb.try_set_xmin(bb.xmin() - 0.01);
    bb.try_set_ymin(bb.ymin() - 0.01);
    bb.try_set_xmax(bb.xmax() + 0.01);
    bb.try_set_ymax(bb.ymax() + 0.01);

    let view = Viewport::from_bounding_box(&bb, cairo_viewport::SideLength::Long(500));

    {
        let path =
            std::path::Path::new("tests/img/rot_plain/arc_parallel_magnet_inner_unsplit.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .surface_magnets(&assembly, false)
                    .map::<Drawable, _>(From::from)
                {
                    d.draw(cr)?;
                }

                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.99).is_ok());
    }
    {
        let path = std::path::Path::new("tests/img/rot_plain/arc_parallel_magnet_inner_split.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .surface_magnets(&assembly, true)
                    .map::<Drawable, _>(From::from)
                {
                    d.draw(cr)?;
                }

                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.99).is_ok());
    }
}

#[test]
fn test_plot_outer_air_gap_arc_parallel_magnet() {
    let air_gap = PlainAirGap::default();

    let core = RotCore::try_from(RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(85.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    })
    .unwrap();

    let magnet = ArcParallelMagnet::with_const_thickness(
        Length::new::<millimeter>(165.0),
        -core.air_gap_radius(),
        SideHeightOrThickness::Thickness(Length::new::<millimeter>(10.0)),
        AngleOrWidth::Angle(10.0 / 180.0 * PI),
        Arc::new(Default::default()),
    )
    .unwrap();
    let assembly =
        MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap());

    let drawable = core.drawable();
    let bb = drawable.bounding_box();
    let view = Viewport::from_bounding_box(&bb, cairo_viewport::SideLength::Long(500));

    {
        let path =
            std::path::Path::new("tests/img/rot_plain/arc_parallel_magnet_outer_unsplit.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .surface_magnets(&assembly, false)
                    .map::<Drawable, _>(From::from)
                {
                    d.draw(cr)?;
                }

                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.99).is_ok());
    }
    {
        let path = std::path::Path::new("tests/img/rot_plain/arc_parallel_magnet_outer_split.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .surface_magnets(&assembly, true)
                    .map::<Drawable, _>(From::from)
                {
                    d.draw(cr)?;
                }

                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.99).is_ok());
    }

    let magnet = ArcSegmentMagnet::with_const_thickness(
        Length::new::<millimeter>(165.0),
        -core.air_gap_radius(),
        Length::new::<millimeter>(10.0),
        10.0 / 180.0 * PI,
        Arc::new(Default::default()),
    )
    .unwrap();
    let assembly =
        MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap());
    {
        let path = std::path::Path::new("tests/img/rot_plain/arc_segment_magnet_outer_split.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .surface_magnets(&assembly, true)
                    .map::<Drawable, _>(From::from)
                {
                    d.draw(cr)?;
                }

                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.99).is_ok());
    }
}

#[test]
fn serialize_and_deserialize() {
    let air_gap = PlainAirGap::default();
    let builder = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(85.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 5,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    };

    let core = RotCore::new(builder).expect("valid inputs");
    let serialized = yaml_serde::to_string(&core).expect("can be serialized");
    let de_core: RotCore = yaml_serde::from_str(&serialized).expect("can be deserialized");
    assert_eq!(core.air_gap_radius(), de_core.air_gap_radius());
}
