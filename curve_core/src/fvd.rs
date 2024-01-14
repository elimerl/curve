use glam::{Quat, Vec3};

use crate::{spline::TrackSpline, transitions::Transitions, units::G};
pub const DT: f32 = 0.01; // seconds between integrating

pub const GRAVITY: Vec3 = Vec3::new(0., -9.806, 0.); // m/s^2

pub const FORWARD: Vec3 = Vec3::Z;
pub const UP: Vec3 = Vec3::Y;
pub const RIGHT: Vec3 = Vec3::X;

pub const EPSILON: f32 = 0.0001;

pub fn create_spline(transitions: &Transitions, start: Vec3, start_velocity: f32) -> TrackSpline {
    let mut spline = TrackSpline::new();
    let mut velocity = start_velocity;
    let mut pos = start;
    let mut traveled = 0.;
    let mut direction = Quat::IDENTITY;
    let mut time = 0.;

    while time < transitions.length() {
        let length = DT * velocity;

        match transitions.interpolate(time) {
            Some((vert, lat, roll_rate, fixed_speed)) => {
                velocity = if let Some(v) = fixed_speed {
                    v
                } else {
                    velocity
                };
                let mut new_dir = direction;
                let linear_accel =
                    (vert * G * (direction * -UP)) + (lat * G * (direction * -RIGHT));
                let remainder_accel = GRAVITY - linear_accel;
                let forward_accel = remainder_accel.project_onto(direction * FORWARD);
                let centripetal_accel = remainder_accel - forward_accel;

                if centripetal_accel.length_squared() > EPSILON {
                    let axis = (direction * FORWARD).cross(centripetal_accel).normalize();
                    let radius = velocity * velocity / centripetal_accel.length();
                    let angle = length / radius;
                    let rel_rot = Quat::from_axis_angle(axis, angle);
                    new_dir = rel_rot * new_dir;
                }

                let rel_rot = Quat::from_axis_angle(
                    (direction * FORWARD).normalize(),
                    roll_rate.to_radians() * DT,
                );
                new_dir = rel_rot * new_dir;

                direction = new_dir.normalize();

                pos += direction * FORWARD * length;
            }
            _ => {
                break;
            }
        }
        let d_height = (direction * FORWARD * length).y;
        velocity = (velocity * velocity + 2. * GRAVITY.y * d_height).sqrt();
        time += DT;
        traveled += length;
        spline.points.push((pos, direction));
    }

    spline
}
