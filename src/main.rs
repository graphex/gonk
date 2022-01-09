#![no_main]
#![no_std]

use panic_semihosting as _;
use cortex_m_rt::entry;

use daisy_bsp as daisy;
use daisy::led::Led;

use cortex_m::asm;

// use micromath::F32Ext;
use libm::sqrtf;
use daisy::hal;
use daisy_bsp::loggit;
use hal::prelude::*;
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
use daisy_bsp::hal::gpio::Analog;
use daisy_bsp::hal::i2c::{PinScl, PinSda};
use daisy_bsp::hal::rcc::CoreClocks;
use daisy_bsp::pins::Pins;
use adafruit_led_backpack::*;
use ht16k33::{Display, HT16K33, LedLocation};
use crate::hal::rcc::Ccdr;
use crate::i2c::I2c;
use crate::stm32::{I2C1, Peripherals};
use spectrum_analyzer::scaling::divide_by_N;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};

#[entry]
fn main() -> ! {
    // - board setup ----------------------------------------------------------

    let board = daisy::Board::take().unwrap();
    let dp = daisy::pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();//.sys_ck(320.mhz()).per_ck(40.mhz());
    let mut ccdr = board.freeze_clocks(dp.PWR.constrain(),
                                       rcc,
                                       &dp.SYSCFG);
    // switch adc_ker_ck_input multiplexer to per_ck
    ccdr.peripheral.kernel_adc_clk_mux(AdcClkSel::PER);

    let pins = board.split_gpios(dp.GPIOA.split(ccdr.peripheral.GPIOA),
                                 dp.GPIOB.split(ccdr.peripheral.GPIOB),
                                 dp.GPIOC.split(ccdr.peripheral.GPIOC),
                                 dp.GPIOD.split(ccdr.peripheral.GPIOD),
                                 dp.GPIOE.split(ccdr.peripheral.GPIOE),
                                 dp.GPIOF.split(ccdr.peripheral.GPIOF),
                                 dp.GPIOG.split(ccdr.peripheral.GPIOG));

    loggit!("Board started");

    // - pin setup -------------------------------------------------------------

    let cp = cortex_m::Peripherals::take().unwrap();
    let mut delay = Delay::new(cp.SYST, ccdr.clocks);
    let mut adc1 = adc::Adc::adc1(
        dp.ADC1,
        &mut delay,
        ccdr.peripheral.ADC12,
        &ccdr.clocks,
    ).enable();
    adc1.set_resolution(adc::Resolution::EIGHTBIT);
    adc1.set_sample_time(T_1);

    let mut adc1_ref_pot = pins.SEED_PIN_15.into_analog();
    let scale_factor = 256i16 / 2;//ccdr.clocks.sys_ck().0 as f32 / 65_535.;
    const BUFFER_SIZE: usize = 512;
    loggit!("Scale Factor:{:?}", scale_factor);

    let mut bit = false;
    let mut ctr = 0;
    let mut led_user = daisy::led::LedUser::new(pins.LED_USER);
    let mut test_pin = pins.SEED_PIN_13.into_push_pull_output();
    let mut scl = pins.SEED_PIN_11.into_alternate_af4().set_open_drain();
    let mut sda = pins.SEED_PIN_12.into_alternate_af4().set_open_drain();
    let mut i2c1 = dp.I2C1.i2c(
        (scl, sda),
        400.khz(),
        ccdr.peripheral.I2C1,
        &ccdr.clocks,
    );
    let i2c1_bus = shared_bus::BusManagerSimple::new(i2c1);
    let mut led_matrix = HT16K33::new(i2c1_bus.acquire_i2c(), 0xF0);
    led_matrix.initialize().expect("Could not initialize LED display");
    led_matrix.set_display(Display::ON);


    // - main loop ------------------------------------------------------------
    let one_second = ccdr.clocks.sys_ck().0;
    let mut pcm_buffer: [i16; BUFFER_SIZE] = [0; BUFFER_SIZE];
    loop {
        for i in 0..BUFFER_SIZE
        {
            let raw: u32 = adc1.read(&mut adc1_ref_pot).unwrap();
            pcm_buffer[i] = raw as i16 - scale_factor;
        }
        let mut max: i16 = pcm_buffer.iter().max().unwrap_or(&0).clone();
        let mut min: i16 = pcm_buffer.iter().min().unwrap_or(&0).clone();
        let raw_volume = max - min;
        let volume = (ease_out(raw_volume as f32, 0f32, 7f32, 255f32) + 0.002f32) as u8;

        let mut fbuf = pcm_buffer.into_iter().map(|x| *x as f32);

        // loggit!("Volume:{:?}", volume);

        let hann_window = hann_window(&pcm_buffer);
        // calc spectrum
        let spectrum_hann_window = samples_fft_to_spectrum(
            // (windowed) samples
            &hann_window,
            // sampling rate
            44100,
            // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
            FrequencyLimit::All,
            // optional scale
            Some(&divide_by_N),
        )
            .unwrap();

        for (fr, fr_val) in spectrum_hann_window.data().iter() {
            loggit!("{}Hz => {}", fr, fr_val)
        }

        if bit {
            led_user.off();
            test_pin.set_low().unwrap();
        } else {
            led_user.on();
            test_pin.set_high().unwrap();
        }
        bit = !bit;

        led_matrix.clear_display_buffer();
        for j in 0..volume {
            led_matrix.update_bicolor_led(0, j, Color::Green);
            led_matrix.update_bicolor_led(1, j, Color::Green);
            led_matrix.update_bicolor_led(2, j, Color::Green);
            led_matrix.update_bicolor_led(3, j, Color::Green);
        }
        if (volume > 3) {
            led_matrix.update_bicolor_led(0, volume-1, Color::Yellow);
            led_matrix.update_bicolor_led(1, volume-1, Color::Yellow);
            led_matrix.update_bicolor_led(2, volume-1, Color::Yellow);
            led_matrix.update_bicolor_led(3, volume-1, Color::Yellow);
            led_matrix.update_bicolor_led(0, volume, Color::Red);
            led_matrix.update_bicolor_led(1, volume, Color::Red);
            led_matrix.update_bicolor_led(2, volume, Color::Red);
            led_matrix.update_bicolor_led(3, volume, Color::Red);
        }

        // match (ctr % 3) {
        //     0 => led_matrix.update_bicolor_led(0, 0, Color::Yellow),
        //     1 => {
        //         led_matrix.update_bicolor_led(0, 0, Color::Green);
        //         led_matrix.update_bicolor_led(0, 1, Color::Green);
        //         led_matrix.update_bicolor_led(0, 2, Color::Yellow);
        //         led_matrix.update_bicolor_led(0, 3, Color::Red);
        //         led_matrix.update_bicolor_led(1, 3, Color::Red);
        //         led_matrix.update_bicolor_led(2, 3, Color::Yellow);
        //         led_matrix.update_bicolor_led(3, 3, Color::Yellow);
        //         led_matrix.update_bicolor_led(4, 3, Color::Green);
        //     },
        //     2 => led_matrix.update_display_buffer(LedLocation::new(14, 0).unwrap(), true),
        //     _ => (),
        // };
        led_matrix.write_display_buffer().unwrap();
        ctr = ctr + 1;

        // cortex_m::asm::delay(10000000);
        // cortex_m::asm::delay(one_second / 3);


        // // led_user.set_high().unwrap();
        // led_user.on();
        // asm::delay(ticks);
        //
        // // led_user.set_low().unwrap();
        // led_user.off();
        // asm::delay(ticks);

        // led_user.on();
        // cortex_m::asm::delay(one_second);
        // led_user.off();
        // cortex_m::asm::delay(one_second);
    }
}

fn ease_out(t: f32, b: f32, c: f32, d: f32) -> f32 {
    let t = t / d - 1f32;
    c * sqrtf(1f32 - t * t) + b
}
