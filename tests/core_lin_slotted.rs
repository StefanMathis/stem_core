use std::sync::Arc;

use cairo_viewport::bounding_box::ToBoundingBox;
use cairo_viewport::{SideLength, Viewport, compare_or_create};
use stem_core::prelude::*;
use stem_core::stem_slot::semi_trapezoid::SemiTrapezoidWithoutSlopesBuilder;
use stem_slot::planar_geo::draw::Drawable;
use stem_slot::planar_geo::prelude::*;

fn create_core(starts_in_slot_middle: bool, open_slot: bool) -> LinCore {
    let opening_width = if open_slot {
        Length::new::<millimeter>(2.0)
    } else {
        Length::new::<millimeter>(0.0)
    };

    let slot: SemiTrapezoidSlot = SemiTrapezoidWithoutSlopesBuilder {
        bottom_width: Length::new::<millimeter>(8.0),
        opening_width,
        height: Length::new::<millimeter>(17.75),
        opening_height: Length::new::<millimeter>(0.75),
        slot_angle: 0.0,
        bottom_radius: Length::new::<millimeter>(3.0),
        top_radius: Length::new::<millimeter>(2.0),
        opening_radius: Length::new::<millimeter>(0.0),
        consider_tooth_tip_leakage: true,
    }
    .try_into()
    .unwrap();

    let air_gap = SlottedAirGap::new(
        12,
        starts_in_slot_middle,
        CarterFactorModel::Bin12,
        Box::new(slot),
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
fn test_read_properties() {
    let core = create_core(false, true);

    approx::assert_abs_diff_eq!(core.yoke_height().get::<millimeter>(), 7.25, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(core.height().get::<millimeter>(), 25.0, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(
        core.tooth_height().get::<millimeter>(),
        25.0 - 7.25,
        epsilon = 1e-6
    );
    approx::assert_abs_diff_eq!(core.yoke_height().get::<millimeter>(), 7.25, epsilon = 1e-6);
}

#[test]
fn test_assembly_check() {
    let magnet = BreadLoafMagnet::new(
        Length::new::<millimeter>(165.0),
        Length::new::<millimeter>(16.0),
        Length::new::<millimeter>(4.0),
        Length::new::<millimeter>(30.0),
        Arc::new(Default::default()),
    )
    .unwrap();
    let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 2.try_into().unwrap());

    let core = create_core(true, true);
    assert!(
        core.assembly_check(
            &CoilLayout::SingleFilled,
            Some(&assembly),
            DEFAULT_EPSILON,
            DEFAULT_MAX_RELATIVE
        )
        .is_ok()
    );

    let core = create_core(false, true);
    assert!(
        core.assembly_check(
            &CoilLayout::SingleFilled,
            Some(&assembly),
            DEFAULT_EPSILON,
            DEFAULT_MAX_RELATIVE
        )
        .is_ok()
    );

    let core = create_core(true, false);
    assert!(
        core.assembly_check(
            &CoilLayout::SingleFilled,
            Some(&assembly),
            DEFAULT_EPSILON,
            DEFAULT_MAX_RELATIVE
        )
        .is_ok()
    );

    let core = create_core(false, false);
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
fn test_plot_core() {
    {
        let core = create_core(false, true);

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/lin_slotted/open_starts_not_in_slot_middle.png");
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
        let core = create_core(true, true);

        let drawable = core.drawable();
        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/lin_slotted/open_starts_in_slot_middle.png");
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
        Length::new::<millimeter>(16.0),
        Length::new::<millimeter>(4.0),
        Length::new::<millimeter>(30.0),
        Arc::new(Default::default()),
    )
    .unwrap();
    let assembly = MagnetAssembly::new(magnet, 1.try_into().unwrap(), 2.try_into().unwrap());

    {
        let core = create_core(false, true);

        let drawable = core.drawable();
        let magnets: Vec<Drawable> = core
            .surface_magnets(&assembly, true)
            .map(From::from)
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
        let zones: Vec<Drawable> = core
            .winding_zones(&CoilLayout::Quadruple)
            .map(PositionedZoneContour::into_drawable)
            .collect();

        let path = std::path::Path::new("tests/img/lin_slotted/assembly_1.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in mag_ref {
                    m.draw(cr)?;
                }
                for z in &zones {
                    z.draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());
    }
    {
        let core = create_core(true, true);

        let drawable = core.drawable();
        let magnets: Vec<Drawable> = core
            .surface_magnets(&assembly, true)
            .map(From::from)
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
        let zones: Vec<Drawable> = core
            .winding_zones(&CoilLayout::DoubleHorizontal)
            .map(PositionedZoneContour::into_drawable)
            .collect();

        let path = std::path::Path::new("tests/img/lin_slotted/assembly_2.png");
        let callback = |path: &std::path::Path| {
            return view.write_to_file(path, |cr| {
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.paint()?;
                drawable.draw(cr)?;
                for m in mag_ref {
                    m.draw(cr)?;
                }
                for z in &zones {
                    z.draw(cr)?;
                }
                return Ok(());
            });
        };
        assert!(compare_or_create(path, &callback, 0.98).is_ok());

        approx::assert_abs_diff_eq!(core.slot_opening_factor(1), 0.8, epsilon = 1e-6);
    }
}

#[test]
fn test_closed_slots() {
    {
        let core = create_core(true, false);

        let drawable = core.drawable();

        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path = std::path::Path::new("tests/img/lin_slotted/closed_starts_in_slot_middle.png");
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
        let core = create_core(false, false);

        let drawable = core.drawable();

        let view = Viewport::from_bounded_entity(&drawable, SideLength::Long(500));
        let path =
            std::path::Path::new("tests/img/lin_slotted/closed_starts_not_in_slot_middle.png");
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
fn test_current_displacement_coefficients() {
    let core = create_core(false, false);
    let coeffs = core.current_displacement_coefficients().eval(
        Frequency::new::<uom::si::frequency::hertz>(50.0),
        ElectricalConductivity::new::<siemens_per_meter>(57e6),
        1.0,
    );
    approx::assert_abs_diff_eq!(coeffs.resistance, 1.603643, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(coeffs.inductance, 0.855859, epsilon = 1e-3);

    let coeffs = core.current_displacement_coefficients().eval(
        Frequency::new::<uom::si::frequency::hertz>(200.0),
        ElectricalConductivity::new::<siemens_per_meter>(57e6),
        1.0,
    );
    approx::assert_abs_diff_eq!(coeffs.resistance, 3.581551, epsilon = 1e-6);
    approx::assert_abs_diff_eq!(coeffs.inductance, 0.503458, epsilon = 1e-3);
}

#[test]
fn test_tooth_width_at() {
    let core = create_core(false, true);
    approx::assert_abs_diff_eq!(
        core.tooth_width_at(Length::new::<millimeter>(0.0))
            .get::<millimeter>(),
        10.5,
        epsilon = 1e-6
    );
    approx::assert_abs_diff_eq!(
        core.tooth_width_at(Length::new::<millimeter>(1.0))
            .get::<millimeter>(),
        6.563508,
        epsilon = 1e-6
    );
    approx::assert_abs_diff_eq!(
        core.tooth_width_at(Length::new::<millimeter>(12.0))
            .get::<millimeter>(),
        4.5,
        epsilon = 1e-6
    );
    approx::assert_abs_diff_eq!(
        core.tooth_width_at(Length::new::<millimeter>(18.0))
            .get::<millimeter>(),
        12.5,
        epsilon = 1e-6
    );
    approx::assert_abs_diff_eq!(
        core.tooth_width_at(Length::new::<millimeter>(50.0))
            .get::<millimeter>(),
        12.5,
        epsilon = 1e-6
    );
}
