extern crate binning_analysis;
extern crate mersenne_twister;
extern crate rand;

use binning_analysis::Accumulator;
use mersenne_twister::MersenneTwister;
use rand::{Rng, SeedableRng};

#[test]
fn random_test() {
    let seed: u64 = 22919;
    let mut rng: MersenneTwister = SeedableRng::from_seed(seed);
    let mut data = Accumulator::<f64>::new();
    let mut x: f64 = 0.0;
    for _ in 0..65536 * 128 {
        let next = x + rng.next_f64() - 0.5;
        if x.abs() < next.abs() {
            let probability = ((x * x - next * next) * 0.125).exp();
            let random = rng.next_f64();
            if probability > random {
                x = next;
            }
        } else {
            x = next;
        }
        data.push(x);
    }
    let result = data.binning();
    let res = binning_analysis::Analyzed::<f64> {
        mean: 0.0,
        error: 0.0,
        correlation_time: 1.0,
        converged: false,
        number_of_inputs: 1000,
    };
    assert_eq!(result.unwrap(), res);
}
