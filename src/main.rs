#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

pub mod dtmf_signals;
pub mod space_command_remote;
pub mod goertzel;

use core::alloc::Layout;
// use panic_semihosting as _;
use panic_halt as _;
use cortex_m_rt::entry;

use daisy_bsp as daisy;
use daisy::led::Led;

use cortex_m::asm;
use alloc_cortex_m::CortexMHeap;

use libm::sqrtf;
use daisy::hal;
use daisy_bsp::loggit;
use hal::prelude::*;
use hal::pac::RTC;
use hal::pac::rtc;
use hal::rcc::rec::AdcClkSel;
use hal::rcc::rec::I2c123ClkSel;
use hal::rcc::ResetEnable;
use hal::adc;
use hal::delay::Delay;
use hal::i2c;
use hal::stm32;
use crate::hal::rcc::rec::I2c1;

// use hal::hal as embedded_hal;
use daisy::embedded_hal::digital::v2::OutputPin;
use daisy::embedded_hal::blocking::i2c::*;
use daisy_bsp::hal::adc::AdcSampleTime::{T_1, T_64};
use daisy_bsp::hal::gpio::{Analog, PushPull};
use daisy_bsp::hal::i2c::{PinScl, PinSda};
use daisy_bsp::hal::rcc::CoreClocks;
use daisy_bsp::pins::Pins;
use adafruit_led_backpack::*;
use daisy_bsp::hal::gpio::gpiob::PB6;
// use daisy::pac::rtc;
// use daisy::pac::RTC;
use hashbrown::HashMap;
use embedded_time::Clock;
use fugit::Instant;
use ht16k33::{Display, HT16K33, LedLocation};
use crate::hal::rcc::Ccdr;
use crate::i2c::I2c;
use crate::stm32::{I2C1, Peripherals};
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyValue, FrequencyLimit, FrequencySpectrum};
use crate::hal::gpio::Output;

use crate::{dtmf_signals::*, space_command_remote::*};
use crate::goertzel::Filter;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    //TODO: blink the user LED in a pattern
    // loop {
    //     ;//loggit!("OOM");
    // }
    panic!()
}

#[entry]
fn main() -> ! {
    // - board setup ----------------------------------------------------------

    let board = daisy::Board::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = daisy::pac::Peripherals::take().unwrap();
    // Constrain and Freeze power
    let pwr = dp.PWR.constrain();
    let mut pwrcfg = pwr.freeze();
    // // Take the backup power domain
    // let backup = pwrcfg.backup().unwrap();
    // Constrain and Freeze clock
    let mut rcc = dp.RCC.constrain();
    let mut ccdr = rcc
        .sys_ck(400.mhz())
        .per_ck(36.mhz())
        .freeze(pwrcfg, &dp.SYSCFG);

    // switch adc_ker_ck_input multiplexer to per_ck
    ccdr.peripheral.kernel_adc_clk_mux(AdcClkSel::PER);


    let pins = board.split_gpios(dp.GPIOA.split(ccdr.peripheral.GPIOA),
                                 dp.GPIOB.split(ccdr.peripheral.GPIOB),
                                 dp.GPIOC.split(ccdr.peripheral.GPIOC),
                                 dp.GPIOD.split(ccdr.peripheral.GPIOD),
                                 dp.GPIOE.split(ccdr.peripheral.GPIOE),
                                 dp.GPIOF.split(ccdr.peripheral.GPIOF),
                                 dp.GPIOG.split(ccdr.peripheral.GPIOG));

    let mut delay = Delay::new(cp.SYST, ccdr.clocks);

    // let mut rtc = rtc:Rtc::open_or_init(
    //     dp.RTC,
    //     backup.RTC,
    //     rtc::RtcClock::Lsi,
    //     &ccdr.clocks,
    // );


    //loggit!("Board started");

    // Initialize the heap allocator
    let start = cortex_m_rt::heap_start() as usize;
    let size = 1_048_576; // in bytes
    unsafe { ALLOCATOR.init(start, size) }

    // - pin setup -------------------------------------------------------------

    let mut adc1 = adc::Adc::adc1(
        dp.ADC1,
        &mut delay,
        ccdr.peripheral.ADC12,
        &ccdr.clocks,
    ).enable();
    adc1.set_resolution(adc::Resolution::EIGHTBIT);
    adc1.set_sample_time(T_1);

    const BUFFER_SIZE: usize = 2048;
    const SAMPLE_RATE: u32 = 430_000;
    const SCALE_FACTOR: i16 = 256i16 / 2;
    //ccdr.clocks.sys_ck().0 as f32 / 65_535.;
    //loggit!("Scale Factor:{:?}", SCALE_FACTOR);

    //set up goertzel filters for each of the 12 frequencies we are interested in
    let freqs = [
        RemFreqs::CHANNEL_DN, RemFreqs::VOLUME, RemFreqs::OFF_ON, RemFreqs::CHANNEL_UP,
        DtmfFreqs::ROW_A, DtmfFreqs::ROW_B, DtmfFreqs::ROW_C, DtmfFreqs::ROW_D,
        DtmfFreqs::COL_1, DtmfFreqs::COL_2, DtmfFreqs::COL_3, DtmfFreqs::COL_A,
    ];
    let filters: HashMap<FreqKey, Filter> = freqs.map(|curFreq|
        (
            FreqKey::from(curFreq),
            Filter::new(curFreq, SAMPLE_RATE as f32),
        )
    ).iter().cloned().collect();

    let mut adc1_ref_pot = pins.SEED_PIN_15.into_analog();
    let mut bit = false;
    let mut ctr = 0;
    let mut led_user = daisy::led::LedUser::new(pins.LED_USER);
    let mut test_bit = TestBit::new(pins.SEED_PIN_13.into_push_pull_output());

    //setup i2c1 bus for shared use
    let mut scl = pins.SEED_PIN_11.into_alternate_af4().set_open_drain();
    let mut sda = pins.SEED_PIN_12.into_alternate_af4().set_open_drain();
    let mut i2c1 = dp.I2C1.i2c(
        (scl, sda),
        1.mhz(),
        ccdr.peripheral.I2C1,
        &ccdr.clocks,
    );
    let i2c1_bus = shared_bus::BusManagerSimple::new(i2c1);

    //set up LED matrix
    let mut led_matrix = HT16K33::new(i2c1_bus.acquire_i2c(), 0xF0);
    led_matrix.initialize().expect("Could not initialize LED display");
    led_matrix.set_display(Display::ON);


    // - main loop ------------------------------------------------------------
    let _one_second = ccdr.clocks.sys_ck().0;
    let mut pcm_buffer: [i16; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut fbuf: [f32; BUFFER_SIZE] = [0f32; BUFFER_SIZE];
    loop {
        //load the buffer manually
        //getting about 460kHz (SAMPLE_RATE) with what we do in this loop
        // (36MHz adc_ker_ck_input * 80 clock cycles per iteration)
        for i in 0..BUFFER_SIZE
        {
            test_bit.toggle();
            let raw: u32 = adc1.read(&mut adc1_ref_pot).unwrap();
            pcm_buffer[i] = raw as i16 - SCALE_FACTOR;
            fbuf[i] = raw as f32 - SCALE_FACTOR as f32
        }
        let mut max: i16 = pcm_buffer.iter().max().unwrap_or(&0).clone();
        let mut min: i16 = pcm_buffer.iter().min().unwrap_or(&0).clone();
        let raw_volume = max - min;
        let volume = (ease_out(raw_volume as f32, 0f32, 3f32, 255f32) + 0.002f32) as u8;

        // loggit!("Volume:{:?}", volume);

        // let hann_window = hann_window(&fbuf);

        // calc spectra with fft
        // let dtmf_spectrum = samples_fft_to_spectrum(
        //     &hann_window,
        //     SAMPLE_RATE,
        //     FrequencyLimit::Range(600f32, 1700f32),
        //     Some(&divide_by_N),
        // )
        //     .unwrap();
        // for (fr, fr_val) in dtmf_spectrum.data().iter() {
        //     loggit!("{}Hz => {}", fr, fr_val)
        // }
        // let remote_spectrum = samples_fft_to_spectrum(
        //     &hann_window,
        //     SAMPLE_RATE,
        //     FrequencyLimit::Range(37_000f32, 42_000f32),
        //     Some(&divide_by_N),
        // )
        //     .unwrap();


        //calc goertzel frequencies
        let mut filter_results = HashMap::new();
        for (freq, mut filter) in &filters {
            filter_results.insert(freq, filter.clone().process(&fbuf));
        }
        let remote_buttons = [
            RemoteButtonEval::new(
                RemoteSignals::CHANNEL_DN,
                *filter_results.get(&FreqKey::from(RemFreqs::CHANNEL_DN)).unwrap_or(&0f32)),
            RemoteButtonEval::new(
                RemoteSignals::VOLUME,
                *filter_results.get(&FreqKey::from(RemFreqs::VOLUME)).unwrap_or(&0f32)),
            RemoteButtonEval::new(
                RemoteSignals::OFF_ON,
                *filter_results.get(&FreqKey::from(RemFreqs::OFF_ON)).unwrap_or(&0f32)),
            RemoteButtonEval::new(
                RemoteSignals::CHANNEL_UP,
                *filter_results.get(&FreqKey::from(RemFreqs::CHANNEL_UP)).unwrap_or(&0f32)),
        ];

        test_bit.toggle();
        if bit {
            led_user.off();
        } else {
            led_user.on();
        }
        bit = !bit;

        //ok this got too confusing for me and the compiler i think
        // let buttons = [
        //     DtmfSignals::_1, DtmfSignals::_2, DtmfSignals::_3, DtmfSignals::_A,
        //     DtmfSignals::_4, DtmfSignals::_5, DtmfSignals::_6, DtmfSignals::_B,
        //     DtmfSignals::_7, DtmfSignals::_8, DtmfSignals::_9, DtmfSignals::_C,
        //     DtmfSignals::_STAR, DtmfSignals::_0, DtmfSignals::_POUND, DtmfSignals::_D,
        // ];
        // let rows = [DtmfFreqs::ROW_A, DtmfFreqs::ROW_B, DtmfFreqs::ROW_C, DtmfFreqs::ROW_D, ];
        // let cols = [DtmfFreqs::COL_1, DtmfFreqs::COL_2, DtmfFreqs::COL_3, DtmfFreqs::COL_A, ];
        // let mut b: usize = 0;
        // let dtmf_keypad:[[DtmfButtonEval; 4]; 4] = rows.map(|r| {
        //     b = b + 1;
        //     cols.map(|c|
        //         DtmfButtonEval::new(buttons[b-1].clone(),
        //                             *filter_results.get(&FreqKey::from(r)).unwrap_or(&0f32),
        //                             *filter_results.get(&FreqKey::from(c)).unwrap_or(&0f32))
        //     ).iter().cloned().collect()
        // }).iter().cloned().collect();

        let dtmf_keypad = [
            [
                DtmfButtonEval::new(DtmfSignals::_STAR,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_D)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_1)).unwrap_or(&0f32)),
                DtmfButtonEval::new(DtmfSignals::_0,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_D)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_2)).unwrap_or(&0f32)),
                DtmfButtonEval::new(DtmfSignals::_POUND,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_D)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_3)).unwrap_or(&0f32)),
                DtmfButtonEval::new(DtmfSignals::_D,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_D)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_A)).unwrap_or(&0f32)),
            ], [
                DtmfButtonEval::new(DtmfSignals::_7,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_C)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_1)).unwrap_or(&0f32)),
                DtmfButtonEval::new(DtmfSignals::_8,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_C)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_2)).unwrap_or(&0f32)),
                DtmfButtonEval::new(DtmfSignals::_9,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_C)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_3)).unwrap_or(&0f32)),
                DtmfButtonEval::new(DtmfSignals::_C,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_C)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_A)).unwrap_or(&0f32)),
            ], [
                DtmfButtonEval::new(DtmfSignals::_4,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_B)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_1)).unwrap_or(&0f32)),
                DtmfButtonEval::new(DtmfSignals::_5,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_B)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_2)).unwrap_or(&0f32)),
                DtmfButtonEval::new(DtmfSignals::_6,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_B)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_3)).unwrap_or(&0f32)),
                DtmfButtonEval::new(DtmfSignals::_B,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_B)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_A)).unwrap_or(&0f32)),
            ], [
                DtmfButtonEval::new(DtmfSignals::_1,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_A)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_1)).unwrap_or(&0f32)),
                DtmfButtonEval::new(DtmfSignals::_2,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_A)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_2)).unwrap_or(&0f32)),
                DtmfButtonEval::new(DtmfSignals::_3,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_A)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_3)).unwrap_or(&0f32)),
                DtmfButtonEval::new(DtmfSignals::_A,
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::ROW_A)).unwrap_or(&0f32),
                                    *filter_results.get(&FreqKey::from(DtmfFreqs::COL_A)).unwrap_or(&0f32)),
            ]
        ];

        //--- display updates

        led_matrix.clear_display_buffer();

        let mut r = 0u8;
        let mut c = 0u8;
        for row in dtmf_keypad {
            for key in row {
                if key.either_triggered() {
                    led_matrix.update_bicolor_led(c, r + 4, Color::Green);
                }
                if key.triggered() {
                    led_matrix.update_bicolor_led(c, r + 4, Color::Red);
                }
                c = c + 1;
            }
            r = r + 1;
            c = 0u8;
        }

        let mut col = 4u8;
        let mut idx: usize = 0;
        for btn in remote_buttons {
            let curpwr = btn.display_range();
            for k in 0..curpwr {
                led_matrix.update_bicolor_led(col, k, Color::Green);
            }
            if (curpwr > 3) {
                led_matrix.update_bicolor_led(col, curpwr - 1, Color::Yellow);
                led_matrix.update_bicolor_led(col, curpwr, Color::Red);
            }
            if btn.triggered() {
                led_matrix.update_bicolor_led(col, 7, Color::Red);
            }
            col = col + 1;
            idx = idx + 1;
        }


        for j in 0..volume {
            led_matrix.update_bicolor_led(0, j, Color::Green);
        }
        if (volume > 2) {
            led_matrix.update_bicolor_led(0, volume - 1, Color::Yellow);
            led_matrix.update_bicolor_led(0, volume, Color::Red);
        }

        led_matrix.write_display_buffer().unwrap();
        ctr = ctr + 1;
    }
}

struct TestBit {
    bit: bool,
    test_pin: PB6<Output<PushPull>>,
}

impl TestBit {
    pub fn new(test_pin: PB6<Output<PushPull>>) -> TestBit {
        let mut newbit = TestBit { bit: false, test_pin };
        newbit.toggle();
        newbit
    }
    pub fn toggle(&mut self) {
        self.bit = !self.bit;
        self.apply();
    }
    fn apply(&mut self) {
        if self.bit {
            self.test_pin.set_low().unwrap();
        } else {
            self.test_pin.set_high().unwrap();
        }
    }
}


fn ease_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let t = t / d - 1f32;
    c * sqrtf(1f32 - t * t) + b
}





