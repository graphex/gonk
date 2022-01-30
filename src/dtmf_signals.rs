use spectrum_analyzer::{FrequencyLimit, FrequencySpectrum};

pub struct ButtonFrequency {
    pub frequency: f32,
    pub power_threshold: f32,
    pub upper_bandwidth: f32,
    pub lower_bandwidth: f32,
}

pub struct DtmfButtonEval {
    button:DtmfButtonSignal<'static>,
    //the max power reading for the lower frequency of this button
    power_a:f32,
    power_b:f32,
}
impl DtmfButtonEval {
    pub fn from_spectrum(button:DtmfButtonSignal<'static>, spectrum:&FrequencySpectrum) -> DtmfButtonEval {
        let (power_a, power_b) = button.pwr_in_spectrum(spectrum);
        DtmfButtonEval{
            button,
            power_a,
            power_b,
        }
    }
    pub fn either_triggered(&self) -> bool {
        self.power_a > self.button.freq_a.power_threshold ||
            self.power_b > self.button.freq_b.power_threshold
    }
    pub fn triggered(&self) -> bool {
        self.power_a > self.button.freq_a.power_threshold &&
            self.power_b > self.button.freq_b.power_threshold
    }
}


pub struct DtmfButtonSignal<'a> {
    pub name: &'a str,
    pub short_name: &'a str,
    pub freq_a: ButtonFrequency,
    pub freq_b: ButtonFrequency,
}
impl DtmfButtonSignal<'static> {
    fn pwr_in_spectrum(&self, spectrum: &FrequencySpectrum) -> (f32, f32) {
        let limit_a = FrequencyLimit::Range(
            self.freq_a.frequency - self.freq_a.lower_bandwidth,
            self.freq_a.frequency + self.freq_a.upper_bandwidth,
        );
        let limit_b = FrequencyLimit::Range(
            self.freq_b.frequency - self.freq_b.lower_bandwidth,
            self.freq_b.frequency + self.freq_b.upper_bandwidth,
        );
        (
            max_pwr_in_range(spectrum, limit_a),
            max_pwr_in_range(spectrum, limit_b),
        )
    }
}

const STD_THRESHOLD: f32 = 1f32;
const HALF_BANDWIDTH: f32 = 40f32;

#[non_exhaustive]
pub struct DtmfSignals;

impl DtmfSignals {
    pub const _1: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "1",
        short_name: "1",
        freq_a: ButtonFrequency {
            frequency: 697f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1209f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _2: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "2",
        short_name: "2",
        freq_a: ButtonFrequency {
            frequency: 697f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1336f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _3: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "3",
        short_name: "3",
        freq_a: ButtonFrequency {
            frequency: 697f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1477f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _A: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "A",
        short_name: "A",
        freq_a: ButtonFrequency {
            frequency: 697f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1633f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _4: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "4",
        short_name: "4",
        freq_a: ButtonFrequency {
            frequency: 770f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1209f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _5: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "5",
        short_name: "5",
        freq_a: ButtonFrequency {
            frequency: 770f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1336f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _6: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "6",
        short_name: "6",
        freq_a: ButtonFrequency {
            frequency: 770f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1477f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _B: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "B",
        short_name: "B",
        freq_a: ButtonFrequency {
            frequency: 770f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1633f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _7: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "7",
        short_name: "7",
        freq_a: ButtonFrequency {
            frequency: 852f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1209f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _8: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "8",
        short_name: "8",
        freq_a: ButtonFrequency {
            frequency: 852f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1336f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _9: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "9",
        short_name: "9",
        freq_a: ButtonFrequency {
            frequency: 852f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1477f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _C: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "C",
        short_name: "C",
        freq_a: ButtonFrequency {
            frequency: 852f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1633f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _STAR: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "STAR",
        short_name: "*",
        freq_a: ButtonFrequency {
            frequency: 941f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1209f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _0: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "0",
        short_name: "0",
        freq_a: ButtonFrequency {
            frequency: 941f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1336f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _POUND: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "POUND",
        short_name: "#",
        freq_a: ButtonFrequency {
            frequency: 941f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1477f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _D: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "D",
        short_name: "D",
        freq_a: ButtonFrequency {
            frequency: 941f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        freq_b: ButtonFrequency {
            frequency: 1633f32,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
}

pub fn max_pwr_in_range(spectrum: &FrequencySpectrum, btn_freq: FrequencyLimit) -> f32 {
    let mut max = 0f32;
    for (fr, fr_val) in spectrum.data().iter() {
        if fr.val() > btn_freq.maybe_min().unwrap_or(0f32)
            && fr.val() < btn_freq.maybe_max().unwrap_or(42_000f32) {
            if fr_val.val() > max {
                max = fr_val.val();
            }
        }
    }
    max
}