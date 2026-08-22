use std::{
    f64::consts::{FRAC_PI_2, PI, TAU},
    sync::Arc,
};

use cairo_viewport::{SideLength, Viewport, compare_or_create};
use planar_geo::{
    draw::*,
    prelude::{BoundingBox, ToBoundingBox},
};
use stem_core::{
    magnets::{MagnetsEqSpaced, pole_coverage_angle},
    prelude::{surface_magnet_assembly_shapes_lin, surface_magnet_assembly_shapes_rot},
};
use stem_magnet::{
    arc::{AngleOrWidth, ArcParallelMagnet, ArcSegmentMagnet, SideHeightOrThickness},
    assembly::MagnetAssembly,
    bread_loaf::BreadLoafMagnet,
    magnet::Magnet,
};
use stem_slot::prelude::*;

fn compare_to_reference<P: AsRef<std::path::Path>>(
    drawables: Vec<Drawable>,
    path: P,
    view: Option<Viewport>,
) {
    let view = view.unwrap_or(
        Viewport::from_bounded_entities(drawables.iter(), SideLength::Long(500)).unwrap(),
    );

    let callback = |path: &std::path::Path| {
        return view.write_to_file(path, move |cr| {
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.paint()?;
            for (idx, drawable) in drawables.iter().enumerate() {
                drawable.draw(cr)?;
                let text = Text {
                    text: idx.to_string(),
                    anchor: Anchor::Center,
                    fixed_anchor_offset: [0.0, 0.0],
                    scaled_anchor_offset: drawable.bounding_box().center(),
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
fn test_lin() {
    let magnet = BreadLoafMagnet::new(
        Length::new::<millimeter>(165.0),
        Length::new::<millimeter>(20.0),
        Length::new::<millimeter>(10.0),
        Length::new::<millimeter>(50.0),
        Arc::new(Default::default()),
    )
    .unwrap();

    let shapes = surface_magnet_assembly_shapes_lin(
        &MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap()),
        false,
        None,
    );
    compare_to_reference(
        MagnetsEqSpaced::<true>::new(Length::new::<millimeter>(400.0), shapes, 4, FRAC_PI_2)
            .map(From::from)
            .collect(),
        "tests/img/magnets/smooth_lin_1.png",
        None,
    );

    let shapes = surface_magnet_assembly_shapes_lin(
        &MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 2.try_into().unwrap()),
        false,
        None,
    );
    compare_to_reference(
        MagnetsEqSpaced::<true>::new(Length::new::<millimeter>(400.0), shapes, 6, FRAC_PI_2)
            .map(From::from)
            .collect(),
        "tests/img/magnets/smooth_lin_2.png",
        None,
    );

    let shapes = surface_magnet_assembly_shapes_lin(
        &MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 2.try_into().unwrap()),
        true,
        None,
    );
    compare_to_reference(
        MagnetsEqSpaced::<true>::new(Length::new::<millimeter>(400.0), shapes, 6, FRAC_PI_2)
            .map(From::from)
            .collect(),
        "tests/img/magnets/smooth_lin_3.png",
        None,
    );
}

#[test]
fn rot_inner() {
    {
        let magnet = ArcParallelMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            Length::new::<millimeter>(85.0),
            SideHeightOrThickness::Thickness(Length::new::<millimeter>(10.0)),
            AngleOrWidth::Angle(10.0 / 180.0 * PI),
            Arc::new(Default::default()),
        )
        .unwrap();

        let shapes = surface_magnet_assembly_shapes_rot(
            &MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap()),
            true,
            Length::new::<millimeter>(85.0),
            false,
            None,
        );
        compare_to_reference(
            MagnetsEqSpaced::<false>::new(
                Length::new::<millimeter>(85.0) * TAU,
                shapes.clone(),
                4,
                FRAC_PI_2,
            )
            .map(From::from)
            .collect(),
            "tests/img/magnets/smooth_rot_inner_1.png",
            None,
        );

        let bb = BoundingBox::from_bounded_entities(
            MagnetsEqSpaced::<false>::new(
                Length::new::<millimeter>(85.0) * TAU,
                shapes,
                4,
                FRAC_PI_2,
            )
            .map(|s| s.shape.bounding_box()),
        )
        .unwrap();
        approxim::assert_abs_diff_eq!(bb.xmin(), -0.081831, epsilon = 1e-6);
        approxim::assert_abs_diff_eq!(bb.xmax(), 0.081831, epsilon = 1e-6);
        approxim::assert_abs_diff_eq!(bb.ymin(), -0.081831, epsilon = 1e-6);
        approxim::assert_abs_diff_eq!(bb.ymax(), 0.081831, epsilon = 1e-6);
    }
    {
        let magnet = ArcSegmentMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            Length::new::<millimeter>(85.0),
            Length::new::<millimeter>(10.0),
            10.0 / 180.0 * PI,
            Arc::new(Default::default()),
        )
        .unwrap();

        let shapes = surface_magnet_assembly_shapes_rot(
            &MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap()),
            true,
            Length::new::<millimeter>(85.0),
            false,
            None,
        );
        compare_to_reference(
            MagnetsEqSpaced::<false>::new(
                Length::new::<millimeter>(85.0) * TAU,
                shapes.clone(),
                4,
                FRAC_PI_2,
            )
            .map(From::from)
            .collect(),
            "tests/img/magnets/smooth_rot_inner_2.png",
            None,
        );

        let bb = BoundingBox::from_bounded_entities(
            MagnetsEqSpaced::<false>::new(
                Length::new::<millimeter>(85.0) * TAU,
                shapes,
                4,
                FRAC_PI_2,
            )
            .map(|s| s.shape.bounding_box()),
        )
        .unwrap();
        approxim::assert_abs_diff_eq!(bb.xmin(), -0.082272, epsilon = 1e-6);
        approxim::assert_abs_diff_eq!(bb.xmax(), 0.082272, epsilon = 1e-6);
        approxim::assert_abs_diff_eq!(bb.ymin(), -0.082272, epsilon = 1e-6);
        approxim::assert_abs_diff_eq!(bb.ymax(), 0.082272, epsilon = 1e-6);
    }
    {
        let magnet = BreadLoafMagnet::with_center_thickness(
            Length::new::<millimeter>(165.0),
            Length::new::<millimeter>(16.0),
            Length::new::<millimeter>(10.0),
            Length::new::<millimeter>(12.0),
            Arc::new(Default::default()),
        )
        .unwrap();

        let shapes = surface_magnet_assembly_shapes_rot(
            &MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap()),
            true,
            Length::new::<millimeter>(85.0),
            false,
            None,
        );
        compare_to_reference(
            MagnetsEqSpaced::<false>::new(
                Length::new::<millimeter>(85.0) * TAU,
                shapes.clone(),
                4,
                FRAC_PI_2,
            )
            .map(From::from)
            .collect(),
            "tests/img/magnets/smooth_rot_inner_3.png",
            None,
        );
    }
}

#[test]
fn rot_outer() {
    {
        let magnet = ArcParallelMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            -Length::new::<millimeter>(85.0),
            SideHeightOrThickness::Thickness(Length::new::<millimeter>(10.0)),
            AngleOrWidth::Angle(10.0 / 180.0 * PI),
            Arc::new(Default::default()),
        )
        .unwrap();

        let shapes = surface_magnet_assembly_shapes_rot(
            &MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap()),
            true,
            Length::new::<millimeter>(85.0),
            true,
            None,
        );
        compare_to_reference(
            MagnetsEqSpaced::<false>::new(
                Length::new::<millimeter>(85.0) * TAU,
                shapes.clone(),
                4,
                FRAC_PI_2,
            )
            .map(From::from)
            .collect(),
            "tests/img/magnets/smooth_rot_outer_1.png",
            None,
        );

        let bb = BoundingBox::from_bounded_entities(
            MagnetsEqSpaced::<false>::new(
                Length::new::<millimeter>(85.0) * TAU,
                shapes.clone(),
                4,
                FRAC_PI_2,
            )
            .map(|s| s.shape.bounding_box()),
        )
        .unwrap();
        approxim::assert_abs_diff_eq!(bb.xmin(), -0.074584, epsilon = 1e-6);
        approxim::assert_abs_diff_eq!(bb.xmax(), 0.074584, epsilon = 1e-6);
        approxim::assert_abs_diff_eq!(bb.ymin(), -0.074584, epsilon = 1e-6);
        approxim::assert_abs_diff_eq!(bb.ymax(), 0.074584, epsilon = 1e-6);
    }
    {
        let magnet = ArcSegmentMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            -Length::new::<millimeter>(85.0),
            Length::new::<millimeter>(10.0),
            10.0 / 180.0 * PI,
            Arc::new(Default::default()),
        )
        .unwrap();

        let shapes = surface_magnet_assembly_shapes_rot(
            &MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap()),
            true,
            Length::new::<millimeter>(85.0),
            true,
            None,
        );
        compare_to_reference(
            MagnetsEqSpaced::<false>::new(
                Length::new::<millimeter>(85.0) * TAU,
                shapes.clone(),
                4,
                FRAC_PI_2,
            )
            .map(From::from)
            .collect(),
            "tests/img/magnets/smooth_rot_outer_2.png",
            None,
        );

        let bb = BoundingBox::from_bounded_entities(
            MagnetsEqSpaced::<false>::new(
                Length::new::<millimeter>(85.0) * TAU,
                shapes.clone(),
                4,
                FRAC_PI_2,
            )
            .map(|s| s.shape.bounding_box()),
        )
        .unwrap();
        approxim::assert_abs_diff_eq!(bb.xmin(), -0.073612, epsilon = 1e-6);
        approxim::assert_abs_diff_eq!(bb.xmax(), 0.073612, epsilon = 1e-6);
        approxim::assert_abs_diff_eq!(bb.ymin(), -0.073612, epsilon = 1e-6);
        approxim::assert_abs_diff_eq!(bb.ymax(), 0.073612, epsilon = 1e-6);
    }
    {
        let magnet = BreadLoafMagnet::with_center_thickness(
            Length::new::<millimeter>(165.0),
            Length::new::<millimeter>(16.0),
            Length::new::<millimeter>(10.0),
            Length::new::<millimeter>(12.0),
            Arc::new(Default::default()),
        )
        .unwrap();

        let shapes = surface_magnet_assembly_shapes_rot(
            &MagnetAssembly::new(magnet.clone(), 1.try_into().unwrap(), 3.try_into().unwrap()),
            true,
            Length::new::<millimeter>(85.0),
            true,
            None,
        );
        compare_to_reference(
            MagnetsEqSpaced::<false>::new(
                Length::new::<millimeter>(85.0) * TAU,
                shapes.clone(),
                4,
                FRAC_PI_2,
            )
            .map(From::from)
            .collect(),
            "tests/img/magnets/smooth_rot_outer_3.png",
            None,
        );
    }
}

#[test]
fn test_pole_coverage() {
    {
        let magnet = ArcParallelMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            Length::new::<millimeter>(85.0),
            SideHeightOrThickness::Thickness(Length::new::<millimeter>(10.0)),
            AngleOrWidth::Angle(10.0 / 180.0 * PI),
            Arc::new(Default::default()),
        )
        .unwrap();
        let shapes = magnet.north_south_shapes().map(|s| s.into_owned());
        let a = pole_coverage_angle(
            shapes.iter(),
            magnet.core_radius().get::<meter>(),
            Length::new::<millimeter>(0.0),
        );
        approxim::assert_abs_diff_eq!(a, magnet.angle(), epsilon = 1e-8);
    }
    {
        let magnet = ArcSegmentMagnet::with_const_thickness(
            Length::new::<millimeter>(165.0),
            -Length::new::<millimeter>(85.0),
            Length::new::<millimeter>(10.0),
            10.0 / 180.0 * PI,
            Arc::new(Default::default()),
        )
        .unwrap();
        let shapes = magnet.north_south_shapes().map(|s| s.into_owned());
        let a = pole_coverage_angle(
            shapes.iter(),
            magnet.core_radius().get::<meter>(),
            Length::new::<millimeter>(0.0),
        );
        approxim::assert_abs_diff_eq!(a, magnet.angle(), epsilon = 1e-8);
    }
}
