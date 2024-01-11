use glam::Vec3;

use crate::{spline::TrackSpline, transitions::Transitions};
pub const DT: f32 = 0.01; // seconds between integrating
pub const INTERVAL: f32 = 2.; // m between points

pub const GRAVITY: f32 = 9.806; // m/s^2

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
    let mut forward = Vec3::X;
    let mut roll = 0f32;
    let mut time = 0.;

    let mut last_point_traveled = 0.;

    while time < vertical.length() {
        let vert = vertical.interpolate(time);
        // let lat = lateral.interpolate(time);
        let lat = Some(0.);
        match (vert, lat) {
            (Some(vert), Some(lat)) => {
                let velocity = (2. * (start.y - pos.y) * GRAVITY).sqrt() + start_velocity;
                let vert_radius = (velocity * velocity) / ((vert * GRAVITY) - roll.cos() * GRAVITY);
                let delta_vert_angle = (1. / vert_radius) * velocity;
                let delta_vert_x = delta_vert_angle.cos();
                let delta_vert_y = delta_vert_angle.sin();
                pos += Vec3::new(delta_vert_x, delta_vert_y, 0.);
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
        if traveled > last_point_traveled {
            spline.points.push((pos, roll));
        }
    }

    spline
}
