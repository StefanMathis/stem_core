use std::f64::consts::{PI, TAU};

use approxim::assert_abs_diff_eq;
use stem_core::air_gap::slot_opening_factor;
use stem_core::core::ext::skew_factor;
use stem_core::stem_material::prelude::*;

#[test]
fn test_skew_factor_no_segment() {
    approxim::assert_abs_diff_eq!(skew_factor(60, 6.0 / 180.0 * PI, 0), 0.0, epsilon = 0.0001);
    approxim::assert_abs_diff_eq!(skew_factor(30, 12.0 / 180.0 * PI, 0), 0.0, epsilon = 0.0001);
    approxim::assert_abs_diff_eq!(skew_factor(60, 12.0 / 180.0 * PI, 0), 0.0, epsilon = 0.0001);
    approxim::assert_abs_diff_eq!(skew_factor(90, 12.0 / 180.0 * PI, 0), 0.0, epsilon = 0.0001);
    approxim::assert_abs_diff_eq!(
        skew_factor(120, 12.0 / 180.0 * PI, 0),
        0.0,
        epsilon = 0.0001
    );
    approxim::assert_abs_diff_eq!(
        skew_factor(150, 12.0 / 180.0 * PI, 0),
        0.0,
        epsilon = 0.0001
    );
    approxim::assert_abs_diff_eq!(
        skew_factor(180, 12.0 / 180.0 * PI, 0),
        0.0,
        epsilon = 0.0001
    );
}

#[test]
fn test_skew_factor_single_segment() {
    assert_eq!(skew_factor(60, 6.0 / 180.0 * PI, 1), 1.0);
    assert_eq!(skew_factor(10, 6.0 / 180.0 * PI, 1), 1.0);
    assert_eq!(skew_factor(10, 0.1, 1), 1.0);
    assert_eq!(skew_factor(10, 3.0, 1), 1.0);
    assert_eq!(skew_factor(20, 3.0, 1), 1.0);
    assert_eq!(skew_factor(25, 2.0, 1), 1.0);
}

// Manually calculate the normalized torque harmonic for a staggered component
// and compare it with the skew factor calculation
#[test]
fn test_skew_factor_multiple_segments() {
    {
        // Cogging torque suppression of a 12/10 winding with staggered rotor magnets
        approxim::assert_abs_diff_eq!(skew_factor(60, 6.0 / 180.0 * PI, 3), 0.0, epsilon = 0.0001);
        approxim::assert_abs_diff_eq!(
            skew_factor(30, 6.0 / 180.0 * PI, 3),
            2.0 / 3.0,
            epsilon = 0.0001
        );
        approxim::assert_abs_diff_eq!(skew_factor(30, 12.0 / 180.0 * PI, 3), 0.0, epsilon = 0.0001);
        approxim::assert_abs_diff_eq!(skew_factor(60, 12.0 / 180.0 * PI, 3), 0.0, epsilon = 0.0001);
        approxim::assert_abs_diff_eq!(skew_factor(90, 12.0 / 180.0 * PI, 3), 1.0, epsilon = 0.0001);
        approxim::assert_abs_diff_eq!(
            skew_factor(120, 12.0 / 180.0 * PI, 3),
            0.0,
            epsilon = 0.0001
        );
        approxim::assert_abs_diff_eq!(
            skew_factor(150, 12.0 / 180.0 * PI, 3),
            0.0,
            epsilon = 0.0001
        );
        approxim::assert_abs_diff_eq!(
            skew_factor(180, 12.0 / 180.0 * PI, 3),
            1.0,
            epsilon = 0.0001
        );
    }
    {
        const NUMBER_POINTS: usize = 50;

        fn angle(idx: usize, offset: f64) -> f64 {
            return (idx as f64 / NUMBER_POINTS as f64) * TAU + offset;
        }
        fn curve(beta: f64, ordinal: usize) -> Vec<f64> {
            let offset = beta * ordinal as f64;
            return (0..NUMBER_POINTS)
                .map(|idx| angle(idx, offset).sin())
                .collect();
        }
        fn amplitude_two_segments(skew_angle: f64, ordinal: usize) -> f64 {
            // Difference between the segments is beta = segments * skew_angle
            let first_segment = curve(-0.25 * skew_angle, ordinal);
            let second_segment = curve(0.25 * skew_angle, ordinal);
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
            skew_factor(60, skew_angle, 2),
            amplitude_two_segments(skew_angle, 60),
            epsilon = 0.0001
        );
        approxim::assert_abs_diff_eq!(
            skew_factor(30, skew_angle, 2),
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
            skew_factor(60, skew_angle, 2),
            amplitude_two_segments(skew_angle, 60),
            epsilon = 0.02
        );
    }
}

#[test]
fn test_slot_opening_factor() {
    let slot_pitch = Length::new::<millimeter>(10.0);

    // Special (theoretical) case of the current load being concentrated in the slot
    // middle
    assert_abs_diff_eq!(
        slot_opening_factor(slot_pitch, Length::new::<millimeter>(0.0), 36, 1),
        1.0,
        epsilon = 1e-6
    );
    assert_abs_diff_eq!(
        slot_opening_factor(slot_pitch, Length::new::<millimeter>(0.0), 36, 10),
        1.0,
        epsilon = 1e-6
    );

    // Special case of the current load being distributed along the entire slot
    // pitch
    assert_abs_diff_eq!(
        slot_opening_factor(slot_pitch, slot_pitch, 36, 1),
        0.998731,
        epsilon = 1e-6
    );
    assert_abs_diff_eq!(
        slot_opening_factor(slot_pitch, slot_pitch, 36, 10),
        0.877822,
        epsilon = 1e-6
    );

    // Slot opening of 2 mm
    assert_abs_diff_eq!(
        slot_opening_factor(slot_pitch, Length::new::<millimeter>(2.0), 36, 1),
        0.9999492,
        epsilon = 1e-6
    );
    assert_abs_diff_eq!(
        slot_opening_factor(slot_pitch, Length::new::<millimeter>(2.0), 36, 10),
        0.9949307,
        epsilon = 1e-6
    );
}
