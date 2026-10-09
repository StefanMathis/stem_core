use std::f64::consts::{PI, TAU};
use std::num::NonZeroU16;

use approxim::assert_abs_diff_eq;
use stem_core::air_gap::slot_opening_factor;
use stem_core::core::ext::skew_factor;
use stem_core::stem_material::prelude::*;
use stem_slot::prelude::SpatialOrder;

#[test]
fn test_skew_factor_no_segment() {
    approxim::assert_abs_diff_eq!(
        skew_factor(
            6.0 / 180.0 * PI,
            0,
            NonZeroU16::MIN,
            SpatialOrder::Mechanical(60)
        ),
        0.0,
        epsilon = 0.0001
    );
    approxim::assert_abs_diff_eq!(
        skew_factor(
            12.0 / 180.0 * PI,
            0,
            NonZeroU16::MIN,
            SpatialOrder::Mechanical(30)
        ),
        0.0,
        epsilon = 0.0001
    );
    approxim::assert_abs_diff_eq!(
        skew_factor(
            12.0 / 180.0 * PI,
            0,
            NonZeroU16::MIN,
            SpatialOrder::Mechanical(60)
        ),
        0.0,
        epsilon = 0.0001
    );
    approxim::assert_abs_diff_eq!(
        skew_factor(
            12.0 / 180.0 * PI,
            0,
            NonZeroU16::MIN,
            SpatialOrder::Mechanical(90)
        ),
        0.0,
        epsilon = 0.0001
    );
    approxim::assert_abs_diff_eq!(
        skew_factor(
            12.0 / 180.0 * PI,
            0,
            NonZeroU16::MIN,
            SpatialOrder::Mechanical(120)
        ),
        0.0,
        epsilon = 0.0001
    );
    approxim::assert_abs_diff_eq!(
        skew_factor(
            12.0 / 180.0 * PI,
            0,
            NonZeroU16::MIN,
            SpatialOrder::Mechanical(150)
        ),
        0.0,
        epsilon = 0.0001
    );
    approxim::assert_abs_diff_eq!(
        skew_factor(
            12.0 / 180.0 * PI,
            0,
            NonZeroU16::MIN,
            SpatialOrder::Mechanical(180)
        ),
        0.0,
        epsilon = 0.0001
    );
}

#[test]
fn test_skew_factor_single_segment() {
    assert_eq!(
        skew_factor(
            6.0 / 180.0 * PI,
            1,
            NonZeroU16::MIN,
            SpatialOrder::Mechanical(60)
        ),
        1.0
    );
    assert_eq!(
        skew_factor(
            6.0 / 180.0 * PI,
            1,
            NonZeroU16::MIN,
            SpatialOrder::Mechanical(10)
        ),
        1.0
    );
    assert_eq!(
        skew_factor(0.1, 1, NonZeroU16::MIN, SpatialOrder::Mechanical(10)),
        1.0
    );
    assert_eq!(
        skew_factor(3.0, 1, NonZeroU16::MIN, SpatialOrder::Mechanical(10)),
        1.0
    );
    assert_eq!(
        skew_factor(3.0, 1, NonZeroU16::MIN, SpatialOrder::Mechanical(20)),
        1.0
    );
    assert_eq!(
        skew_factor(2.0, 1, NonZeroU16::MIN, SpatialOrder::Mechanical(25)),
        1.0
    );
}

// Manually calculate the normalized torque harmonic for a staggered component
// and compare it with the skew factor calculation
#[test]
fn test_skew_factor_multiple_segments() {
    {
        // Cogging torque suppression of a 12/10 winding with staggered rotor magnets
        approxim::assert_abs_diff_eq!(
            skew_factor(
                6.0 / 180.0 * PI,
                3,
                NonZeroU16::MIN,
                SpatialOrder::Mechanical(60)
            ),
            0.0,
            epsilon = 0.0001
        );
        approxim::assert_abs_diff_eq!(
            skew_factor(
                6.0 / 180.0 * PI,
                3,
                NonZeroU16::MIN,
                SpatialOrder::Mechanical(30)
            ),
            2.0 / 3.0,
            epsilon = 0.0001
        );
        approxim::assert_abs_diff_eq!(
            skew_factor(
                12.0 / 180.0 * PI,
                3,
                NonZeroU16::MIN,
                SpatialOrder::Mechanical(30)
            ),
            0.0,
            epsilon = 0.0001
        );
        approxim::assert_abs_diff_eq!(
            skew_factor(
                12.0 / 180.0 * PI,
                3,
                NonZeroU16::MIN,
                SpatialOrder::Mechanical(60)
            ),
            0.0,
            epsilon = 0.0001
        );
        approxim::assert_abs_diff_eq!(
            skew_factor(
                12.0 / 180.0 * PI,
                3,
                NonZeroU16::MIN,
                SpatialOrder::Mechanical(90)
            ),
            1.0,
            epsilon = 0.0001
        );
        approxim::assert_abs_diff_eq!(
            skew_factor(
                12.0 / 180.0 * PI,
                3,
                NonZeroU16::MIN,
                SpatialOrder::Mechanical(120)
            ),
            0.0,
            epsilon = 0.0001
        );
        approxim::assert_abs_diff_eq!(
            skew_factor(
                12.0 / 180.0 * PI,
                3,
                NonZeroU16::MIN,
                SpatialOrder::Mechanical(150)
            ),
            0.0,
            epsilon = 0.0001
        );
        approxim::assert_abs_diff_eq!(
            skew_factor(
                12.0 / 180.0 * PI,
                3,
                NonZeroU16::MIN,
                SpatialOrder::Mechanical(180)
            ),
            1.0,
            epsilon = 0.0001
        );
    }
    {
        const NUMBER_POINTS: usize = 50;

        fn angle(idx: usize, offset: f64) -> f64 {
            return (idx as f64 / NUMBER_POINTS as f64) * TAU + offset;
        }
        fn curve(beta: f64, order: usize) -> Vec<f64> {
            let offset = beta * order as f64;
            return (0..NUMBER_POINTS)
                .map(|idx| angle(idx, offset).sin())
                .collect();
        }
        fn amplitude_two_segments(skew_angle: f64, order: usize) -> f64 {
            // Difference between the segments is beta = segments * skew_angle
            let first_segment = curve(-0.25 * skew_angle, order);
            let second_segment = curve(0.25 * skew_angle, order);
            let amplitude = first_segment
                .iter()
                .zip(second_segment.iter())
                .map(|(x, y)| *x + *y)
                .reduce(f64::max)
                .unwrap()
                / 2.0;
            return amplitude;
        }

        let skew_angle = 6.0 / 180.0 * PI;
        approxim::assert_abs_diff_eq!(
            skew_factor(skew_angle, 2, NonZeroU16::MIN, SpatialOrder::Mechanical(60)),
            amplitude_two_segments(skew_angle, 60),
            epsilon = 0.0001
        );
        approxim::assert_abs_diff_eq!(
            skew_factor(skew_angle, 2, NonZeroU16::MIN, SpatialOrder::Mechanical(30)),
            amplitude_two_segments(skew_angle, 30),
            epsilon = 0.02
        );
        approxim::assert_abs_diff_eq!(
            amplitude_two_segments(skew_angle, 60),
            0.0,
            epsilon = 0.0001
        );

        let skew_angle = 3.0 / 180.0 * PI;
        approxim::assert_abs_diff_eq!(
            skew_factor(skew_angle, 2, NonZeroU16::MIN, SpatialOrder::Mechanical(60)),
            amplitude_two_segments(skew_angle, 60),
            epsilon = 0.02
        );
    }
}

#[test]
fn test_slot_opening_factor() {
    let slot_pitch = Length::new::<millimeter>(10.0);

    // Is ignored, since we are using mechanical orders anyway
    let pole_pairs = NonZeroU16::new(2).expect("not zero");

    // Special (theoretical) case of the current load being concentrated in the slot
    // middle
    assert_abs_diff_eq!(
        slot_opening_factor(
            slot_pitch,
            Length::new::<millimeter>(0.0),
            36,
            pole_pairs,
            SpatialOrder::Mechanical(10)
        ),
        1.0,
        epsilon = 1e-6
    );
    assert_abs_diff_eq!(
        slot_opening_factor(
            slot_pitch,
            Length::new::<millimeter>(0.0),
            36,
            pole_pairs,
            SpatialOrder::Mechanical(10)
        ),
        1.0,
        epsilon = 1e-6
    );

    // Special case of the current load being distributed along the entire slot
    // pitch
    assert_abs_diff_eq!(
        slot_opening_factor(
            slot_pitch,
            slot_pitch,
            36,
            pole_pairs,
            SpatialOrder::Mechanical(1)
        ),
        0.998731,
        epsilon = 1e-6
    );
    assert_abs_diff_eq!(
        slot_opening_factor(
            slot_pitch,
            slot_pitch,
            36,
            pole_pairs,
            SpatialOrder::Mechanical(10)
        ),
        0.877822,
        epsilon = 1e-6
    );

    // Slot opening of 2 mm
    assert_abs_diff_eq!(
        slot_opening_factor(
            slot_pitch,
            Length::new::<millimeter>(2.0),
            36,
            pole_pairs,
            SpatialOrder::Mechanical(1)
        ),
        0.9999492,
        epsilon = 1e-6
    );
    assert_abs_diff_eq!(
        slot_opening_factor(
            slot_pitch,
            Length::new::<millimeter>(2.0),
            36,
            pole_pairs,
            SpatialOrder::Mechanical(10)
        ),
        0.9949307,
        epsilon = 1e-6
    );
}
