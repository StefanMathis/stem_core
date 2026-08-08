use std::{
    any::Any,
    f64::consts::{FRAC_PI_2, PI},
    sync::Arc,
};

use cairo_viewport::{SideLength, Viewport, bounding_box::ToBoundingBox, compare_or_create};
use planar_geo::draw::{Color, Style};
use stem_core::prelude::*;
use stem_slot::{
    planar_geo::{DEFAULT_EPSILON, DEFAULT_MAX_RELATIVE},
    semi_trapezoid::SemiTrapezoidWidthsAndHeightsBuilder,
};

fn create_slotted_core(flux_barrier: Option<V1rFluxBarrier>) -> RotCore {
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
    .unwrap();

    let flux_barrier: Option<Box<dyn FluxBarrier>> = flux_barrier
        .map(Box::new)
        .map(|fb| fb as Box<dyn FluxBarrier>);

    let air_gap = SlottedAirGap::new(28, false, CarterFactorModel::Bin12, Box::new(slot));
    let core = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(54.4),
        yoke_radius: Length::new::<millimeter>(19.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        skew_angle: 0.0,
        air_gap: Box::new(air_gap),
        flux_barrier,
    };

    return core.try_into().unwrap();
}

#[test]
fn test_slotted_core_90deg_open() {
    let mut barrier = V1rFluxBarrier {
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

    {
        let core = create_slotted_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 4);

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/contours_1.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                for c in contours.iter() {
                    c.draw(&style, cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let core = create_slotted_core(Some(barrier));

        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                None,
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );

        let flux_barrier = (core.flux_barrier().unwrap() as &dyn Any)
            .downcast_ref::<V1rFluxBarrier>()
            .unwrap();

        // Geometry comparison
        approx::assert_abs_diff_eq!(core.pole_coverage(None), 8.0 / 14.0, epsilon = 0.0001);
        approx::assert_abs_diff_eq!(
            flux_barrier
                .cache
                .as_ref()
                .unwrap()
                .leakage_segment
                .length(),
            6.76e-3,
            epsilon = 0.0001
        );

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_1.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let magnets: Vec<PositionedMagnetShape> = core.interior_magnets(true).collect();
        let view = Viewport::from_bounded_entities(
            magnets.iter().map(|m| &(m.shape)),
            SideLength::Long(1000),
        )
        .unwrap();
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/magnets_1.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                for m in magnets.as_slice() {
                    m.clone().into_drawable().draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_with_magnets_1.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in magnets.as_slice() {
                    m.clone().into_drawable().draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        assert_eq!(
            2,
            core.interior_magnet_assemblies()
                .iter()
                .map(|m| m.num_magnets())
                .sum::<usize>()
        );
    }
}

#[test]
fn test_slotted_core_90deg_closed() {
    let mut barrier = V1rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(4.5),
        relief_path_air_gap_width: Length::new::<millimeter>(0.0),
        relief_path_length: Length::new::<millimeter>(4.83),
        relief_path_width: Length::new::<millimeter>(4.0),
        opening_angle: FRAC_PI_2,
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height: Length::new::<millimeter>(20.0),
        glue_gap: Length::new::<millimeter>(0.2),
        leakage_path_width: Length::new::<millimeter>(1.0),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
    };

    {
        let core = create_slotted_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 8);

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/contours_2.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                for c in contours.iter() {
                    c.draw(&style, cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let core = create_slotted_core(Some(barrier));
        let flux_barrier = (core.flux_barrier().unwrap() as &dyn Any)
            .downcast_ref::<V1rFluxBarrier>()
            .unwrap();

        // Geometry properties
        approx::assert_abs_diff_eq!(
            flux_barrier
                .distance_to_q_axis_at_yoke()
                .get::<millimeter>(),
            8.1578,
            epsilon = 1e-3
        );
        approx::assert_abs_diff_eq!(
            flux_barrier
                .distance_to_q_axis_at_air_gap()
                .get::<millimeter>(),
            8.1578,
            epsilon = 1e-3
        );

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_2.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let magnets: Vec<PositionedMagnetShape> = core.interior_magnets(true).collect();
        let view = Viewport::from_bounded_entities(
            magnets.iter().map(|m| &(m.shape)),
            SideLength::Long(1000),
        )
        .unwrap();
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/magnets_1.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                for m in magnets.as_slice() {
                    m.clone().into_drawable().draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_with_magnets_2.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in magnets.as_slice() {
                    m.clone().into_drawable().draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
}

#[test]
fn test_plain_core_90deg_open() {
    let barrier = V1rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(4.5),
        relief_path_air_gap_width: Length::new::<millimeter>(3.5),
        relief_path_length: Length::new::<millimeter>(1.33),
        relief_path_width: Length::new::<millimeter>(4.0),
        opening_angle: FRAC_PI_2,
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height: Length::new::<millimeter>(26.0),
        glue_gap: Length::new::<millimeter>(0.2),
        leakage_path_width: Length::new::<millimeter>(1.0),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
    };

    let air_gap = PlainAirGap::new(
        Length::new::<meter>(0.0),
        1.0,
        1.try_into().unwrap(),
        28,
        true,
    )
    .expect("valid data");
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
        flux_barrier: Some(Box::new(barrier)),
    }
    .try_into()
    .unwrap();

    let drawable = core.drawable();
    let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
    let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_3.png");
    let callback = |path: &std::path::Path| {
        return view.write_to_file(path, |cr| {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.paint()?;
            drawable.draw(cr)
        });
    };
    assert!(compare_or_create(path, &callback, 0.98).is_ok());

    let magnets: Vec<PositionedMagnetShape> = core.interior_magnets(true).collect();
    let view =
        Viewport::from_bounded_entities(magnets.iter().map(|m| &(m.shape)), SideLength::Long(1000))
            .unwrap();
    let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/magnets_3.png");
    let callback = |path: &std::path::Path| {
        return view.write_to_file(path, |cr| {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.paint()?;
            for m in magnets.as_slice() {
                m.clone().into_drawable().draw(cr)?;
            }
            return Ok(());
        });
    };
    assert!(compare_or_create(path, &callback, 0.98).is_ok());

    let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
    let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_with_magnets_3.png");
    let callback = |path: &std::path::Path| {
        return view.write_to_file(path, |cr| {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.paint()?;
            drawable.draw(cr)?;
            for m in magnets.as_slice() {
                m.clone().into_drawable().draw(cr)?;
            }
            return Ok(());
        });
    };
    assert!(compare_or_create(path, &callback, 0.98).is_ok());
}

#[test]
fn test_plain_core_90deg_closed() {
    let barrier = V1rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(4.5),
        relief_path_air_gap_width: Length::new::<millimeter>(0.0),
        relief_path_length: Length::new::<millimeter>(4.83),
        relief_path_width: Length::new::<millimeter>(4.0),
        opening_angle: FRAC_PI_2,
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height: Length::new::<millimeter>(26.0),
        glue_gap: Length::new::<millimeter>(0.2),
        leakage_path_width: Length::new::<millimeter>(1.0),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
    };

    let air_gap = PlainAirGap::default();
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
        flux_barrier: Some(Box::new(barrier)),
    }
    .try_into()
    .unwrap();

    let drawable = core.drawable();
    let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
    let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_4.png");
    let callback = |path: &std::path::Path| {
        return view.write_to_file(path, |cr| {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.paint()?;
            drawable.draw(cr)
        });
    };
    assert!(compare_or_create(path, &callback, 0.98).is_ok());

    let magnets: Vec<PositionedMagnetShape> = core.interior_magnets(true).collect();
    let view =
        Viewport::from_bounded_entities(magnets.iter().map(|m| &(m.shape)), SideLength::Long(1000))
            .unwrap();
    let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/magnets_3.png");
    let callback = |path: &std::path::Path| {
        return view.write_to_file(path, |cr| {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.paint()?;
            for m in magnets.as_slice() {
                m.clone().into_drawable().draw(cr)?;
            }
            return Ok(());
        });
    };
    assert!(compare_or_create(path, &callback, 0.98).is_ok());

    let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
    let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_with_magnets_4.png");
    let callback = |path: &std::path::Path| {
        return view.write_to_file(path, |cr| {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.paint()?;
            drawable.draw(cr)?;
            for m in magnets.as_slice() {
                m.clone().into_drawable().draw(cr)?;
            }
            return Ok(());
        });
    };
    assert!(compare_or_create(path, &callback, 0.98).is_ok());
}

#[test]
fn test_slotted_core_45deg_open() {
    let mut barrier = V1rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(4.5),
        relief_path_air_gap_width: Length::new::<millimeter>(3.5),
        relief_path_length: Length::new::<millimeter>(1.33),
        relief_path_width: Length::new::<millimeter>(4.0),
        opening_angle: 45.0 / 180.0 * PI,
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height: Length::new::<millimeter>(15.0),
        glue_gap: Length::new::<millimeter>(0.2),
        leakage_path_width: Length::new::<millimeter>(1.0),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
    };

    {
        let core = create_slotted_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 4);

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/contours_5.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                for c in contours.iter() {
                    c.draw(&style, cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let core: RotCore = create_slotted_core(Some(barrier));
        let flux_barrier = (core.flux_barrier().unwrap() as &dyn Any)
            .downcast_ref::<V1rFluxBarrier>()
            .unwrap();
        let slot = core.slot().unwrap();

        approx::assert_abs_diff_eq!(
            flux_barrier
                .cache
                .as_ref()
                .unwrap()
                .leakage_segment
                .length(),
            slot.width_at(slot.height()).get::<meter>(),
            epsilon = 0.0001
        );

        approx::assert_abs_diff_eq!(
            flux_barrier.leakage_path_width.get::<meter>(),
            1e-3,
            epsilon = 1e-9
        );

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_5.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let magnets: Vec<PositionedMagnetShape> = core.interior_magnets(false).collect();
        let view = Viewport::from_bounded_entities(
            magnets.iter().map(|m| &(m.shape)),
            SideLength::Long(1000),
        )
        .unwrap();
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/magnets_5.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                for m in magnets.as_slice() {
                    m.clone().into_drawable().draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_with_magnets_5.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in magnets.as_slice() {
                    m.clone().into_drawable().draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
}

#[test]
fn test_slotted_core_45deg_closed() {
    let mut barrier = V1rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(4.5),
        relief_path_air_gap_width: Length::new::<millimeter>(0.0),
        relief_path_length: Length::new::<millimeter>(4.83),
        relief_path_width: Length::new::<millimeter>(4.0),
        opening_angle: 45.0 / 180.0 * PI,
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height: Length::new::<millimeter>(15.0),
        glue_gap: Length::new::<millimeter>(0.2),
        leakage_path_width: Length::new::<millimeter>(1.0),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
    };

    {
        let core = create_slotted_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 8);

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/contours_6.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                for c in contours.iter() {
                    c.draw(&style, cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let core: RotCore = create_slotted_core(Some(barrier));
        let flux_barrier = (core.flux_barrier().unwrap() as &dyn Any)
            .downcast_ref::<V1rFluxBarrier>()
            .unwrap();
        let slot = core.slot().unwrap();

        approx::assert_abs_diff_eq!(
            flux_barrier
                .cache
                .as_ref()
                .unwrap()
                .leakage_segment
                .length(),
            slot.width_at(slot.height()).get::<meter>(),
            epsilon = 0.0001
        );

        approx::assert_abs_diff_eq!(
            flux_barrier.leakage_path_width.get::<meter>(),
            1e-3,
            epsilon = 1e-9
        );

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_6.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let magnets: Vec<PositionedMagnetShape> = core.interior_magnets(false).collect();
        let view = Viewport::from_bounded_entities(
            magnets.iter().map(|m| &(m.shape)),
            SideLength::Long(1000),
        )
        .unwrap();
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/magnets_5.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                for m in magnets.as_slice() {
                    m.clone().into_drawable().draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_with_magnets_6.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in magnets.as_slice() {
                    m.clone().into_drawable().draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
}

#[test]
fn test_slotted_core_no_relief_path() {
    let mut barrier = V1rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(4.5),
        relief_path_air_gap_width: Length::new::<millimeter>(3.5),
        relief_path_length: Length::new::<millimeter>(1.33),
        relief_path_width: Length::new::<millimeter>(0.0),
        opening_angle: FRAC_PI_2,
        magnet_space_width: Length::new::<millimeter>(10.0),
        magnet_space_height: Length::new::<millimeter>(20.0),
        glue_gap: Length::new::<millimeter>(0.2),
        leakage_path_width: Length::new::<millimeter>(1.0),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
    };

    {
        let core = create_slotted_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 4);

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/contours_7.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                for c in contours.iter() {
                    c.draw(&style, cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let core = create_slotted_core(Some(barrier));
        let flux_barrier = (core.flux_barrier().unwrap() as &dyn Any)
            .downcast_ref::<V1rFluxBarrier>()
            .unwrap();

        // Geometry comparison
        approx::assert_abs_diff_eq!(core.pole_coverage(None), 8.0 / 14.0, epsilon = 0.0001);
        approx::assert_abs_diff_eq!(
            flux_barrier
                .cache
                .as_ref()
                .unwrap()
                .leakage_segment
                .length(),
            6.76e-3,
            epsilon = 0.0001
        );

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_7.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let magnets: Vec<PositionedMagnetShape> = core.interior_magnets(true).collect();
        let view = Viewport::from_bounded_entities(
            magnets.iter().map(|m| &(m.shape)),
            SideLength::Long(1000),
        )
        .unwrap();
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/magnets_7.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                for m in magnets.as_slice() {
                    m.clone().into_drawable().draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_with_magnets_7.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in magnets.as_slice() {
                    m.clone().into_drawable().draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
}

#[test]
fn test_slotted_core_slim_magnet_width() {
    let mut barrier = V1rFluxBarrier {
        yoke_distance: Length::new::<millimeter>(4.5),
        relief_path_air_gap_width: Length::new::<millimeter>(3.5),
        relief_path_length: Length::new::<millimeter>(1.33),
        relief_path_width: Length::new::<millimeter>(0.0),
        opening_angle: FRAC_PI_2,
        magnet_space_width: Length::new::<millimeter>(5.0),
        magnet_space_height: Length::new::<millimeter>(20.0),
        glue_gap: Length::new::<millimeter>(0.2),
        leakage_path_width: Length::new::<millimeter>(1.0),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
    };

    {
        let core = create_slotted_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 4);

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/contours_8.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                for c in contours.iter() {
                    c.draw(&style, cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let core = create_slotted_core(Some(barrier));
        let flux_barrier = (core.flux_barrier().unwrap() as &dyn Any)
            .downcast_ref::<V1rFluxBarrier>()
            .unwrap();

        // Geometry comparison
        approx::assert_abs_diff_eq!(core.pole_coverage(None), 8.0 / 14.0, epsilon = 0.0001);
        approx::assert_abs_diff_eq!(
            flux_barrier
                .cache
                .as_ref()
                .unwrap()
                .leakage_segment
                .length(),
            6.76e-3,
            epsilon = 0.0001
        );

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_8.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let magnets: Vec<PositionedMagnetShape> = core.interior_magnets(true).collect();
        let view = Viewport::from_bounded_entities(
            magnets.iter().map(|m| &(m.shape)),
            SideLength::Long(1000),
        )
        .unwrap();
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/magnets_8.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                for m in magnets.as_slice() {
                    m.clone().into_drawable().draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_with_magnets_8.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in magnets.as_slice() {
                    m.clone().into_drawable().draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
}

#[test]
fn test_deserialize() {
    let yaml = indoc::indoc! {"
    ---
    air_gap_radius: 54.4 mm
    yoke_radius: 19 mm
    axial_length: 165 mm
    iron_fill_factor: 0.95
    material:
        name: M270-50A
        mass_density: 1
    pole_pairs: 2
    skew_angle: 0.0 deg
    axial_coil_overhang: 0 mm
    air_gap:
        SlottedAirGap:
            slots: 28
            starts_in_slot_middle: false
            carter_factor_model: Bin12
            slot:
                SemiTrapezoidSlot:
                    bottom_width: 6.75 mm
                    top_width: 1.5 mm
                    top_side_width: 8 mm
                    opening_width: 1.5 mm
                    height: 6.79 mm
                    opening_height: 0.75 mm
                    slot_angle: -360/28 deg
                    bottom_angle:
                        bottom_width: 6.75 mm
                        bottom_side_width: 6.75 mm
                        bottom_height: 0.0 mm
                        slot_angle: -360/28 deg
                    top_angle:
                        top_width: 1.5 mm
                        top_side_width: 8 mm
                        top_height: 0.5 mm
                        slot_angle: -360/28 deg
                    bottom_radius: 0.0 mm
                    bottom_side_radius: 0.0 mm
                    top_radius: 0.0 mm
                    top_side_radius: 0.0 mm
                    opening_radius: 0.0 mm
                    consider_tooth_tip_leakage: true
    flux_barrier:
        V1rFluxBarrier:
            yoke_distance: 4.5 mm
            relief_path_air_gap_width: 3.5 mm
            relief_path_length: 1.33 mm
            relief_path_width: 4.0 mm
            opening_angle: 90 deg
            magnet_space_width: 10.0 mm
            magnet_space_height: 20.0 mm
            glue_gap: 0.2 mm
            leakage_path_width: 1 mm
            magnet_material:
    "};
    let core: RotCore = serde_yaml::from_str(&yaml).unwrap();

    let drawable = core.drawable();
    let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
    let path = std::path::Path::new("tests/img/v1r_flux_barrier_rot/core_1.png");
    let callback = |path: &std::path::Path| {
        return view.write_to_file(path, |cr| {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.paint()?;
            drawable.draw(cr)
        });
    };
    assert!(compare_or_create(path, &callback, 0.98).is_ok());
}
