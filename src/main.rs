#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

use core::alloc::Layout;
use panic_semihosting as _;
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


#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop { loggit!("OOM"); }
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


    loggit!("Board started");

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
    const SAMPLE_RATE: u32 = 450_000;
    const SCALE_FACTOR: i16 = 256i16 / 2;
    //ccdr.clocks.sys_ck().0 as f32 / 65_535.;
    loggit!("Scale Factor:{:?}", SCALE_FACTOR);

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
        //getting about 450khz (SAMPLE_RATE) with what we do in this loop
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
        let volume = (ease_out(raw_volume as f32, 0f32, 7f32, 255f32) + 0.002f32) as u8;

        // loggit!("Volume:{:?}", volume);

        let hann_window = hann_window(&fbuf);

        // calc spectrum
        let spectrum_hann_window = samples_fft_to_spectrum(
            // (windowed) samples
            &hann_window,
            // sampling rate
            SAMPLE_RATE,
            // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
            FrequencyLimit::Range(37_000f32, 42_000f32),
            // optional scale
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

        // for (fr, fr_val) in spectrum_hann_window.data().iter() {
        //     loggit!("{}Hz => {}", fr, fr_val)
        // }
        let ch_dn = max_pwr_in_range(&spectrum_hann_window,
                                     FrequencyLimit::Range(39_980f32, 40_780f32));
        let vol = max_pwr_in_range(&spectrum_hann_window,
                                   FrequencyLimit::Range(37_480f32, 38_280f32));
        let off_on = max_pwr_in_range(&spectrum_hann_window,
                                      FrequencyLimit::Range(38_480f32, 39_280f32));
        let ch_up = max_pwr_in_range(&spectrum_hann_window,
                                     FrequencyLimit::Range(40_980f32, 41_780f32));
        let powers = [ch_dn, vol, off_on, ch_up];
        // let powers = [vol, off_on, ch_dn, ch_up];
        let scaled_powers = powers.iter().map(|x| (libm::fminf(7f32,ease_out(*x as f32, 0f32, 7f32, 10f32)) as u8));
        let mut col = 4u8;
        let mut idx:usize = 0;

        led_matrix.clear_display_buffer();
        for curpwr in scaled_powers {
            // loggit!("Column {} => [{}] {}", col, powers[idx], curpwr);
            for k in 0..curpwr {
                led_matrix.update_bicolor_led(col, k, Color::Green);
            }
            if (curpwr > 3) {
                led_matrix.update_bicolor_led(col, curpwr - 1, Color::Yellow);
                led_matrix.update_bicolor_led(col, curpwr, Color::Red);
            }
            col = col + 1;
            idx = idx + 1;
        }


        for j in 0..volume {
            led_matrix.update_bicolor_led(0, j, Color::Green);
            led_matrix.update_bicolor_led(1, j, Color::Green);
            led_matrix.update_bicolor_led(2, j, Color::Green);
            led_matrix.update_bicolor_led(3, j, Color::Green);
        }
        if (volume > 3) {
            led_matrix.update_bicolor_led(0, volume - 1, Color::Yellow);
            led_matrix.update_bicolor_led(1, volume - 1, Color::Yellow);
            led_matrix.update_bicolor_led(2, volume - 1, Color::Yellow);
            led_matrix.update_bicolor_led(3, volume - 1, Color::Yellow);
            led_matrix.update_bicolor_led(0, volume, Color::Red);
            led_matrix.update_bicolor_led(1, volume, Color::Red);
            led_matrix.update_bicolor_led(2, volume, Color::Red);
            led_matrix.update_bicolor_led(3, volume, Color::Red);
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

fn max_pwr_in_range(spectrum: &FrequencySpectrum, range: FrequencyLimit) -> f32 {
    let mut max = 0f32;
    for (fr, fr_val) in spectrum.data().iter() {
        if fr.val() > range.maybe_min().unwrap_or(0f32)
            && fr.val() < range.maybe_max().unwrap_or(42_000f32) {
            if fr_val.val() > max {
                max = fr_val.val();
            }
        }
    }
    max
}

fn ease_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let t = t / d - 1f32;
    c * sqrtf(1f32 - t * t) + b
}
