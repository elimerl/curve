#[derive(Clone, Debug)]
pub struct Transitions {
    pub transitions: Vec<FullTransition>,
    pub vert_start: f32,
    pub lat_start: f32,
    pub roll_start: f32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FullTransition {
    pub vert: Transition,
    pub lat: Transition,
    pub roll: Transition,
    pub length: f32,
    pub speed: Option<f32>,
}

impl FullTransition {
    pub fn new(
        vert: Transition,
        lat: Transition,
        roll: Transition,
        length: f32,
        speed: Option<f32>,
    ) -> Self {
        Self {
            vert,
            lat,
            roll,
            length,
            speed,
        }
    }
}

impl Default for FullTransition {
    fn default() -> Self {
        let default = Transition::new(TransitionFunction::Cubic, 0.);
        Self::new(
            default,
            default,
            Transition::new(TransitionFunction::Plateau, 0.),
            1.,
            None,
        )
    }
}

impl Transitions {
    pub fn new(vert_start: f32, lat_start: f32, roll_start: f32) -> Self {
        Self {
            transitions: vec![FullTransition::default()],
            vert_start,
            lat_start,
            roll_start,
        }
    }

    pub fn interpolate(&self, time: f32) -> Option<(f32, f32, f32, Option<f32>)> {
        let mut vert_value = self.vert_start;
        let mut lat_value = self.lat_start;
        let mut roll_value = self.roll_start;
        let mut fixed_speed = None;
        if time < 0. || time > self.length() {
            return None;
        }

        {
            let mut time_so_far = 0.;

            for transition in &self.transitions {
                fixed_speed = transition.speed;

                if time_so_far <= time && time <= time_so_far + transition.length {
                    vert_value += transition
                        .vert
                        .interpolate((time - time_so_far) / transition.length);
                    lat_value += transition
                        .lat
                        .interpolate((time - time_so_far) / transition.length);
                    roll_value += transition
                        .roll
                        .interpolate((time - time_so_far) / transition.length);
                    break;
                }
                vert_value += transition.vert.end_value();
                lat_value += transition.lat.end_value();
                roll_value += transition.roll.end_value();
                time_so_far += transition.length;
            }
        }

        Some((vert_value, lat_value, roll_value, fixed_speed))
    }
    pub fn length(&self) -> f32 {
        self.transitions.iter().map(|v| v.length).sum::<f32>()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    pub function: TransitionFunction,
    pub change: f32,
}

impl Transition {
    pub fn new(function: TransitionFunction, change: f32) -> Transition {
        Transition { function, change }
    }
    pub fn interpolate(&self, time: f32) -> f32 {
        if self.change == 0. {
            return 0.;
        }
        let t = time;

        self.function.interpolate(t) * self.change
    }
    pub fn end_value(&self) -> f32 {
        match self.function {
            TransitionFunction::Cubic
            | TransitionFunction::Quadratic
            | TransitionFunction::Linear => self.change,
            TransitionFunction::Plateau => 0.,
            // _ => self.function.interpolate(1.) * self.change,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionFunction {
    Linear,
    Quadratic,
    Cubic,
    Plateau,
}

impl TransitionFunction {
    pub fn interpolate(&self, t: f32) -> f32 {
        let t = t.clamp(0., 1.);
        match self {
            TransitionFunction::Linear => ezing::linear(t),
            TransitionFunction::Quadratic => ezing::quad_inout(t),
            TransitionFunction::Cubic => ezing::cubic_inout(t),
            TransitionFunction::Plateau => {
                if (0.33..=0.66).contains(&t) {
                    1.
                } else if t < 0.33 {
                    ezing::sine_inout(t * (1. / 0.33))
                } else {
                    1. - ezing::sine_inout((t - 0.66) * (1. / 0.33))
                }
            }
        }
    }
}
