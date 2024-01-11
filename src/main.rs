use glam::{Quat, Vec3};
use transitions::{Transition, Transitions};

use crate::spline::TrackSpline;

mod spline;
mod transitions;

fn main() {
    let mut vertical = Transitions::new(1.);
    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Plateau,
        -2.,
        1.,
    ));
    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Cubic,
        1.,
        1.,
    ));
    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Cubic,
        0.,
        1.,
    ));

    dbg!(vertical.interpolate(0.));
    dbg!(vertical.interpolate(0.5));
    dbg!(vertical.interpolate(1.));

    let mut track = TrackSpline::new();
    track.points.push((Vec3::ZERO, 0.));
    track.points.push((Vec3::new(10., 5., 0.), 0.1));
    track.points.push((Vec3::new(20., 0., 0.), 0.));

    dbg!(track.evaluate(0.), track.evaluate(0.99999), track.length());
    println!("{}", track.to_nolimits_element());
}
