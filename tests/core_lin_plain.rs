use std::sync::Arc;

use cairo_viewport::{Viewport, compare_or_create};
use planar_geo::prelude::ToBoundingBox;
use stem_core::{magnets::PositionedMagnetShape, prelude::*};

#[test]
fn test_properties() {
    let air_gap = PlainAirGap::new(1, Length::new::<millimeter>(10.0), 0.7, 12, false).unwrap();

    let core = LinCore::try_from(LinCoreBuilder {
        height: Length::new::<millimeter>(25.0),
        width: Length::new::<millimeter>(150.0),
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

    approxim::assert_abs_diff_eq!(core.height().get::<meter>(), 0.025);
    approxim::assert_abs_diff_eq!(core.width().get::<meter>(), 0.15);
    approxim::assert_abs_diff_eq!(core.mass().get::<gram>(), 375.0);
    approxim::assert_abs_diff_eq!(
        core.tooth_width_at(Length::new::<millimeter>(0.0))
            .get::<meter>(),
        0.0
    );
    approxim::assert_abs_diff_eq!(core.tooth_height().get::<meter>(), 0.0);
    approxim::assert_abs_diff_eq!(core.teeth_mass().get::<gram>(), 0.0);
    approxim::assert_abs_diff_eq!(core.tooth_mass().get::<gram>(), 0.0);
    approxim::assert_abs_diff_eq!(core.slot_opening_factor(1), 0.994412, epsilon = 1e-6);
    approxim::assert_abs_diff_eq!(core.slot_opening_factor(10), 0.527081, epsilon = 1e-6);
}

#[test]
fn test_current_displacement_coefficients() {
    let air_gap = PlainAirGap::new(1, Length::new::<millimeter>(10.0), 0.7, 12, false).unwrap();

    let core = LinCore::try_from(LinCoreBuilder {
        height: Length::new::<millimeter>(25.0),
        width: Length::new::<millimeter>(150.0),
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

    let coeffs = core.current_displacement_coefficients().eval(
        Frequency::new::<uom::si::frequency::hertz>(200.0),
        ElectricalConductivity::new::<siemens_per_meter>(57e6),
        100.0,
    );
    approxim::assert_abs_diff_eq!(coeffs.resistance, 1.0, epsilon = 1e-6);
    approxim::assert_abs_diff_eq!(coeffs.inductance, 1.0, epsilon = 1e-3);
}

#[test]
fn test_failed_creating_core() {
    let air_gap = PlainAirGap::new(1, Length::new::<millimeter>(0.0), 1.0, 12, true).unwrap();

    // Fails because the iron fill factor is not between 0 and 1
    assert!(
        LinCore::try_from(LinCoreBuilder {
            height: Length::new::<millimeter>(100.0),
            width: Length::new::<millimeter>(200.0),
            axial_length: Length::new::<millimeter>(100.0),
            axial_coil_overhang: Length::new::<millimeter>(0.0),
            skew_angle: 0.0,
            iron_fill_factor: -0.1,
            material: Arc::new(Material::default()),
            pole_pairs: 4,
            air_gap: Box::new(air_gap.clone()),
            flux_barrier: None,
        })
        .is_err()
    );

    // Fails because the iron fill factor is not between 0 and 1
    assert!(
        LinCore::try_from(LinCoreBuilder {
            height: Length::new::<millimeter>(100.0),
            width: Length::new::<millimeter>(200.0),
            axial_length: Length::new::<millimeter>(100.0),
            axial_coil_overhang: Length::new::<millimeter>(0.0),
            skew_angle: 0.0,
            iron_fill_factor: 1.1,
            material: Arc::new(Material::default()),
            pole_pairs: 4,
            air_gap: Box::new(air_gap.clone()),
            flux_barrier: None,
        })
        .is_err()
    );
}

#[test]
fn test_pole_coverage() {
    let air_gap = PlainAirGap::new(1, Length::new::<millimeter>(0.0), 1.0, 12, true).unwrap();

    let core = LinCore::try_from(LinCoreBuilder {
        height: Length::new::<millimeter>(25.0),
        width: Length::new::<millimeter>(150.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 1,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    })
    .unwrap();

    let magnet = BreadLoafMagnet::new(
        Length::new::<millimeter>(165.0),
        Length::new::<millimeter>(20.0),
        Length::new::<millimeter>(10.0),
        Length::new::<millimeter>(50.0),
        Arc::new(Default::default()),
    )
    .unwrap();

    let assembly =
        MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap());

    approxim::assert_abs_diff_eq!(core.pole_coverage(Some(&assembly)), 0.8, epsilon = 1e-6);
}

#[test]
fn test_plot_core() {
    let air_gap = PlainAirGap::new(1, Length::new::<millimeter>(10.0), 0.7, 12, false).unwrap();

    let core = LinCore::try_from(LinCoreBuilder {
        height: Length::new::<millimeter>(25.0),
        width: Length::new::<millimeter>(150.0),
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
    let path = std::path::Path::new("tests/img/lin_plain/core.png");

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
fn test_plot_air_gap_winding() {
    let air_gap = PlainAirGap::new(1, Length::new::<millimeter>(10.0), 0.7, 12, false).unwrap();

    let core = LinCore::try_from(LinCoreBuilder {
        height: Length::new::<millimeter>(25.0),
        width: Length::new::<millimeter>(150.0),
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
        let path = std::path::Path::new("tests/img/lin_plain/wdg_double_vertical.png");
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
        let path = std::path::Path::new("tests/img/lin_plain/wdg_double_horizontal.png");
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
        let path = std::path::Path::new("tests/img/lin_plain/wdg_multi_vert.png");
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
    let air_gap = PlainAirGap::new(1, Length::new::<millimeter>(10.0), 0.9, 12, true).unwrap();

    let core = LinCore::try_from(LinCoreBuilder {
        height: Length::new::<millimeter>(25.0),
        width: Length::new::<millimeter>(150.0),
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
        let path =
            std::path::Path::new("tests/img/lin_plain/wdg_double_horizontal_slot_middle.png");
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
        let path = std::path::Path::new("tests/img/lin_plain/wdg_quadruple_slot_middle.png");
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
    let air_gap = PlainAirGap::new(1, Length::new::<millimeter>(10.0), 0.0, 12, true).unwrap();

    let core = LinCore::try_from(LinCoreBuilder {
        height: Length::new::<millimeter>(25.0),
        width: Length::new::<millimeter>(150.0),
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
        let path = std::path::Path::new("tests/img/lin_plain/wdg_zero_coverage.png");
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
fn test_plot_air_gap_surface_magnets() {
    let air_gap = PlainAirGap::new(1, Length::new::<millimeter>(10.0), 0.0, 12, true).unwrap();

    let core = LinCore::try_from(LinCoreBuilder {
        height: Length::new::<millimeter>(25.0),
        width: Length::new::<millimeter>(150.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 1,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    })
    .unwrap();

    let magnet = BreadLoafMagnet::new(
        Length::new::<millimeter>(165.0),
        Length::new::<millimeter>(20.0),
        Length::new::<millimeter>(10.0),
        Length::new::<millimeter>(50.0),
        Arc::new(Default::default()),
    )
    .unwrap();

    let assembly =
        MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap());

    let drawable = core.drawable();
    let mut bb = drawable.bounding_box();
    bb.try_set_ymin(bb.ymin() - 0.02);

    let view = Viewport::from_bounding_box(&bb, cairo_viewport::SideLength::Long(500));

    {
        let path = std::path::Path::new("tests/img/lin_plain/surface_magnets_unsplit.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .surface_magnets(&assembly, false)
                    .map(PositionedMagnetShape::into_drawable)
                {
                    d.draw(cr)?;
                }

                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.99).is_ok());
    }
    {
        let path = std::path::Path::new("tests/img/lin_plain/surface_magnets_split.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;

                for d in core
                    .surface_magnets(&assembly, true)
                    .map(PositionedMagnetShape::into_drawable)
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
    let builder = LinCoreBuilder {
        height: Length::new::<millimeter>(20.0),
        width: Length::new::<millimeter>(100.0),
        axial_length: Length::new::<millimeter>(100.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        skew_angle: 0.0,
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        air_gap: Box::new(air_gap),
        flux_barrier: None,
    };

    let core = LinCore::new(builder).expect("valid inputs");
    let serialized = yaml_serde::to_string(&core).expect("can be serialized");
    let de_core: LinCore = yaml_serde::from_str(&serialized).expect("can be deserialized");
    assert_eq!(core.width(), de_core.width());
}
