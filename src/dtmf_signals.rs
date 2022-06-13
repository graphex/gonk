use spectrum_analyzer::{FrequencyLimit, FrequencySpectrum};

#[derive(Clone)]
pub struct ButtonFrequency {
    pub frequency: f32,
    pub power_threshold: f32,
    pub upper_bandwidth: f32,
    pub lower_bandwidth: f32,
}

#[derive(Clone)]
pub struct DtmfButtonEval {
    button: DtmfButtonSignal<'static>,
    //the max power reading for the lower frequency of this button
    power_row: f32,
    power_col: f32,
}

impl DtmfButtonEval {
    pub fn from_spectrum(button: DtmfButtonSignal<'static>, spectrum: &FrequencySpectrum) -> DtmfButtonEval {
        let (power_row, power_col) = button.pwr_in_spectrum(spectrum);
        DtmfButtonEval {
            button,
            power_row,
            power_col,
        }
    }
    pub fn new(button: DtmfButtonSignal<'static>, power_row: f32, power_col: f32) -> DtmfButtonEval {
        DtmfButtonEval {
            button,
            power_row,
            power_col,
        }
    }
    pub fn either_triggered(&self) -> bool {
        self.power_row > self.button.row_freq.power_threshold ||
            self.power_col > self.button.col_freq.power_threshold
    }
    pub fn triggered(&self) -> bool {
        self.power_row > self.button.row_freq.power_threshold &&
            self.power_col > self.button.col_freq.power_threshold
    }
}

pub struct DtmfFreqs;

impl DtmfFreqs {
    //named after the last button in the row
    pub const ROW_A: f32 = 697f32;
    pub const ROW_B: f32 = 770f32;
    pub const ROW_C: f32 = 852f32;
    pub const ROW_D: f32 = 941f32;
    //named after the first button in the column
    pub const COL_1: f32 = 1209f32;
    pub const COL_2: f32 = 1336f32;
    pub const COL_3: f32 = 1477f32;
    pub const COL_A: f32 = 1633f32;
}

#[derive(Clone)]
pub struct DtmfButtonSignal<'a> {
    pub name: &'a str,
    pub short_name: &'a str,
    pub row_freq: ButtonFrequency,
    pub col_freq: ButtonFrequency,
}

impl DtmfButtonSignal<'static> {
    fn pwr_in_spectrum(&self, spectrum: &FrequencySpectrum) -> (f32, f32) {
        let limit_a = FrequencyLimit::Range(
            self.row_freq.frequency - self.row_freq.lower_bandwidth,
            self.row_freq.frequency + self.row_freq.upper_bandwidth,
        );
        let limit_b = FrequencyLimit::Range(
            self.col_freq.frequency - self.col_freq.lower_bandwidth,
            self.col_freq.frequency + self.col_freq.upper_bandwidth,
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
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_A,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_1,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _2: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "2",
        short_name: "2",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_A,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_2,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _3: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "3",
        short_name: "3",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_A,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_3,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _A: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "A",
        short_name: "A",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_A,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_A,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _4: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "4",
        short_name: "4",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_B,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_1,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _5: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "5",
        short_name: "5",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_B,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_2,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _6: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "6",
        short_name: "6",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_B,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_3,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _B: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "B",
        short_name: "B",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_B,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_A,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _7: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "7",
        short_name: "7",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_C,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_1,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _8: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "8",
        short_name: "8",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_C,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_2,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _9: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "9",
        short_name: "9",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_C,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_3,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _C: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "C",
        short_name: "C",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_C,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_A,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _STAR: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "STAR",
        short_name: "*",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_D,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_1,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _0: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "0",
        short_name: "0",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_D,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_2,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _POUND: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "POUND",
        short_name: "#",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_D,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_3,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
    };
    pub const _D: DtmfButtonSignal<'static> = DtmfButtonSignal {
        name: "D",
        short_name: "D",
        row_freq: ButtonFrequency {
            frequency: DtmfFreqs::ROW_D,
            power_threshold: STD_THRESHOLD,
            upper_bandwidth: HALF_BANDWIDTH,
            lower_bandwidth: HALF_BANDWIDTH,
        },
        col_freq: ButtonFrequency {
            frequency: DtmfFreqs::COL_A,
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