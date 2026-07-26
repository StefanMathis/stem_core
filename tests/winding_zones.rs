use std::f64::consts::TAU;

use cairo_viewport::{SideLength, Viewport, compare_or_create};
use planar_geo::{contour::Contour, draw::Style};
use stem_core::winding_zones::WindingZonesEqSpaced;
use stem_slot::{
    planar_geo::{draw::*, prelude::Composite},
    prelude::*,
};

fn compare_to_reference<P: AsRef<std::path::Path>>(
    contours: Vec<Contour>,
    path: P,
    view: Option<Viewport>,
) {
    let view = view.unwrap_or(
        Viewport::from_bounded_entities(contours.iter(), SideLength::Long(500)).unwrap(),
    );
    let mut style = Style::default();
    style.background_color = stem_slot::ORANGE;

    let callback = |path: &std::path::Path| {
        return view.write_to_file(path, move |cr| {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.paint()?;
            for (idx, contour) in contours.iter().enumerate() {
                contour.draw(&style, cr)?;
                let text = Text {
                    text: idx.to_string(),
                    anchor: Anchor::Center,
                    fixed_anchor_offset: [0.0, 0.0],
                    scaled_anchor_offset: contour.centroid(),
                    color: Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                    font_size: 10.0,
                    angle: 0.0,
                };
                text.draw(cr)?;
            }
            return Ok(());
        });
    };
    assert!(compare_or_create(path, callback, 0.99).is_ok());
}

#[test]
fn from_slot_rot_outer() {
    let radius = Length::new::<millimeter>(55.0);
    let slot = RectangularSlot::new(
        Length::new::<millimeter>(8.0),
        Length::new::<millimeter>(4.0),
        Length::new::<millimeter>(20.0),
        Length::new::<millimeter>(2.0),
        false,
    )
    .unwrap();

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, false>::from_slot(
            radius * TAU,
            36,
            &slot,
            &CoilLayout::DoubleHorizontal,
            false,
            true,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_rot_outer_1.png"),
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, false>::from_slot(
            radius * TAU,
            36,
            &slot,
            &CoilLayout::DoubleHorizontal,
            true,
            true,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_rot_outer_2.png"),
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, false>::from_slot(
            radius * TAU,
            36,
            &slot,
            &CoilLayout::Quadruple,
            true,
            true,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_rot_outer_3.png"),
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, false>::from_slot(
            radius * TAU,
            36,
            &slot,
            &CoilLayout::SingleFilled,
            true,
            true,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_rot_outer_4.png"),
        None,
    );

    for starts_in_slot_middle in [false, true] {
        let slot_contour = WindingZonesEqSpaced::<Contour, false>::from_slot(
            radius * TAU,
            36,
            &slot,
            &CoilLayout::SingleFilled,
            starts_in_slot_middle,
            true,
        )
        .next()
        .unwrap();
        let slot_opening_pt = slot_contour.points().next().unwrap();
        approx::assert_abs_diff_eq!(
            (slot_opening_pt[0].powi(2) + slot_opening_pt[1].powi(2)).sqrt(),
            radius.get::<meter>()
        );
    }
}

#[test]
fn from_slot_rot_inner() {
    let radius = Length::new::<millimeter>(75.0);
    let slot = RectangularSlot::new(
        Length::new::<millimeter>(8.0),
        Length::new::<millimeter>(4.0),
        Length::new::<millimeter>(20.0),
        Length::new::<millimeter>(2.0),
        false,
    )
    .unwrap();

    for starts_in_slot_middle in [false, true] {
        let slot_contour = WindingZonesEqSpaced::<Contour, false>::from_slot(
            radius * TAU,
            36,
            &slot,
            &CoilLayout::SingleFilled,
            starts_in_slot_middle,
            false,
        )
        .next()
        .unwrap();
        let slot_opening_pt = slot_contour.points().next().unwrap();
        approx::assert_abs_diff_eq!(
            (slot_opening_pt[0].powi(2) + slot_opening_pt[1].powi(2)).sqrt(),
            radius.get::<meter>()
        );
    }

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, false>::from_slot(
            radius * TAU,
            36,
            &slot,
            &CoilLayout::SingleFilled,
            false,
            false,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_rot_inner_1.png"),
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, false>::from_slot(
            radius * TAU,
            36,
            &slot,
            &CoilLayout::DoubleHorizontal,
            false,
            false,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_rot_inner_2.png"),
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, false>::from_slot(
            radius * TAU,
            36,
            &slot,
            &CoilLayout::DoubleHorizontal,
            true,
            false,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_rot_inner_3.png"),
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, false>::from_slot(
            radius * TAU,
            36,
            &slot,
            &CoilLayout::Quadruple,
            true,
            false,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_rot_inner_4.png"),
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, false>::from_slot(
            radius * TAU,
            36,
            &slot,
            &CoilLayout::Single,
            true,
            false,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_rot_inner_5.png"),
        None,
    );
}

#[test]
fn from_slot_lin_outer() {
    let width = Length::new::<millimeter>(200.0);
    let slot = RectangularSlot::new(
        Length::new::<millimeter>(8.0),
        Length::new::<millimeter>(4.0),
        Length::new::<millimeter>(20.0),
        Length::new::<millimeter>(2.0),
        false,
    )
    .unwrap();

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, true>::from_slot(
            width,
            12,
            &slot,
            &CoilLayout::DoubleHorizontal,
            false,
            true,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_lin_outer_1.png"),
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, true>::from_slot(
            width,
            12,
            &slot,
            &CoilLayout::DoubleHorizontal,
            true,
            true,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_lin_outer_2.png"),
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, true>::from_slot(
            width,
            12,
            &slot,
            &CoilLayout::Quadruple,
            true,
            true,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_lin_outer_3.png"),
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, true>::from_slot(
            width,
            12,
            &slot,
            &CoilLayout::SingleFilled,
            true,
            true,
        )
        .collect(),
        format!("tests/img/winding_zones/slot_lin_outer_4.png"),
        None,
    );
}

#[test]
fn from_air_gap_winding_lin_outer() {
    let width = Length::new::<millimeter>(200.0);
    let air_gap_winding_height = Length::new::<millimeter>(10.0);
    let winding_coverage = 0.8;

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, true>::from_air_gap_winding(
            width,
            12,
            air_gap_winding_height,
            winding_coverage,
            &CoilLayout::Single,
            false,
            true,
        )
        .collect(),
        "tests/img/winding_zones/ag_winding_lin_outer_1.png",
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, true>::from_air_gap_winding(
            width,
            12,
            air_gap_winding_height,
            winding_coverage,
            &CoilLayout::DoubleHorizontal,
            false,
            true,
        )
        .collect(),
        "tests/img/winding_zones/ag_winding_lin_outer_2.png",
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, true>::from_air_gap_winding(
            width,
            12,
            air_gap_winding_height,
            winding_coverage,
            &CoilLayout::DoubleVertical,
            false,
            true,
        )
        .collect(),
        "tests/img/winding_zones/ag_winding_lin_outer_3.png",
        None,
    );

    let mut contours: Vec<Contour> = WindingZonesEqSpaced::<Contour, true>::from_air_gap_winding(
        width,
        12,
        air_gap_winding_height,
        winding_coverage,
        &CoilLayout::Quadruple,
        false,
        true,
    )
    .collect();
    contours.append(
        &mut WindingZonesEqSpaced::<Contour, true>::from_air_gap_winding(
            width,
            12,
            air_gap_winding_height,
            1.0,
            &CoilLayout::MultiVertical(3),
            true,
            false,
        )
        .collect(),
    );

    compare_to_reference(
        contours,
        "tests/img/winding_zones/ag_winding_lin_outer_4.png",
        None,
    );
}

#[test]
fn from_air_gap_winding_rot_outer() {
    let width = Length::new::<millimeter>(200.0);
    let air_gap_winding_height = Length::new::<millimeter>(10.0);
    let winding_coverage = 0.8;

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, false>::from_air_gap_winding(
            width,
            12,
            air_gap_winding_height,
            winding_coverage,
            &CoilLayout::Single,
            false,
            true,
        )
        .collect(),
        "tests/img/winding_zones/ag_winding_rot_outer_1.png",
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, false>::from_air_gap_winding(
            width,
            12,
            air_gap_winding_height,
            winding_coverage,
            &CoilLayout::Single,
            true,
            true,
        )
        .collect(),
        "tests/img/winding_zones/ag_winding_rot_outer_2.png",
        None,
    );

    compare_to_reference(
        WindingZonesEqSpaced::<Contour, false>::from_air_gap_winding(
            width,
            12,
            air_gap_winding_height,
            winding_coverage,
            &CoilLayout::DoubleHorizontal,
            true,
            true,
        )
        .collect(),
        "tests/img/winding_zones/ag_winding_rot_outer_3.png",
        None,
    );

    let mut contours: Vec<Contour> = WindingZonesEqSpaced::<Contour, false>::from_air_gap_winding(
        width,
        12,
        air_gap_winding_height,
        winding_coverage,
        &CoilLayout::DoubleVertical,
        true,
        false,
    )
    .collect();
    contours.append(
        &mut WindingZonesEqSpaced::<Contour, false>::from_air_gap_winding(
            width,
            12,
            air_gap_winding_height,
            winding_coverage,
            &CoilLayout::DoubleVertical,
            false,
            true,
        )
        .collect(),
    );

    compare_to_reference(
        contours,
        "tests/img/winding_zones/ag_winding_rot_outer_4.png",
        None,
    );

    let mut contours: Vec<Contour> = WindingZonesEqSpaced::<Contour, false>::from_air_gap_winding(
        width,
        12,
        air_gap_winding_height,
        winding_coverage,
        &CoilLayout::Quadruple,
        true,
        false,
    )
    .collect();
    contours.append(
        &mut WindingZonesEqSpaced::<Contour, false>::from_air_gap_winding(
            width,
            12,
            air_gap_winding_height,
            winding_coverage,
            &CoilLayout::Quadruple,
            false,
            true,
        )
        .collect(),
    );

    compare_to_reference(
        contours,
        "tests/img/winding_zones/ag_winding_rot_outer_5.png",
        None,
    );

    let mut contours: Vec<Contour> = WindingZonesEqSpaced::<Contour, false>::from_air_gap_winding(
        width,
        12,
        air_gap_winding_height,
        winding_coverage,
        &CoilLayout::MultiVertical(3),
        true,
        false,
    )
    .collect();
    contours.append(
        &mut WindingZonesEqSpaced::<Contour, false>::from_air_gap_winding(
            width,
            12,
            air_gap_winding_height,
            winding_coverage,
            &CoilLayout::MultiVertical(5),
            false,
            true,
        )
        .collect(),
    );

    compare_to_reference(
        contours,
        "tests/img/winding_zones/ag_winding_rot_outer_6.png",
        None,
    );
}
