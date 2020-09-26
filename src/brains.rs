#![allow(unused)]
use crate::scaled_rand;
use rand::Rng;
const N_INPUTS: usize = 4;
const N_OUTPUTS: usize = 6;

const INNER_SIZE: usize = (N_INPUTS + N_OUTPUTS) / 2;

/// stored as an array for easy
/// neural network access.
/// but accessed/modified through methods
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Inputs {
    data: [f64; N_INPUTS],
}

impl Inputs {
    pub fn sound_mut(&mut self) -> &mut f64 {
        &mut self.data[0]
    }
    pub fn smell_mut(&mut self) -> &mut f64 {
        &mut self.data[1]
    }
    pub fn clock1_mut(&mut self) -> &mut f64 {
        &mut self.data[2]
    }
    pub fn clock2_mut(&mut self) -> &mut f64 {
        &mut self.data[3]
    }
}

/// stored as an array for easy
/// neural network access.
/// but accessed/modified through methods
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Outputs {
    data: [f64; N_OUTPUTS],
}
impl Outputs {
    pub fn spike(&self) -> f64 {
        self.data[0]
    }
    pub fn steering(&self) -> f64 {
        self.data[1]
    }
    pub fn speed(&self) -> f64 {
        // the exp is to undo the sigmoid from the nn, maybe i can output raws and do the
        // sigmoid inline on the getters
        self.data[2].exp()
    }
    // would be nice if there was an easy way to return &[f64;3] split of from the main array
    pub fn r(&self) -> f64 {
        self.data[3] + 0.5
    }
    pub fn b(&self) -> f64 {
        self.data[4] + 0.5
    }
    pub fn g(&self) -> f64 {
        self.data[5] + 0.5
    }
}

pub trait Brain: Clone {
    fn init<R: Rng>(rng: R) -> Self;
    fn think(&self, inputs: &Inputs) -> Outputs;
    fn mutate<R: Rng>(&mut self, rng: R, rate: f64);
}

#[derive(Copy, Clone, Default, PartialEq)]
pub struct BigBrain {
    // each output gets a weight for each input
    in2mid: [[f64; N_INPUTS]; INNER_SIZE],
    mid_bias: [f64; INNER_SIZE],
    mid2out: [[f64; INNER_SIZE]; N_OUTPUTS],
    out_bias: [f64; N_OUTPUTS],
    // todo: maybe i can add some loopback values
}

impl Brain for BigBrain {
    fn mutate<R: Rng>(&mut self, mut rng: R, rate: f64) {
        for mid in &mut self.in2mid {
            for inp in mid.iter_mut() {
                scaled_rand(&mut rng, rate, 0.1, 0.1, inp);
            }
        }

        for out in &mut self.mid2out {
            for mid in out.iter_mut() {
                scaled_rand(&mut rng, rate, 0.1, 0.1, mid);
            }
        }

        for bias in self.mid_bias.iter_mut() {
            scaled_rand(&mut rng, rate, 0.01, 0.01, bias);
        }

        for bias in self.out_bias.iter_mut() {
            scaled_rand(&mut rng, rate, 0.01, 0.01, bias);
        }
    }
    fn init<R: Rng>(mut r: R) -> Self {
        let mut s: Self = Default::default();
        for mid in &mut s.in2mid {
            for inp in mid.iter_mut() {
                *inp = r.gen_range(-0.1, 0.1);
            }
        }
        for out in &mut s.mid2out {
            for mid in out.iter_mut() {
                *mid = r.gen_range(-0.1, 0.1);
            }
        }
        for bias in &mut s.mid_bias {
            *bias = r.gen_range(-0.01, 0.01);
        }
        for bias in &mut s.out_bias {
            *bias = r.gen_range(-0.01, 0.01);
        }
        s
    }
    // todo: maybe use my own lib for this
    // no direct learing is happening so maybe not
    fn think(&self, inputs: &Inputs) -> Outputs {
        let mut mid = [0.0_f64; INNER_SIZE];

        for ((iw, m), bias) in self
            .in2mid
            .iter()
            .zip(mid.iter_mut())
            .zip(self.mid_bias.iter())
        {
            assert_eq!(iw.len(), inputs.data.len());
            let weighted_in: f64 = iw.iter().zip(&inputs.data).map(|(iw, i)| iw * i).sum();
            let weighted_in = weighted_in + bias;
            let clamped = weighted_in.max(-20.).min(20.);
            let res = 1. / (1. + (-clamped).exp());
            // center sigmoid around 0
            *m = res - 0.5;
        }

        let mut o: Outputs = Outputs::default();
        for ((mw, o), bias) in self
            .mid2out
            .iter()
            .zip(o.data.iter_mut())
            .zip(self.out_bias.iter())
        {
            let weighted_in: f64 = mw.iter().zip(&mid).map(|(mw, m)| mw * m).sum();
            let weighted_in = weighted_in + bias;
            let clamped = weighted_in.max(-20.).min(20.);
            let res = 1. / (1. + (-clamped).exp());
            // center sigmoid around 0
            *o = res - 0.5;
        }
        o
    }
}

#[derive(Copy, Clone, Default, PartialEq)]
pub struct SimpleBrain {
    // each output gets a weight for each input
    weights: [[f64; N_INPUTS]; INNER_SIZE],
    bias: [f64; INNER_SIZE],
}

impl Brain for SimpleBrain {
    fn mutate<R: Rng>(&mut self, mut rng: R, rate: f64) {
        for out in &mut self.weights {
            for inp in out.iter_mut() {
                scaled_rand(&mut rng, rate, 0.1, 0.1, inp);
            }
        }

        for bias in self.bias.iter_mut() {
            scaled_rand(&mut rng, rate, 0.01, 0.01, bias);
        }
    }
    fn init<R: Rng>(mut r: R) -> Self {
        let mut s: Self = Default::default();
        for out in &mut s.weights {
            for inp in out.iter_mut() {
                *inp = r.gen_range(-0.1, 0.1);
            }
        }
        for bias in &mut s.bias {
            *bias = r.gen_range(-0.01, 0.01);
        }
        s
    }
    // todo: maybe use my own lib for this
    // no direct learing is happening so maybe not
    fn think(&self, inputs: &Inputs) -> Outputs {
        let mut o: Outputs = Outputs::default();
        for ((iw, o), bias) in self
            .weights
            .iter()
            .zip(o.data.iter_mut())
            .zip(self.bias.iter())
        {
            let weighted_in: f64 = iw.iter().zip(&inputs.data).map(|(iw, i)| iw * i).sum();
            let weighted_in = weighted_in + bias;
            let clamped = weighted_in.max(-20.).min(20.);
            let res = 1. / (1. + (-clamped).exp());
            // center sigmoid around 0
            *o = res - 0.5;
        }
        o
    }
}
