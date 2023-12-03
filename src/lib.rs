#![allow(non_snake_case)]

#[cfg(feature = "f64")]
pub use glam::{dvec3 as vec3, DMat3 as Mat3, DVec3 as Vec3};
#[cfg(feature = "f32")]
pub use glam::{vec3, Mat3, Vec3};

#[cfg(feature = "f32")]
pub type Num = f32;

#[cfg(feature = "f64")]
pub type Num = f64;

pub mod astro;
pub mod constants;
pub mod elements;
pub mod math;
pub mod state_vectors;
pub mod utils;

use constants::{G, PI, TWO_PI};

pub use self::elements::KeplerianElements;
pub use self::state_vectors::StateVectors;

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    const MASS: Num = 100_000_000_000.0;
    const EPOCH: Num = 0.0;
    const MAX_ABS_DIFF: Num = 0.0001;
    const TOLERANCE: Num = 0.0001;

    // #[track_caller]
    fn test_back_and_forth_conversion(original: KeplerianElements, mass: Num, epoch: Num) {
        let sv = original.state_vectors_at_epoch(mass, epoch, TOLERANCE);

        let elements = KeplerianElements::from_state_vectors(&sv, mass, epoch);

        let sv_converted = elements.state_vectors_at_epoch(mass, epoch, TOLERANCE);

        // let elements_converted = KeplerianElements::from_state_vectors(&sv_converted, mass, epoch);
        // println!("Elements converted: {elements_converted:#?}");

        println!("Original: {original:#?}");
        println!("State vectors: {sv:#?}");
        println!("Elements: {elements:#?}");
        println!("State vectors converted: {sv_converted:#?}");

        let pos_diff = sv.position.distance(sv_converted.position);
        assert!(
            sv.position.abs_diff_eq(sv_converted.position, MAX_ABS_DIFF),
            "Position {:?} not equal {:?} - distance is {}",
            sv.position,
            sv_converted.position,
            pos_diff
        );
        assert!(
            sv.velocity.abs_diff_eq(sv_converted.velocity, MAX_ABS_DIFF),
            "Velocity {:?} not equal {:?}",
            sv.velocity,
            sv_converted.velocity
        );
    }

    #[test]
    fn conversion_zero_params() {
        test_back_and_forth_conversion(
            KeplerianElements {
                eccentricity: 0.0,
                semi_major_axis: 1.0,
                inclination: 0.0,
                right_ascension_of_the_ascending_node: 0.0,
                argument_of_periapsis: 0.0,
                mean_anomaly_at_epoch: 0.0,
                epoch: 0.0,
            },
            MASS,
            EPOCH,
        );
    }

    #[test]
    fn conversion_highly_eccentric() {
        test_back_and_forth_conversion(
            KeplerianElements {
                eccentricity: 0.9,
                semi_major_axis: 1.0,
                inclination: 0.0,
                right_ascension_of_the_ascending_node: 0.0,
                argument_of_periapsis: 0.0,
                mean_anomaly_at_epoch: 0.0,
                epoch: 0.0,
            },
            MASS,
            EPOCH,
        );
    }

    #[test]
    fn conversion_arbitrary() {
        test_back_and_forth_conversion(
            KeplerianElements {
                eccentricity: 0.123,
                semi_major_axis: 1.0,
                inclination: 1.2,
                right_ascension_of_the_ascending_node: 0.5,
                argument_of_periapsis: 0.3,
                mean_anomaly_at_epoch: 1.01,
                epoch: 0.1,
            },
            MASS,
            EPOCH,
        );
    }

    #[test]
    fn conversion_error_case_1() {
        test_back_and_forth_conversion(
            KeplerianElements {
                eccentricity: 0.005408803,
                semi_major_axis: 751338500.0,
                inclination: 0.023166878,
                right_ascension_of_the_ascending_node: 1.7773559,
                argument_of_periapsis: 1.3521711,
                mean_anomaly_at_epoch: -0.46838284,
                epoch: 5837.1787,
            },
            19890000.0,
            5844.272,
        );
    }

    #[test]
    fn conversion_error_case_2() {
        test_back_and_forth_conversion(
            KeplerianElements {
                eccentricity: 0.0069337534,
                semi_major_axis: 752926600.0,
                inclination: 0.023143709,
                right_ascension_of_the_ascending_node: 1.7756647,
                argument_of_periapsis: 0.9487356,
                mean_anomaly_at_epoch: -0.06723439,
                epoch: 0.0,
            },
            19890000.0,
            0.0,
        );
    }

    #[test]
    fn state_vectors_conversion() {
        let sv = StateVectors {
            position: vec3(-661208300.0, 348866180.0, 13342606.0),
            velocity: vec3(-6.13e-7, -1.182874e-6, 1.9689882e-8),
        };


    }

    #[test_case(0.0, vec3(1.0, 0.0, 0.0))]
    #[test_case(PI / 2.0, vec3(0.0, 1.0, 0.0))]
    #[test_case(PI, vec3(-1.0, 0.0, 0.0))]
    #[test_case(PI + (PI / 2.0), vec3(0.0, -1.0, 0.0))]
    fn elements_to_position(v: Num, exp: Vec3) {
        let elements = KeplerianElements {
            eccentricity: 0.0,
            semi_major_axis: 1.0,
            inclination: 0.0,
            right_ascension_of_the_ascending_node: 0.0,
            argument_of_periapsis: 0.0,
            mean_anomaly_at_epoch: 0.0,
            epoch: 0.0,
        };

        let position = elements.position_at_true_anomaly(MASS, v);
        let velocity = elements.velocity_at_true_anomaly(MASS, v);

        println!("velocity = {velocity:#?}");

        assert!(
            position.abs_diff_eq(exp, MAX_ABS_DIFF),
            "Position {:?} not equal {:?}",
            position,
            exp
        );
    }
}
