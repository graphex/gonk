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
use hal::adc;
use hal::delay::Delay;

use hal::hal as embedded_hal;
use embedded_hal::digital::v2::OutputPin;

#[entry]
fn main() -> ! {
    // - board setup ----------------------------------------------------------

    let board = daisy::Board::take().unwrap();
    let dp = daisy::pac::Peripherals::take().unwrap();

    let mut ccdr = board.freeze_clocks(dp.PWR.constrain(),
                                       dp.RCC.constrain(),
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

    let mut adc1_ref_pot = pins.SEED_PIN_15.into_analog();
    // let scale_factor = ccdr.clocks.sys_ck().0 as f32 / 65_535.;
    let scale_factor = 1000.;
    loggit!("Scale Factor:{:?}", scale_factor);

    let mut led_user = daisy::led::LedUser::new(pins.LED_USER);
    let mut i2c_clock = pins.SEED_PIN_11.into_push_pull_output();
    // let mut led_user = pins.LED_USER.into_push_pull_output();
    let mut bit = false;
    // - main loop ------------------------------------------------------------

    let one_second = ccdr.clocks.sys_ck().0;
    loop {

        let pot_1: u32 = adc1.read(&mut adc1_ref_pot).unwrap();
        // loggit!("pot_1:{:?}", pot_1);
        // let ticks = (pot_1 as f32 * scale_factor) as u32;
        // loggit!("ticks:{:?}", ticks);
        if bit {
            i2c_clock.set_high().unwrap();
        } else {
            i2c_clock.set_low().unwrap();
        }
        bit = !bit;


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
