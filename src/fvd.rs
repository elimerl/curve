use glam::{Quat, Vec3};

use crate::{spline::TrackSpline, transitions::Transitions};
pub const DT: f32 = 0.01; // seconds between integrating
pub const INTERVAL: f32 = 1.; // m between points

pub const GRAVITY: f32 = 9.806; // m/s^2

pub const FORWARD: Vec3 = Vec3::X;
pub const UP: Vec3 = Vec3::Y;
pub const RIGHT: Vec3 = Vec3::Z;

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
    let mut roll = 0f32;
    let mut time = 0.;

    let mut next_point = 0.;

    while time < vertical.length() {
        // dbg!((2. * GRAVITY * (start.y + 10.0 - pos.y)).sqrt());
        velocity = start_velocity + (2. * GRAVITY * ((start.y) - pos.y)).sqrt();
        let vert = vertical.interpolate(time);
        // let lat = lateral.interpolate(time);
        let lat = Some(0.);
        match (vert, lat) {
            (Some(vert), Some(lat)) => {
                let up_relative = direction * UP;
                let right_relative = direction * RIGHT;
                let forward_relative = direction * FORWARD;

                let vert_radius = (velocity * velocity) / ((vert + up_relative.dot(-UP)) * GRAVITY); // FIXME 2g difference, up_relative.dot(-UP) is broken
                dbg!(vert_radius);
                if vert_radius != 0. {
                    let delta_up_rot = (1. / vert_radius) * DT * velocity * std::f32::consts::PI;

                    let up_rot = Quat::from_axis_angle(right_relative, delta_up_rot);

                    direction = (direction * up_rot).normalize();
                }

                let roll = Quat::from_axis_angle(forward_relative, 0.0);
                direction = (direction * roll).normalize();

                // dbg!((direction * FORWARD).y);
                pos += direction * FORWARD * (DT * velocity);
            }
            _ => {
                break;
            }
        }
        if let Some(rate) = roll_rate.interpolate(time) {
            roll += rate * DT;
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
