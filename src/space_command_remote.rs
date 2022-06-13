use core::cmp::Ordering;
use libm::sqrtf;
use ordered_float::OrderedFloat;
use spectrum_analyzer::{FrequencyLimit, FrequencySpectrum, FrequencyValue};
use crate::dtmf_signals::ButtonFrequency;
use crate::max_pwr_in_range;


pub struct RemoteButtonEval {
    remote_button: RemoteButtonSignal<'static>,
    //the max power reading for the frequency of this button
    power: f32,
}

impl RemoteButtonEval {
    pub fn from_spectrum(remote_button: RemoteButtonSignal<'static>, spectrum: &FrequencySpectrum) -> RemoteButtonEval {
        let power = remote_button.pwr_in_spectrum(spectrum);
        RemoteButtonEval {
            remote_button,
            power,
        }
    }
    pub fn new(remote_button: RemoteButtonSignal<'static>, power: f32) -> RemoteButtonEval {
        RemoteButtonEval {
            remote_button,
            power,
        }
    }
    pub fn display_range(&self) -> u8 {
        libm::fminf(
            7f32,
            RemoteButtonEval::ease_out(self.power as f32, 0f32, 7f32, 10f32),
        ) as u8
    }
    pub fn triggered(&self) -> bool {
        self.power > self.remote_button.freq.power_threshold
    }
    fn ease_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
        let t = t / d - 1f32;
        c * sqrtf(1f32 - t * t) + b
    }
}


pub struct RemoteButtonSignal<'a> {
    pub name: &'a str,
    pub short_name: &'a str,
    pub freq: ButtonFrequency,
}

impl RemoteButtonSignal<'static> {
    fn pwr_in_spectrum(&self, spectrum: &FrequencySpectrum) -> f32 {
        let limit = FrequencyLimit::Range(
            self.freq.frequency - self.freq.lower_bandwidth,
            self.freq.frequency + self.freq.upper_bandwidth,
        );
        max_pwr_in_range(spectrum, limit)
    }
}

const HALF_KHZ: f32 = 500f32;
const STD_THRESHOLD: f32 = 1f32;

pub type FreqKey = OrderedFloat<f32>;

pub struct RemFreqs;

impl RemFreqs {
    pub const CHANNEL_DN: f32 = 40_380f32;
    pub const VOLUME: f32 = 37_880f32;
    pub const OFF_ON: f32 = 38_880f32;
    pub const CHANNEL_UP: f32 = 41_380f32;
}

#[non_exhaustive]
pub struct RemoteSignals;

impl RemoteSignals {
    pub const CHANNEL_DN: RemoteButtonSignal<'static> = RemoteButtonSignal {
        name: "Channel-",
        short_name: "Ch-",
        freq: ButtonFrequency {
            frequency: RemFreqs::CHANNEL_DN,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_KHZ,
            lower_bandwidth: HALF_KHZ,
        },
    };
    pub const VOLUME: RemoteButtonSignal<'static> = RemoteButtonSignal {
        name: "Volume",
        short_name: "VOL",
        freq: ButtonFrequency {
            frequency: RemFreqs::VOLUME,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_KHZ,
            lower_bandwidth: HALF_KHZ,
        },
    };
    pub const OFF_ON: RemoteButtonSignal<'static> = RemoteButtonSignal {
        name: "Off/On",
        short_name: "Pwr",
        freq: ButtonFrequency {
            frequency: RemFreqs::OFF_ON,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_KHZ,
            lower_bandwidth: HALF_KHZ,
        },
    };
    pub const CHANNEL_UP: RemoteButtonSignal<'static> = RemoteButtonSignal {
        name: "Channel+",
        short_name: "Ch+",
        freq: ButtonFrequency {
            frequency: RemFreqs::CHANNEL_UP,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_KHZ,
            lower_bandwidth: HALF_KHZ,
        },
    };
}