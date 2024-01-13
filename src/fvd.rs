use glam::{Quat, Vec3};

use crate::{spline::TrackSpline, transitions::Transitions};
pub const DT: f32 = 0.01; // seconds between integrating
pub const INTERVAL: f32 = 0.3; // m between points

pub const GRAVITY: f32 = 9.806; // m/s^2

pub const FORWARD: Vec3 = Vec3::Z;
pub const UP: Vec3 = Vec3::Y;
pub const RIGHT: Vec3 = Vec3::X;

pub fn create_spline(
    vertical: &Transitions,
    roll_rate: &Transitions,
    start: Vec3,
    start_velocity: f32,
) -> TrackSpline {
    let mut spline = TrackSpline::new();
    let mut velocity = start_velocity;
    let mut pos = start;
    let mut traveled = 0.;
    let mut direction = Quat::IDENTITY;
    let mut time = 0.;

    let mut next_point = 0.;

    while time < vertical.length() {
        // dbg!((2. * GRAVITY * (start.y + 10.0 - pos.y)).sqrt());
        velocity = start_velocity + (2. * GRAVITY * (start.y - pos.y)).sqrt();
        let vert = vertical.interpolate(time);
        // let lat = lateral.interpolate(time);
        let lat = Some(0.1);
        let roll = roll_rate.interpolate(time);

        match (vert, lat, roll) {
            (Some(vert), Some(lat), Some(roll)) => {
                dbg!(vert);
                direction = direction.normalize();
                let lat_radius =
                    (velocity * velocity) / ((lat - (direction * RIGHT).dot(-UP)) * GRAVITY);
                let delta_lat_rot = if lat_radius.is_finite() {
                    (1. / lat_radius) * DT * velocity * std::f32::consts::PI
                } else {
                    0.
                };

                let vert_radius =
                    (velocity * velocity) / -((vert + (direction * UP).dot(-UP)) * GRAVITY);
                dbg!(vert_radius);
                let delta_vert_rot = if vert_radius.is_finite() {
                    (1. / vert_radius) * DT * velocity * std::f32::consts::PI
                } else {
                    0.
                };
                dbg!(vert_radius);
                let old_dir = direction;
                direction *= Quat::from_axis_angle(old_dir * UP, delta_lat_rot); // horizontal rotation
                direction *= Quat::from_axis_angle(old_dir * RIGHT, delta_vert_rot); // vertical rotation
                direction *= Quat::from_axis_angle(old_dir * FORWARD, roll.to_radians() * DT); // roll

                direction = direction.normalize();

                pos += direction * FORWARD * (DT * velocity);
            }
            _ => {
                break;
            }
        }
        time += DT;
        traveled += DT * velocity;
        if traveled > next_point {
            spline.points.push((pos, direction));
            next_point = traveled + INTERVAL;
        }
    }

    spline
}
