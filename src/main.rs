#![no_main]
#![no_std]

use panic_halt as _;
use cortex_m_rt::entry;

use daisy_bsp as daisy;
use daisy::led::Led;

use cortex_m::asm;

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
use daisy_bsp::hal::gpio::Analog;
use daisy_bsp::hal::i2c::{PinScl, PinSda};
use daisy_bsp::hal::rcc::CoreClocks;
use daisy_bsp::pins::Pins;
use crate::hal::rcc::Ccdr;
use crate::i2c::I2c;
use crate::stm32::{I2C1, Peripherals};


#[entry]
fn main() -> ! {
    // - board setup ----------------------------------------------------------

    let board = daisy::Board::take().unwrap();
    let dp = daisy::pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut ccdr = board.freeze_clocks(dp.PWR.constrain(),
                                       rcc,
                                       &dp.SYSCFG);
    // switch adc_ker_ck_input multiplexer to per_ck
    // ccdr.peripheral.kernel_adc_clk_mux(AdcClkSel::PER);
    // ccdr.peripheral.kernel_i2c123_clk_mux(I2c123ClkSel::PLL3_R);
    // let i2c1_prec = ccdr.peripheral.I2C1;
    // let i2c1_prec = ccdr.peripheral.I2C1.enable().reset();

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

    // let mut adc1 = adc::Adc::adc1(
    //     dp.ADC1,
    //     &mut delay,
    //     ccdr.peripheral.ADC12,
    //     &ccdr.clocks,
    // ).enable();
    // adc1.set_resolution(adc::Resolution::EIGHTBIT);

    let mut adc1_ref_pot = pins.SEED_PIN_15.into_analog();
    // let scale_factor = ccdr.clocks.sys_ck().0 as f32 / 65_535.;
    let scale_factor = 1000.;
    loggit!("Scale Factor:{:?}", scale_factor);

    let mut bit = false;
    let mut led_user = daisy::led::LedUser::new(pins.LED_USER);
    let mut test_pin = pins.SEED_PIN_13.into_push_pull_output();
    let mut i2c = init_led_panel(
        dp.I2C1,
        ccdr.peripheral.I2C1,
        &ccdr.clocks,
        pins.SEED_PIN_11.into_alternate_af4().set_open_drain(),
        pins.SEED_PIN_12.into_alternate_af4().set_open_drain(),
    );

    // - main loop ------------------------------------------------------------
    let one_second = ccdr.clocks.sys_ck().0;
    loop {

        // let pot_1: u32 = adc1.read(&mut adc1_ref_pot).unwrap();
        // loggit!("pot_1:{:?}", pot_1);
        // let ticks = (pot_1 as f32 * scale_factor) as u32;
        // loggit!("ticks:{:?}", ticks);
        if bit {
            led_user.off();
            test_pin.set_low().unwrap();
            i2c.write(
                0xF0,
                &[0x00,
                    0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00],
            ).unwrap();
        } else {
            led_user.on();
            test_pin.set_high().unwrap();
            i2c.write(
                0xF0,
                &[0x00,
                    0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF],
            ).unwrap();
        }
        bit = !bit;

        // cortex_m::asm::delay(10000000);
        cortex_m::asm::delay(one_second/3);

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

fn init_led_panel(
    i2c1: I2C1,
    ccdr_i2c1: I2c1,
    clocks: &CoreClocks,
    scl: impl PinScl<daisy_bsp::pac::I2C1>,
    sda: impl PinSda<daisy_bsp::pac::I2C1>,
) -> I2c<I2C1> {
    // let scl = scl_pin.into_alternate_af4().set_open_drain();//pins.SEED_PIN_11.into_alternate_af4().set_open_drain();
    // let sda = sda_pin.into_alternate_af4().set_open_drain();//pins.SEED_PIN_12.into_alternate_af4().set_open_drain();
    let mut i2c = i2c1.i2c(
        (scl, sda),
        100.khz(),
        ccdr_i2c1,
        clocks,
    );
    i2c.write(
        0xF0,
        &[0x21],
    ).unwrap();
    i2c.write(
        0xF0,
        &[0x81],
    ).unwrap();
    i2c.write(
        0xF0,
        &[0xEF],
    ).unwrap();
    i2c.write(
        0xF0,
        &[0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    ).unwrap();
    i2c
}
