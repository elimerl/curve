use glam::{Quat, Vec3};
use transitions::{Transition, Transitions};

use crate::spline::TrackSpline;

mod fvd;
mod spline;
mod transitions;

fn main() {
    let mut vertical = Transitions::new(1.);
    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Plateau,
        -2.,
        2.,
    ));
    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Cubic,
        2.,
        1.,
    ));
    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Cubic,
        -4.,
        1.,
    ));
    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Cubic,
        4.,
        1.,
    ));

    let mut roll_rate = Transitions::new(0.);
    roll_rate.transitions.push(Transition::new(
        transitions::TransitionFunction::Cubic,
        0.,
        5.,
    ));

    dbg!(vertical.interpolate(0.));
    dbg!(vertical.interpolate(0.5));
    dbg!(vertical.interpolate(1.));

    println!(
        "{}",
        fvd::create_spline(&vertical, &roll_rate, Vec3::new(0., 10., 0.), 5.).to_nolimits_element()
    );
}
