#[derive(Clone, Debug)]
pub struct Transitions {
    pub transitions: Vec<Transition>,
    start_value: f32,
}

impl Transitions {
    pub fn new(start_value: f32) -> Self {
        Self {
            transitions: Vec::new(),
            start_value,
        }
    }

    pub fn interpolate(&self, time: f32) -> Option<f32> {
        let mut time_so_far = 0.;
        let mut value = self.start_value;
        for transition in &self.transitions {
            if time_so_far <= time && time <= time_so_far + transition.length {
                return Some(transition.interpolate(time - time_so_far) + value);
            }
            value += transition.end_value();
            time_so_far += transition.length;
        }
        None
    }
    pub fn length(&self) -> f32 {
        self.transitions.iter().map(|v| v.length).sum()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    function: TransitionFunction,
    change: f32,
    length: f32,
}

impl Transition {
    pub fn new(function: TransitionFunction, change: f32, length: f32) -> Transition {
        Transition {
            function,
            change,
            length,
        }
    }
    pub fn interpolate(&self, time: f32) -> f32 {
        if self.change == 0. {
            return 0.;
        }
        let t = time / self.length;

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
                if (0.3..=0.7).contains(&t) {
                    1.
                } else if t < 0.3 {
                    ezing::cubic_inout(t * (1. / 0.3))
                } else {
                    1. - ezing::cubic_inout((t - 0.7) * (1. / 0.3))
                }
            }
        }
    }
}
