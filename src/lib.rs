extern crate num;
use num::Float;

#[derive(Debug, PartialEq)]
pub struct Accumulator<T: Float> {
    dataset: Vec<T>,
}

impl<T: Float>  Accumulator<T>
    where T: std::convert::From<u32> + std::fmt::Debug + std::convert::From<f64>
{
    pub fn new() -> Self {
        Self {
            dataset: Vec::new()
        }
    }
    
    pub fn push(&mut self, observable: T) {
        self.dataset.push(observable);
    }
    
    pub fn binning(self) -> Result<Analyzed<T>, String>{
        match self.dataset.len() {
            0 => Err("Dataset has no data".to_string()),
            num_sample @ 1 ... 31 => {
                let mean = self.dataset.iter().fold(num::zero::<T>(), |sum, &value| sum + value) / (num_sample as u32).into();
                let naive_variance = self.dataset.iter().fold(num::zero::<T>(), |variance, &value| variance + (value - mean) * (value - mean)) / (num_sample as f64 * (num_sample - 1) as f64).into();
                let naive_error = naive_variance.sqrt();
                Ok(
                    Analyzed{
                        mean,
                        error: naive_error,
                        correlation_time: num::zero::<T>(),
                        number_of_inputs: num_sample as u32,
                        converged: false,
                    }
                )
            },
            num_sample => {
                let mean = self.dataset.iter().fold(num::zero::<T>(), |sum, &value| sum + value) / (num_sample as f64).into();
                let naive_variance = self.dataset.iter().fold(num::zero::<T>(), |variance, &value| variance + (value - mean) * (value - mean)) / (num_sample as f64 * (num_sample - 1) as f64).into();
                let error_32;
                let error_16;
                let correlation_time;
                let converged;
                // calculate 32 bins
                {
                    let number_of_bins = 32;
                    let samples_in_bin = num_sample / number_of_bins;
                    let mut bin_mean = Vec::<T>::new();
                    for bin_index in 0 .. number_of_bins {
                        bin_mean.push(self.dataset[bin_index * samples_in_bin .. (bin_index + 1) * samples_in_bin].iter().fold(num::zero::<T>(), |sum, &value| sum + value) / (samples_in_bin as f64).into());
                    }
                    let variance_32 = bin_mean.iter().fold(num::zero::<T>(), |variance, &value| variance + (value - mean) * (value - mean)) / (number_of_bins as f64 * (number_of_bins - 1) as f64).into();
                    error_32 = variance_32.sqrt();
                }
                
                // calculate 16 bins
                {
                    let number_of_bins = 16;
                    let samples_in_bin = num_sample / number_of_bins;
                    let mut bin_mean = Vec::<T>::new();
                    for bin_index in 0 .. number_of_bins {
                        bin_mean.push(self.dataset[bin_index * samples_in_bin .. (bin_index + 1) * samples_in_bin].iter().fold(num::zero::<T>(), |sum, &value| sum + value) / (samples_in_bin as f64).into());
                    }
                    let variance_16 = bin_mean.iter().fold(num::zero::<T>(), |variance, &value| variance + (value - mean) * (value - mean)) / (number_of_bins as f64 * (number_of_bins - 1) as f64).into();
                    error_16 = variance_16.sqrt();
                    correlation_time = variance_16 / naive_variance;
                    if error_32 > error_16 * 1.05.into() { // should not occur
                        converged = false;
                    } else {
                        converged = true;
                    }
                }
                
                Ok(
                    Analyzed{
                        mean,
                        error: error_16,
                        correlation_time: correlation_time,
                        number_of_inputs: num_sample as u32,
                        converged,
                    }
                )
            },
        }
    }
}                   
 
#[test]
fn test_binning(){
    let mut an = Accumulator::<f64>::new();
    an.push(1.0);
    an.push(0.0);
    an.push(-1.0);
    an.push(3.0);
    an.push(-3.0);
    let result = an.binning();
    let correct = Analyzed::<f64> {
        mean: 0.0,
        error: 1.0,
        correlation_time: 0.2,
        number_of_inputs: 5,
        converged: false,
    };
    assert_eq!(&result.unwrap(), &correct);
    
    let er = Accumulator::<f64>::new();
    let result_er = er.binning();
    assert_eq!(result_er.err(), Some("Dataset has no data".to_string()));
}    

#[test]
fn test_push(){
    let mut measurement = Accumulator::<f64> {
        dataset: Vec::new(),
    };
    measurement.push(1.0);
    measurement.push(2.0);
    
    let mut raw_vector = Vec::<f64>::new();
    raw_vector.push(1.0);
    raw_vector.push(2.0);
    
    let test_instance = Accumulator::<f64> {
        dataset: raw_vector,
    };
    assert_eq!(measurement, test_instance);
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Analyzed<T: Float + Copy> {
    pub mean: T,
    pub error: T,
    pub number_of_inputs: u32,
    pub correlation_time: T,
    pub converged: bool
}

use std::fmt;

impl<T: Float + Copy + fmt::Display> fmt::Display for Analyzed<T> {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        write!(dest, "{} +- {}, tau: {} ......{} for {} entries",
               self.mean,
               self.error,
               self.correlation_time,
               if self.converged { "converged" } else { "NOT CONVERGED!!!"},
               self.number_of_inputs)
    }
}

#[test]
fn test_format() {
    let an = Analyzed::<f64> {
        mean: 12.1,
        error: 0.3,
        correlation_time: 1.1,
        number_of_inputs: 1024,
        converged: true,
    };
    assert_eq!(format!("{}",an), "12.1 +- 0.3, tau: 1.1 ......converged for 1024 entries");
}

use std::ops::*;
impl<T: Float> Add for Analyzed<T>
    where T: Add<Output=T>
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Analyzed { mean: self.mean + rhs.mean,
                   error: (self.error * self.error + rhs.error * rhs.error).sqrt(),
                   number_of_inputs: self.number_of_inputs.min(rhs.number_of_inputs),
                   correlation_time: self.correlation_time.max(rhs.correlation_time),
                   converged: self.converged & rhs.converged,
        }
    }
}

impl<T: Float> Mul for Analyzed<T>
    where T: Mul<Output=T>
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Analyzed { mean: self.mean * rhs.mean,
                   error: {
                       let term1 = rhs.mean  * self.error;
                       let term2 = self.mean * rhs.error;
                       ((term1 * term1) + (term2 * term2)).sqrt()
                   },
                   number_of_inputs: self.number_of_inputs.min(rhs.number_of_inputs),
                   correlation_time: self.correlation_time.max(rhs.correlation_time),
                   converged: self.converged & rhs.converged,
        }
    }
}

impl<T: Float> Sub for Analyzed<T>
    where T: Sub<Output=T>
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Analyzed { mean: self.mean - rhs.mean,
                   error: (self.error * self.error + rhs.error * rhs.error).sqrt(),
                   number_of_inputs: self.number_of_inputs.min(rhs.number_of_inputs),
                   correlation_time: self.correlation_time.max(rhs.correlation_time),
                   converged: self.converged & rhs.converged,
        }
    }
}

impl<T: Float> Div for Analyzed<T>
    where T: Div<Output=T>
{
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Analyzed { mean: self.mean / rhs.mean,
                   error: {
                       let term1 = self.error / rhs.mean;
                       let term2 = self.mean * rhs.error / rhs.mean / rhs.mean;
                       (term1 * term1 + term2 * term2).sqrt()
                   },
                   number_of_inputs: self.number_of_inputs.min(rhs.number_of_inputs),
                   correlation_time: self.correlation_time.max(rhs.correlation_time),
                   converged: self.converged & rhs.converged,
        }
    }
}

#[test]
fn test_operation(){
    let an1 = Analyzed::<f64> {
        mean: 1.0,
        error: 0.1,
        number_of_inputs: 1000,
        correlation_time: 1.0,
        converged: true,
    };
    let an2 = Analyzed::<f64> {
        mean: 2.0,
        error: 0.2,
        number_of_inputs: 1200,
        correlation_time: 2.0,
        converged: true,
    };
    let an3 = Analyzed::<f64> {
        mean: 3.0,
        error: (0.1*0.1 + 0.2*0.2).sqrt(),
        correlation_time: 2.0,
        number_of_inputs: 1000,
        converged: true,
    };
    let an4 = Analyzed::<f64> {
        mean: 2.0,
        error: (2.0*0.1*2.0*0.1 + 1.0*0.2*1.0*0.2).sqrt(),
        correlation_time: 2.0,
        number_of_inputs: 1000,
        converged: true,
    };
    let an5 = Analyzed::<f64> {
        mean: -1.0,
        error: (0.1*0.1 + 0.2*0.2).sqrt(),
        correlation_time: 2.0,
        number_of_inputs: 1000,
        converged: true,
    };
    let an6 = Analyzed::<f64> {
        mean: 0.5,
        error: (0.1*0.1/2.0/2.0 + 1.0*0.2*0.2/2.0/2.0/2.0/2.0).sqrt(),
        correlation_time: 2.0,
        number_of_inputs: 1000,
        converged: true,
    };
    assert_eq!(an3, an1 + an2);
    assert_eq!(an4, an1 * an2);
    assert_eq!(an5, an1 - an2);
    assert_eq!(an6, an1 / an2);
}
