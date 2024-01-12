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
        -1.,
        2.,
    ));

    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Cubic,
        2.5,
        1.,
    ));

    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Cubic,
        0.,
        5.,
    ));

    let mut roll_rate = Transitions::new(0.);
    roll_rate.transitions.push(Transition::new(
        transitions::TransitionFunction::Cubic,
        0.,
        23.,
    ));

    dbg!(vertical.interpolate(0.));
    dbg!(vertical.interpolate(3.));

    println!(
        "{}",
        fvd::create_spline(&vertical, &roll_rate, Vec3::new(0., 10., 0.), 10.)
            .to_nolimits_element()
    );
}
