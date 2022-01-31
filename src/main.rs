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
use ht16k33::{Display, HT16K33, LedLocation};
use crate::hal::rcc::Ccdr;
use crate::i2c::I2c;
use crate::stm32::{I2C1, Peripherals};
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencySpectrum};
use crate::hal::gpio::Output;

use crate::{dtmf_signals::*, space_command_remote::*};

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

    const BUFFER_SIZE: usize = 1024;
    const SAMPLE_RATE: u32 = 460_000;
    const SCALE_FACTOR: i16 = 256i16 / 2;
    //ccdr.clocks.sys_ck().0 as f32 / 65_535.;
    //loggit!("Scale Factor:{:?}", SCALE_FACTOR);

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
        //getting about 450kHz (SAMPLE_RATE) with what we do in this loop
        // (36MHz adc_ker_ck_input * 80 clock cycles per iteration)
        // ok oscope is saying about 500kHz
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

        let hann_window = hann_window(&fbuf);

        // calc spectra
        let dtmf_spectrum = samples_fft_to_spectrum(
            &hann_window,
            SAMPLE_RATE,
            FrequencyLimit::Range(600f32, 1700f32),
            Some(&divide_by_N),
        )
            .unwrap();
        // for (fr, fr_val) in dtmf_spectrum.data().iter() {
        //     loggit!("{}Hz => {}", fr, fr_val)
        // }

        let remote_spectrum = samples_fft_to_spectrum(
            &hann_window,
            SAMPLE_RATE,
            FrequencyLimit::Range(37_000f32, 42_000f32),
            Some(&divide_by_N),
        )
            .unwrap();

        test_bit.toggle();
        if bit {
            led_user.off();
        } else {
            led_user.on();
        }
        bit = !bit;

        let remote_buttons = [
            RemoteButtonEval::from_spectrum(RemoteSignals::CHANNEL_DN, &remote_spectrum),
            RemoteButtonEval::from_spectrum(RemoteSignals::VOLUME, &remote_spectrum),
            RemoteButtonEval::from_spectrum(RemoteSignals::OFF_ON, &remote_spectrum),
            RemoteButtonEval::from_spectrum(RemoteSignals::CHANNEL_UP, &remote_spectrum),
        ];

        let dtmf_keypad = [
            [
                DtmfButtonEval::from_spectrum(DtmfSignals::_1, &dtmf_spectrum),
                DtmfButtonEval::from_spectrum(DtmfSignals::_2, &dtmf_spectrum),
                DtmfButtonEval::from_spectrum(DtmfSignals::_3, &dtmf_spectrum),
                DtmfButtonEval::from_spectrum(DtmfSignals::_A, &dtmf_spectrum),
            ],
            [
                DtmfButtonEval::from_spectrum(DtmfSignals::_4, &dtmf_spectrum),
                DtmfButtonEval::from_spectrum(DtmfSignals::_5, &dtmf_spectrum),
                DtmfButtonEval::from_spectrum(DtmfSignals::_6, &dtmf_spectrum),
                DtmfButtonEval::from_spectrum(DtmfSignals::_B, &dtmf_spectrum),
            ],
            [
                DtmfButtonEval::from_spectrum(DtmfSignals::_7, &dtmf_spectrum),
                DtmfButtonEval::from_spectrum(DtmfSignals::_8, &dtmf_spectrum),
                DtmfButtonEval::from_spectrum(DtmfSignals::_9, &dtmf_spectrum),
                DtmfButtonEval::from_spectrum(DtmfSignals::_C, &dtmf_spectrum),
            ],
            [
                DtmfButtonEval::from_spectrum(DtmfSignals::_STAR, &dtmf_spectrum),
                DtmfButtonEval::from_spectrum(DtmfSignals::_0, &dtmf_spectrum),
                DtmfButtonEval::from_spectrum(DtmfSignals::_POUND, &dtmf_spectrum),
                DtmfButtonEval::from_spectrum(DtmfSignals::_D, &dtmf_spectrum),
            ],
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





