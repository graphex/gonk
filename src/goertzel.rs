use libm::{cosf, log10f};

pub const PI: f32 = 3.14159265358979323846264338327950288f32;

fn calc_koef(f: f32, fs: f32) -> f32 {
    assert!(f < fs / 2.0);
    assert!(fs > 0.0);
    2.0 * cosf(2.0 * PI * f / fs)
}

/// Helper for remembering the last two filter "running values" for
/// the Geortzel filter implemented as an IIR filter.
#[derive(Copy, Clone, Debug)]
pub struct Vn {
    _1: f32,
    _2: f32
}

/// Holds data for a Goertzel filter
#[derive(Debug)]
pub struct Filter {
    /// The frequency of the filter
    f: f32,
    /// The sampling frequency
    fs: f32,
    /// The Goertzel coefficient, calulcated
    /// from the above mentioned frequencies
    koef: f32,
    /// The running values of the Goertzel filter calculation.
    vn: Vn
}

impl Filter {
    /// Returns a Goertzel filter for the given parameters
    /// * `f` The frequency of the filter
    /// * `fs`  The sampling frequency of the samples to process
    pub fn new(f: f32, fs: f32) -> Filter {
        Filter {
            f: f,
            fs: fs,
            koef: calc_koef(f, fs),
            vn: Vn{_1: 0.0, _2: 0.0}
        }
    }

    /// Resets the filter so that we can start it over again.
    pub fn reset(&mut self) {
        self.vn._1 = 0.0;
        self.vn._2 = 0.0;
    }

    /// Process the samples using the filter.
    /// Returns the resulting power of the signal at the filter frequency
    pub fn process(&mut self, sample: &[f32]) -> f32 {
        kernel(sample, self.koef, &mut self.vn);
        power(self.koef, self.vn, sample.len())
    }
}

/// The "kernel" of the Gortzel filter as an IIR filter
pub fn kernel(sample: &[f32], k: f32, vn: &mut Vn) {
    for x in sample.iter() {
        let t = k * vn._1 - vn._2 + x;
        vn._2 = vn._1;
        vn._1 = t;
    }
}

/// Returns the power of the signal that has passed through a Goertzel
/// filter.
pub fn power(k: f32, vn: Vn, n: usize) -> f32 {
    let mut rslt = vn._1 * vn._1 + vn._2 * vn._2 - k * vn._1 * vn._2;
    if rslt < f32::EPSILON  {
        rslt = f32::EPSILON;
    }
    rslt / (n*n) as f32
}

/// Returns the dBm of the given power of a signal
pub fn dbm(power: f32) -> f32 {
    10.0 * log10f(2.0 * power * 1000.0 / 600.0)
}

/*
This class was modified from https://github.com/sveljko/goertzel

The MIT License (MIT)

Copyright (c) 2015 Srdjan Veljkovic

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */