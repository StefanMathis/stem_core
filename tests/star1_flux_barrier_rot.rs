use std::sync::Arc;

use cairo_viewport::{SideLength, Viewport, bounding_box::ToBoundingBox, compare_or_create};
use planar_geo::draw::{Color, Style};
use stem_core::prelude::*;
use stem_slot::semi_trapezoid::SemiTrapezoidWidthsAndHeightsBuilder;

fn create_plain_inner_core(flux_barrier: Option<Star1FluxBarrier>) -> RotCore {
    let air_gap = PlainAirGap::new(
        Length::new::<millimeter>(10.0),
        0.7,
        1.try_into().unwrap(),
        12,
        false,
    )
    .unwrap();

    let flux_barrier: Option<Box<dyn FluxBarrier>> = flux_barrier
        .map(Box::new)
        .map(|fb| fb as Box<dyn FluxBarrier>);

    let core = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(54.4),
        yoke_radius: Length::new::<millimeter>(19.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 4,
        skew_angle: 0.0,
        air_gap: Box::new(air_gap),
        flux_barrier,
    };

    return core.try_into().unwrap();
}

fn create_plain_outer_core(flux_barrier: Option<Star1FluxBarrier>) -> RotCore {
    let air_gap = PlainAirGap::new(
        Length::new::<millimeter>(10.0),
        0.7,
        1.try_into().unwrap(),
        12,
        false,
    )
    .unwrap();

    let flux_barrier: Option<Box<dyn FluxBarrier>> = flux_barrier
        .map(Box::new)
        .map(|fb| fb as Box<dyn FluxBarrier>);

    let core = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(19.0),
        yoke_radius: Length::new::<millimeter>(54.4),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 4,
        skew_angle: 0.0,
        air_gap: Box::new(air_gap),
        flux_barrier,
    };

    return core.try_into().unwrap();
}

fn create_slotted_inner_core(flux_barrier: Option<Star1FluxBarrier>) -> RotCore {
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

    let air_gap = SlottedAirGap::new(28, true, CarterFactorModel::Bin12, Box::new(slot));
    let core = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(54.4),
        yoke_radius: Length::new::<millimeter>(19.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 4,
        skew_angle: 0.0,
        air_gap: Box::new(air_gap),
        flux_barrier,
    };

    return core.try_into().unwrap();
}

fn create_slotted_outer_core(flux_barrier: Option<Star1FluxBarrier>) -> RotCore {
    let slot: SemiTrapezoidSlot = SemiTrapezoidWidthsAndHeightsBuilder {
        bottom_width: Length::new::<millimeter>(8.0),
        bottom_side_width: Length::new::<millimeter>(8.0),
        top_side_width: Length::new::<millimeter>(6.76),
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

    let air_gap = SlottedAirGap::new(8, true, CarterFactorModel::Bin12, Box::new(slot));
    let core = RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(19.0),
        yoke_radius: Length::new::<millimeter>(54.4),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 4,
        skew_angle: 0.0,
        air_gap: Box::new(air_gap),
        flux_barrier,
    };

    return core.try_into().unwrap();
}

#[test]
fn plain_inner_with_relief_path() {
    let mut barrier = Star1FluxBarrier {
        magnet_space_width: Length::new::<millimeter>(10.0),
        glue_gap: Length::new::<millimeter>(0.2),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(5.0),
        magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<
            millimeter,
        >(2.0)),
    };

    {
        let core = create_plain_inner_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 8);

        let c = barrier.cache.as_ref().unwrap();
        approx::assert_abs_diff_eq!(c.pt_air_gap_leakage[1] - c.pt_inner_relief[1], 0.002);

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/contours_1.png");
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
        let core = create_plain_inner_core(Some(barrier));

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_1.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/magnets_1.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_with_magnets_1.png");
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
            1,
            core.interior_magnet_assemblies()
                .iter()
                .map(|m| m.num_magnets())
                .sum::<usize>()
        );
    }
}

#[test]
fn plain_inner_no_relief_path() {
    let mut barrier = Star1FluxBarrier {
        magnet_space_width: Length::new::<millimeter>(10.0),
        glue_gap: Length::new::<millimeter>(0.2),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(10.0),
        magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<
            millimeter,
        >(0.0)),
    };

    {
        let core = create_plain_inner_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 8);

        let c = barrier.cache.as_ref().unwrap();
        approx::assert_abs_diff_eq!(c.pt_air_gap_leakage[1] - c.pt_inner_relief[1], 0.0);

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/contours_2.png");
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
        let core = create_plain_inner_core(Some(barrier));

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_2.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/magnets_2.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_with_magnets_2.png");
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
fn slotted_inner_with_relief_path() {
    let mut barrier = Star1FluxBarrier {
        magnet_space_width: Length::new::<millimeter>(10.0),
        glue_gap: Length::new::<millimeter>(0.2),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(5.0),
        magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<
            millimeter,
        >(2.0)),
    };

    {
        let core = create_slotted_inner_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 8);

        let c = barrier.cache.as_ref().unwrap();
        approx::assert_abs_diff_eq!(c.pt_air_gap_leakage[1] - c.pt_inner_relief[1], 0.002);

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/contours_3.png");
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
        let core = create_slotted_inner_core(Some(barrier));

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_3.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/magnets_3.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_with_magnets_3.png");
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
fn slotted_inner_no_relief_path() {
    let mut barrier = Star1FluxBarrier {
        magnet_space_width: Length::new::<millimeter>(10.0),
        glue_gap: Length::new::<millimeter>(0.2),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(5.0),
        magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<
            millimeter,
        >(0.0)),
    };

    {
        let core = create_slotted_inner_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 8);

        let c = barrier.cache.as_ref().unwrap();
        approx::assert_abs_diff_eq!(c.pt_air_gap_leakage[1] - c.pt_inner_relief[1], 0.0);

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/contours_4.png");
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
        let core = create_slotted_inner_core(Some(barrier));

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_4.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/magnets_4.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_with_magnets_4.png");
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
fn plain_outer_with_relief_path() {
    let mut barrier = Star1FluxBarrier {
        magnet_space_width: Length::new::<millimeter>(10.0),
        glue_gap: Length::new::<millimeter>(0.2),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(5.0),
        magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<
            millimeter,
        >(2.0)),
    };

    {
        let core = create_plain_outer_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 8);

        let c = barrier.cache.as_ref().unwrap();
        approx::assert_abs_diff_eq!(
            (c.pt_air_gap_leakage[1] - c.pt_inner_relief[1]).abs(),
            0.002
        );

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/contours_5.png");
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
        let core = create_plain_outer_core(Some(barrier));

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_5.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/magnets_5.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_with_magnets_5.png");
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
fn plain_outer_no_relief_path() {
    let mut barrier = Star1FluxBarrier {
        magnet_space_width: Length::new::<millimeter>(10.0),
        glue_gap: Length::new::<millimeter>(0.2),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(10.0),
        magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<
            millimeter,
        >(0.0)),
    };

    {
        let core = create_plain_outer_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 8);

        let c = barrier.cache.as_ref().unwrap();
        approx::assert_abs_diff_eq!((c.pt_air_gap_leakage[1] - c.pt_inner_relief[1]).abs(), 0.0);

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/contours_6.png");
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
        let core = create_plain_outer_core(Some(barrier));

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_6.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/magnets_6.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_with_magnets_6.png");
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
fn slotted_outer_with_relief_path() {
    let mut barrier = Star1FluxBarrier {
        magnet_space_width: Length::new::<millimeter>(10.0),
        glue_gap: Length::new::<millimeter>(0.2),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(5.0),
        magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<
            millimeter,
        >(2.0)),
    };

    {
        let core = create_slotted_outer_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 8);

        let c = barrier.cache.as_ref().unwrap();
        approx::assert_abs_diff_eq!(
            (c.pt_air_gap_leakage[1] - c.pt_inner_relief[1]).abs(),
            0.002
        );

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/contours_7.png");
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
        let core = create_slotted_outer_core(Some(barrier));

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_7.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/magnets_7.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_with_magnets_7.png");
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
fn slotted_outer_no_relief_path() {
    let mut barrier = Star1FluxBarrier {
        magnet_space_width: Length::new::<millimeter>(10.0),
        glue_gap: Length::new::<millimeter>(0.2),
        magnet_material: Some(Arc::new(Material::default())),
        cache: None,
        air_gap_leakage_path_width: Length::new::<millimeter>(1.0),
        yoke_leakage_path_width: Length::new::<millimeter>(1.0),
        relief_path_air_gap_width: Length::new::<millimeter>(5.0),
        magnet_space_height_or_relief_path_width: Star1HeightSplit::ReliefPathWidth(Length::new::<
            millimeter,
        >(0.0)),
    };

    {
        let core = create_slotted_outer_core(None);
        let contours = barrier.combine(core.as_core_ref()).expect("valid contours");
        assert_eq!(contours.len(), 8);

        let c = barrier.cache.as_ref().unwrap();
        approx::assert_abs_diff_eq!((c.pt_air_gap_leakage[1] - c.pt_inner_relief[1]).abs(), 0.0);

        let mut style = Style::default();
        style.background_color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };

        let view = Viewport::from_bounded_entities(contours.iter(), SideLength::Long(1000))
            .expect("contours is not empty");
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/contours_8.png");
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
        let core = create_slotted_outer_core(Some(barrier));

        let drawable = core.drawable();
        let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_8.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        let magnets: Vec<PositionedMagnetShape> = core.interior_magnets(true).collect();
        println!("{:?}", magnets);
        let view = Viewport::from_bounded_entities(
            magnets.iter().map(|m| &(m.shape)),
            SideLength::Long(1000),
        )
        .unwrap();
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/magnets_8.png");
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
        let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_with_magnets_8.png");
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
    air_gap_radius: 54.5 mm
    yoke_radius: 19 mm
    axial_length: 165 mm
    axial_coil_overhang: 0 mm
    iron_fill_factor: 1.0
    material:
        name: M270-50A
        mass_density: 1
    pole_pairs: 4
    skew_angle: 0.0
    air_gap:
        PlainAirGap:
            air_gap_winding_height: 10 mm
            winding_coverage: 0.7
            num_segments: 1
            starts_in_slot_middle: false
            slots: 12
    flux_barrier:
        Star1FluxBarrier:
            magnet_space_width: 10 mm
            glue_gap: 0.2 mm
            magnet_material:
            air_gap_leakage_path_width: 1 mm
            yoke_leakage_path_width: 1 mm
            relief_path_air_gap_width: 5 mm
            magnet_space_height_or_relief_path_width:
                ReliefPathWidth:
                    2 mm
    "};

    let core: RotCore = serde_yaml::from_str(&yaml).unwrap();

    let drawable = core.drawable();
    let view = Viewport::from_bounding_box(&drawable.bounding_box(), SideLength::Long(1000));
    let path = std::path::Path::new("tests/img/star1_flux_barrier_rot/core_1.png");
    let callback = |path: &std::path::Path| {
        return view.write_to_file(path, |cr| {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.paint()?;
            drawable.draw(cr)
        });
    };
    assert!(compare_or_create(path, &callback, 0.98).is_ok());
}
