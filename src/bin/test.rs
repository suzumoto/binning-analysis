extern crate binning_analysis;
extern crate mersenne_twister;
extern crate rand;

use binning_analysis::Accumulator;
use mersenne_twister::MersenneTwister;
use rand::{Rng, SeedableRng};

fn main() {
    let seed: u64 = 30;
    let mut rng: MersenneTwister = SeedableRng::from_seed(seed);
    let mut data = Accumulator::<f64>::new();
    let mut vec = Vec::<f64>::new();
    let mut x: f64 = 0.0;
    for _ in 0..1000000 {
        let next = x + rng.next_f64() - 0.5;
        if x.abs() < next.abs() {
            let probability = ((x * x - next * next) * 0.0125).exp();
            let random = rng.next_f64();
            if probability > random {
                x = next;
            }
        } else {
            x = next;
        }
        data.push(x);
        vec.push(x);
    }
    let mean = vec.iter().fold(0.0, |sum, value| sum + value) / vec.len() as f64;
    let vari = vec
        .iter()
        .fold(0.0, |sum, value| sum + (value - mean) * (value - mean))
        / (vec.len() - 1) as f64;
    let vari_for_mean = vari / vec.len() as f64;
    println!("mean:              {}", mean);
    println!("vari:              {}", vari);
    println!("variance for mean: {}", vari_for_mean);
    println!("error for main:    {}", vari_for_mean.sqrt());
    let result = data.binning();
    println!("jackknife:         {:?}", result);
}
