extern crate num;
use num::Float;

#[derive(Debug, PartialEq)]
pub struct Accumulator<T: Float> {
    dataset: Vec<T>,
}

impl<T: Float>  Accumulator<T> {
    pub fn push(&mut self, observable: T) {
        self.dataset.push(observable);
    }
}
#[test]
fn push_test(){
    let mut measurement = Accumulator::<f64> {
        dataset: Vec::new(),
    };
    measurement.push(1.0);
    measurement.push(2.0);
    
    let mut raw_vector = Vec::<f64>::new();
    raw_vector.push(1.0);
    raw_vector.push(2.0);
    
    let mut test_instance = Accumulator::<f64> {
        dataset: raw_vector,
    };
    assert_eq!(measurement, test_instance);
}

#[derive(Debug, PartialEq)]
pub struct Analyzed<T: Float + Copy> {
    mean: T,
    error: T,
    number_of_inputs: u64,
    correlation_time: Option<T>,
    converged: bool
}

use std::fmt;

impl<T: Float + Copy + fmt::Display> fmt::Display for Analyzed<T> {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        write!(dest, "{} +- {}, tau: {} ......{} for {} entries",
               self.mean,
               self.error,
               match self.correlation_time {
                   Some(tau) => tau.to_string(),
                   None => "Not directly sampled".to_string(),},
               if self.converged { "converged" } else { "NOT CONVERGED!!!"},
               self.number_of_inputs)
    }
}

use std::ops::*;
impl<T: Float + Copy> Add for Analyzed<T>
    where T: Add<Output=T>
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Analyzed { mean: self.mean + rhs.mean,
                   error: (self.error * self.error + rhs.error * rhs.error).sqrt(),
                   number_of_inputs: self.number_of_inputs.min(rhs.number_of_inputs),
                   correlation_time: match (self.correlation_time, rhs.correlation_time) {
                       (Some(time1), Some(time2)) => Some(time1.max(time2)),
                       _ => None,},
                   converged: self.converged & rhs.converged,
        }
    }
}

#[test]
fn add_test(){
    let an1 = Analyzed::<f64> {
        mean: 1.0,
        error: 0.1,
        number_of_inputs: 1000,
        correlation_time: Some(1.0),
        converged: true,
    };
    let an2 = Analyzed::<f64> {
        mean: 2.0,
        error: 0.2,
        number_of_inputs: 1200,
        correlation_time: Some(2.0),
        converged: true,
    };
    let an3 = Analyzed::<f64> {
        mean: 3.0,
        error: (0.1*0.1 + 0.2*0.2).sqrt(),
        correlation_time: Some(2.0),
        number_of_inputs: 1000,
        converged: true,
    };

    assert_eq!(an3, an1 + an2);
}
