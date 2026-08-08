use std::f64::consts::{PI, TAU};
use std::sync::Arc;

use cairo_viewport::bounding_box::ToBoundingBox;
use cairo_viewport::{SideLength, Viewport, compare_or_create};
use stem_core::magnets::PositionedMagnetShape;
use stem_core::prelude::*;
use stem_core::stem_slot::semi_trapezoid::SemiTrapezoidWithoutSlopesBuilder;
use stem_slot::planar_geo::draw::Drawable;
use stem_slot::planar_geo::{DEFAULT_EPSILON, DEFAULT_MAX_RELATIVE};
use stem_slot::semi_trapezoid::SemiTrapezoidWidthsAndHeightsBuilder;
use uom::typenum::P2;

fn create_outer_core(
    slots: u16,
    pole_pairs: u16,
    starts_in_slot_middle: bool,
    open_slot: bool,
) -> RotCore {
    let mut yoke_radius = Length::new::<millimeter>(20.0);
    let opening_width = if open_slot {
        Length::new::<millimeter>(2.0)
    } else {
        Length::new::<millimeter>(0.0)
    };
    let opening_height = Length::new::<millimeter>(1.0);

    // Ratio between the angle covered by a tooth and by a slot bottom
    let rat = 3.0;

    // Angle covered by one tooth
    let slot_angle = TAU / slots as f64;
    let alpha = slot_angle * 1.0 / (1.0 + rat);
    let beta = slot_angle - alpha;

    // Slot bottom width
    let bottom_width = 2.0 * yoke_radius * (beta / 2.0).sin();
    let yoke_height = yoke_radius * (1.0 - (beta / 2.0).cos());

    // Scale the air gap radius by the number of slots up to 0.9.
    // The scaling formula was created "by hand" to give a good visual
    // representation, the values are chosen arbitrarily and have no deeper meaning.
    let scale_air_gap = 0.1 + 0.8 * (1.0 - 1.0 / (slots as f64).sqrt());
    let air_gap_radius = yoke_radius * scale_air_gap;

    // Calculate the slot height
    let height = yoke_radius
        - yoke_height
        - 0.5 * (4.0 * air_gap_radius.powi(P2::new()) - opening_width.powi(P2::new())).sqrt();

    // Raise yoke_radius
    yoke_radius = yoke_radius + Length::new::<millimeter>(5.0);

    // Create slot and core object (they are just used for plotting purposes)
    let slot: SemiTrapezoidSlot = SemiTrapezoidWithoutSlopesBuilder {
        bottom_width,
        opening_width,
        height,
        opening_height,
        slot_angle,
        bottom_radius: Length::new::<millimeter>(2.0),
        top_radius: Length::new::<millimeter>(2.0),
        opening_radius: Length::new::<millimeter>(0.5),
        consider_tooth_tip_leakage: false,
    }
    .try_into()
    .unwrap();

    return RotCoreBuilder {
        air_gap_radius,
        yoke_radius,
        axial_length: Length::new::<millimeter>(1.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs,
        skew_angle: 0.0,
        air_gap: Box::new(SlottedAirGap::new(
            slots,
            starts_in_slot_middle,
            CarterFactorModel::Bin12,
            Box::new(slot),
        )),
        flux_barrier: None,
    }
    .try_into()
    .expect("valid magnetic core");
}

fn create_inner_core(
    slots: u16,
    pole_pairs: u16,
    starts_in_slot_middle: bool,
    open_slot: bool,
) -> RotCore {
    let yoke_radius = Length::new::<millimeter>(10.0);
    let air_gap_radius = Length::new::<millimeter>(25.0);

    let opening_width = if open_slot {
        Length::new::<millimeter>(2.0)
    } else {
        Length::new::<millimeter>(0.0)
    };
    let opening_height = Length::new::<millimeter>(1.0);

    // Angle covered by one tooth
    let slot_angle = -TAU / slots as f64;

    let bottom_width = Length::new::<millimeter>(4.0) * 12.0 / slots as f64;
    let height = Length::new::<millimeter>(10.0);

    // Create slot and core object (they are just used for plotting purposes)
    let slot: SemiTrapezoidSlot = SemiTrapezoidWithoutSlopesBuilder {
        bottom_width,
        opening_width,
        height,
        opening_height,
        slot_angle,
        bottom_radius: Length::new::<millimeter>(1.0),
        top_radius: Length::new::<millimeter>(1.0),
        opening_radius: Length::new::<millimeter>(0.5),
        consider_tooth_tip_leakage: false,
    }
    .try_into()
    .unwrap();

    return RotCoreBuilder {
        air_gap_radius,
        yoke_radius,
        axial_length: Length::new::<millimeter>(1.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs,
        skew_angle: 0.0,
        air_gap: Box::new(SlottedAirGap::new(
            slots,
            starts_in_slot_middle,
            CarterFactorModel::Bin12,
            Box::new(slot),
        )),
        flux_barrier: None,
    }
    .try_into()
    .expect("valid magnetic core");
}

fn create_outer_core_from_phd(model: CarterFactorModel) -> RotCore {
    let slot_angle = PI / 18.0;
    let bottom_width = Length::new::<millimeter>(9.2);
    let slot: SemiTrapezoidSlot = SemiTrapezoidWithoutSlopesBuilder {
        bottom_width,
        opening_width: Length::new::<millimeter>(2.0),
        height: Length::new::<millimeter>(17.75),
        opening_height: Length::new::<millimeter>(2.0),
        slot_angle,
        bottom_radius: Length::new::<millimeter>(2.0),
        top_radius: Length::new::<millimeter>(2.0),
        opening_radius: Length::new::<millimeter>(0.5),
        consider_tooth_tip_leakage: false,
    }
    .try_into()
    .unwrap();

    return RotCoreBuilder {
        air_gap_radius: Length::new::<millimeter>(55.0),
        yoke_radius: Length::new::<millimeter>(85.0),
        axial_length: Length::new::<millimeter>(165.0),
        axial_coil_overhang: Length::new::<millimeter>(0.0),
        iron_fill_factor: 1.0,
        material: Arc::new(Material::default()),
        pole_pairs: 2,
        skew_angle: 0.0,
        air_gap: Box::new(SlottedAirGap::new(36, false, model, Box::new(slot.clone()))),
        flux_barrier: None,
    }
    .try_into()
    .expect("valid magnetic core");
}

#[test]
fn test_assembly_check() {
    {
        let core = create_inner_core(12, 4, false, false);

        let magnet = ArcSegmentMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            core.air_gap_radius(),
            Length::new::<millimeter>(2.0),
            10.0 / 180.0 * PI,
            Arc::new(Default::default()),
        )
        .unwrap();
        let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());

        let core = create_inner_core(12, 4, false, true);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );

        let core = create_inner_core(12, 4, true, true);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
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

        let core = create_inner_core(12, 4, true, false);
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
        let core = create_outer_core(12, 4, false, false);

        let magnet = ArcSegmentMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            -core.air_gap_radius(),
            Length::new::<millimeter>(2.0),
            10.0 / 180.0 * PI,
            Arc::new(Default::default()),
        )
        .unwrap();
        let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());

        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );

        let core = create_outer_core(12, 4, true, false);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );

        let core = create_outer_core(12, 4, false, true);
        assert!(
            core.assembly_check(
                &CoilLayout::SingleFilled,
                Some(&assembly),
                DEFAULT_EPSILON,
                DEFAULT_MAX_RELATIVE
            )
            .is_ok()
        );

        let core = create_outer_core(12, 4, true, true);
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
fn test_plot_outer_core() {
    {
        let core = create_outer_core(12, 4, false, false);

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new(
            "tests/img/rot_slotted/outer_closed_starts_not_in_slot_middle.png",
        );
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
        let core = create_outer_core(12, 4, true, false);

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path =
            std::path::Path::new("tests/img/rot_slotted/outer_closed_starts_in_slot_middle.png");
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
        let core = create_outer_core(12, 4, false, true);

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path =
            std::path::Path::new("tests/img/rot_slotted/outer_open_starts_not_in_slot_middle.png");
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
        let core = create_outer_core(12, 4, true, true);

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path =
            std::path::Path::new("tests/img/rot_slotted/outer_open_starts_in_slot_middle.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        approx::assert_abs_diff_eq!(core.slot_opening_factor(1), 0.8, epsilon = 1e-6);
    }
}

#[test]
fn test_plot_outer_assembly() {
    {
        let core = create_outer_core(12, 4, false, false);

        let magnet = ArcSegmentMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            -core.air_gap_radius(),
            Length::new::<millimeter>(2.0),
            10.0 / 180.0 * PI,
            Arc::new(Default::default()),
        )
        .unwrap();
        let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());

        let drawable = core.drawable();

        let magnets: Vec<Drawable> = core
            .surface_magnets(&assembly, true)
            .map::<Drawable, _>(From::from)
            .collect();
        let mag_ref = &magnets;

        let winding_zones: Vec<Drawable> = core
            .winding_zones(&CoilLayout::DoubleHorizontal)
            .map(PositionedZoneContour::into_drawable)
            .collect();
        let winding_ref = &winding_zones;

        let view = Viewport::from_bounded_entities(
            mag_ref
                .iter()
                .map(|m| m.bounding_box())
                .chain(std::iter::once(drawable.bounding_box())),
            SideLength::Long(500),
        )
        .unwrap();

        let path = std::path::Path::new("tests/img/rot_slotted/outer_assembly_1.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in mag_ref {
                    m.draw(cr)?;
                }
                for w in winding_ref {
                    w.draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let core = create_outer_core(12, 4, false, true);

        let magnet = ArcSegmentMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            -core.air_gap_radius(),
            Length::new::<millimeter>(2.0),
            10.0 / 180.0 * PI,
            Arc::new(Default::default()),
        )
        .unwrap();
        let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());

        let drawable = core.drawable();

        let magnets: Vec<Drawable> = core
            .surface_magnets(&assembly, true)
            .map(PositionedMagnetShape::into_drawable)
            .collect();
        let mag_ref = &magnets;

        let winding_zones: Vec<Drawable> = core
            .winding_zones(&CoilLayout::DoubleHorizontal)
            .map(PositionedZoneContour::into_drawable)
            .collect();
        let winding_ref = &winding_zones;

        let view = Viewport::from_bounded_entities(
            mag_ref
                .iter()
                .map(|m| m.bounding_box())
                .chain(std::iter::once(drawable.bounding_box())),
            SideLength::Long(500),
        )
        .unwrap();

        let path = std::path::Path::new("tests/img/rot_slotted/outer_assembly_2.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in mag_ref {
                    m.draw(cr)?;
                }
                for w in winding_ref {
                    w.draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let core = create_outer_core(12, 4, false, true);

        let magnet = ArcSegmentMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            -core.air_gap_radius(),
            Length::new::<millimeter>(2.0),
            10.0 / 180.0 * PI,
            Arc::new(Default::default()),
        )
        .unwrap();
        let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());

        let drawable = core.drawable();

        let magnets: Vec<Drawable> = core
            .surface_magnets(&assembly, true)
            .map(PositionedMagnetShape::into_drawable)
            .collect();
        let mag_ref = &magnets;

        let winding_zones: Vec<Drawable> = core
            .winding_zones(&CoilLayout::SingleFilled)
            .map(PositionedZoneContour::into_drawable)
            .collect();
        let winding_ref = &winding_zones;

        let view = Viewport::from_bounded_entities(
            mag_ref
                .iter()
                .map(|m| m.bounding_box())
                .chain(std::iter::once(drawable.bounding_box())),
            SideLength::Long(500),
        )
        .unwrap();

        let path = std::path::Path::new("tests/img/rot_slotted/outer_assembly_3.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in mag_ref {
                    m.draw(cr)?;
                }
                for w in winding_ref {
                    w.draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
}

#[test]
fn test_plot_inner_core() {
    {
        let core = create_inner_core(12, 4, false, false);

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new(
            "tests/img/rot_slotted/inner_closed_starts_not_in_slot_middle.png",
        );
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
        let core = create_inner_core(12, 4, true, false);

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path =
            std::path::Path::new("tests/img/rot_slotted/inner_closed_starts_in_slot_middle.png");
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
        let core = create_inner_core(12, 4, false, true);

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path =
            std::path::Path::new("tests/img/rot_slotted/inner_open_starts_not_in_slot_middle.png");
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
        let core = create_inner_core(12, 4, true, true);

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path =
            std::path::Path::new("tests/img/rot_slotted/inner_open_starts_in_slot_middle.png");
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
fn test_plot_inner_assembly() {
    {
        let core = create_inner_core(12, 4, false, true);

        let drawable = core.drawable();

        let magnet = ArcSegmentMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            core.air_gap_radius(),
            Length::new::<millimeter>(2.0),
            10.0 / 180.0 * PI,
            Arc::new(Default::default()),
        )
        .unwrap();
        let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());
        let magnets: Vec<Drawable> = core
            .surface_magnets(&assembly, true)
            .map(PositionedMagnetShape::into_drawable)
            .collect();
        let mag_ref = &magnets;

        let winding_zones: Vec<Drawable> = core
            .winding_zones(&CoilLayout::DoubleHorizontal)
            .map(PositionedZoneContour::into_drawable)
            .collect();
        let winding_ref = &winding_zones;

        let view = Viewport::from_bounded_entities(
            mag_ref
                .iter()
                .map(|m| m.bounding_box())
                .chain(std::iter::once(drawable.bounding_box())),
            SideLength::Long(500),
        )
        .unwrap();

        let path = std::path::Path::new("tests/img/rot_slotted/inner_assembly_1.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in mag_ref {
                    m.draw(cr)?;
                }
                for w in winding_ref {
                    w.draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let core = create_inner_core(12, 4, false, false);

        let drawable = core.drawable();

        let magnet = ArcSegmentMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            core.air_gap_radius(),
            Length::new::<millimeter>(2.0),
            10.0 / 180.0 * PI,
            Arc::new(Default::default()),
        )
        .unwrap();
        let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());
        let magnets: Vec<Drawable> = core
            .surface_magnets(&assembly, true)
            .map(PositionedMagnetShape::into_drawable)
            .collect();
        let mag_ref = &magnets;

        let winding_zones: Vec<Drawable> = core
            .winding_zones(&CoilLayout::DoubleHorizontal)
            .map(PositionedZoneContour::into_drawable)
            .collect();
        let winding_ref = &winding_zones;

        let view = Viewport::from_bounded_entities(
            mag_ref
                .iter()
                .map(|m| m.bounding_box())
                .chain(std::iter::once(drawable.bounding_box())),
            SideLength::Long(500),
        )
        .unwrap();

        let path = std::path::Path::new("tests/img/rot_slotted/inner_assembly_2.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in mag_ref {
                    m.draw(cr)?;
                }
                for w in winding_ref {
                    w.draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let core = create_inner_core(12, 4, false, true);

        let drawable = core.drawable();

        let magnet = ArcSegmentMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            core.air_gap_radius(),
            Length::new::<millimeter>(2.0),
            10.0 / 180.0 * PI,
            Arc::new(Default::default()),
        )
        .unwrap();
        let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 3.try_into().unwrap());
        let magnets: Vec<Drawable> = core
            .surface_magnets(&assembly, true)
            .map(PositionedMagnetShape::into_drawable)
            .collect();
        let mag_ref = &magnets;

        let winding_zones: Vec<Drawable> = core
            .winding_zones(&CoilLayout::SingleFilled)
            .map(PositionedZoneContour::into_drawable)
            .collect();
        let winding_ref = &winding_zones;

        let view = Viewport::from_bounded_entities(
            mag_ref
                .iter()
                .map(|m| m.bounding_box())
                .chain(std::iter::once(drawable.bounding_box())),
            SideLength::Long(500),
        )
        .unwrap();

        let path = std::path::Path::new("tests/img/rot_slotted/inner_assembly_3.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in mag_ref {
                    m.draw(cr)?;
                }
                for w in winding_ref {
                    w.draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
}

#[test]
fn test_odd_number_of_slots() {
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
    .expect("valid slot");

    {
        let air_gap =
            SlottedAirGap::new(15, true, CarterFactorModel::Bin12, Box::new(slot.clone()));
        let core: RotCore = RotCoreBuilder {
            air_gap_radius: Length::new::<millimeter>(40.0),
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
        .try_into()
        .unwrap();

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new(
            "tests/img/rot_slotted/core_odd_number_of_slots_starts_in_slot_middle.png",
        );
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
        let air_gap = SlottedAirGap::new(15, false, CarterFactorModel::Bin12, Box::new(slot));
        let core: RotCore = RotCoreBuilder {
            air_gap_radius: Length::new::<millimeter>(40.0),
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
        .try_into()
        .unwrap();

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new(
            "tests/img/rot_slotted/core_odd_number_of_slots_starts_in_tooth_middle.png",
        );
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
fn test_slotting_ordinals() {
    {
        let core = create_outer_core(12, 4, true, true);
        let mut iter = core.slotting_ordinals();
        assert_eq!(iter.next(), Some(num::rational::Ratio::new(3, 1)));
        assert_eq!(iter.next(), Some(num::rational::Ratio::new(6, 1)));
        assert_eq!(iter.next(), Some(num::rational::Ratio::new(9, 1)));
        assert_eq!(iter.next(), Some(num::rational::Ratio::new(12, 1)));
    }

    {
        let core = create_outer_core(12, 5, true, true);
        let mut iter = core.slotting_ordinals();
        assert_eq!(iter.next(), Some(num::rational::Ratio::new(12, 5)));
        assert_eq!(iter.next(), Some(num::rational::Ratio::new(24, 5)));
        assert_eq!(iter.next(), Some(num::rational::Ratio::new(36, 5)));
        assert_eq!(iter.next(), Some(num::rational::Ratio::new(48, 5)));
    }
}

#[test]
fn test_dimensions_and_mass() {
    let core = create_outer_core_from_phd(CarterFactorModel::Bin12);

    // Stack
    approx::assert_abs_diff_eq!(core.axial_length().get::<meter>(), 0.165, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(core.iron_length().get::<meter>(), 0.165, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(
        core.air_gap_length().get::<meter>(),
        core.air_gap_radius().get::<meter>() * TAU,
        epsilon = 1e-6
    );

    // Offset
    approx::assert_abs_diff_eq!(
        core.origin_offset_core_to_slot().get::<meter>(),
        0.05499,
        epsilon = 1e-3
    );

    // Check the tooth dimensions (values taken from [Mat19])
    approx::assert_abs_diff_eq!(core.tooth_height().get::<meter>(), 17.75e-3, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(
        core.tooth_width_at(Length::new::<millimeter>(5.0))
            .get::<meter>(),
        0.003514,
        epsilon = 1e-6
    );

    // Check the mass of yoke and teeth
    approx::assert_abs_diff_eq!(
        core.tooth_mass().get::<kilogram>(),
        0.010293,
        epsilon = 1e-6
    );
    approx::assert_abs_diff_eq!(
        core.teeth_mass().get::<kilogram>(),
        core.tooth_mass().get::<kilogram>() * core.slots() as f64,
        epsilon = 1e-6
    );
    approx::assert_abs_diff_eq!(core.yoke_mass().get::<kilogram>(), 1.07023, epsilon = 1e-5);
}

#[test]
fn test_carter_factor() {
    let core = create_outer_core_from_phd(CarterFactorModel::Bin12);
    approx::assert_abs_diff_eq!(
        core.carter_factor(Length::new::<millimeter>(1.0)),
        1.063296,
        epsilon = 0.001
    );

    let core = create_outer_core_from_phd(CarterFactorModel::MVP08);
    approx::assert_abs_diff_eq!(
        core.carter_factor(Length::new::<millimeter>(1.0)),
        1.030677,
        epsilon = 0.001
    );

    let core = create_outer_core_from_phd(CarterFactorModel::PS62);
    approx::assert_abs_diff_eq!(
        core.carter_factor(Length::new::<millimeter>(1.0)),
        1.113640,
        epsilon = 0.001
    );
}

#[test]
fn test_current_displacement_coefficients() {
    let core = create_outer_core_from_phd(CarterFactorModel::PS62);

    let coeffs = core.current_displacement_coefficients().eval(
        Frequency::new::<uom::si::frequency::hertz>(200.0),
        ElectricalConductivity::new::<siemens_per_meter>(57e6),
        1.0,
    );
    approx::assert_abs_diff_eq!(coeffs.resistance, 4.839075, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(coeffs.inductance, 0.659253, epsilon = 1e-3);
}
