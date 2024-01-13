use glam::{Quat, Vec3};
use transitions::{Transition, Transitions};

use crate::spline::TrackSpline;

mod fvd;
mod spline;
mod transitions;
mod units;

fn main() {
    let mut vertical = Transitions::new(1.);
    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Plateau,
        -2.,
        1.,
    ));

    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Cubic,
        2.,
        0.5,
    ));
    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Cubic,
        0.,
        2.,
    ));

    vertical.transitions.push(Transition::new(
        transitions::TransitionFunction::Plateau,
        -2.5,
        2.,
    ));

    let mut roll_rate = Transitions::new(0.);

    roll_rate.transitions.push(Transition::new(
        transitions::TransitionFunction::Plateau,
        0.,
        10.,
    ));

    // roll_rate.transitions.push(Transition::new(
    //     transitions::TransitionFunction::Plateau,
    //     200.,
    //     4.,
    // ));

    println!(
        "{}",
        fvd::create_spline(&vertical, &roll_rate, Vec3::new(0., 20., 0.), 5.).to_nolimits_element()
    );
}
