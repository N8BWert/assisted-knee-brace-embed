//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::{
    digital::v2::OutputPin,
    PwmPin,
    prelude::_embedded_hal_blocking_i2c_Read,
};
use panic_probe as _;
use fugit::RateExtU32;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
    pwm::{InputHighRunning, Slices},
    gpio::{Pins, FunctionUart},
    uart::{self, DataBits, StopBits, UartConfig, UartPeripheral},
    i2c::I2C,
};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);
    let mut peripherals = pac::Peripherals::take().unwrap();

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Init PWMS
    let pwm_slices = Slices::new(pac.PWM, &mut pac.RESETS);

    // Initialize the indiactor pin (tells whether the system is on or off)
    let mut indicator_pin = pins.gpio26.into_push_pull_output();

    // Setup the sticky button to tell whether the device should be on or off
    // TODO: test that this is how to setup the type of button
    let on_off_button = pins.gpio28.into_pull_down_input();
    let mut turned_on = false;

    // Setup the calibration button
    let calibration_button = pins.gpio27.into_pull_down_input();

    // Setup pwm channel 6 (6A will be used by motor 1 and 6B will be used by motor 2)
    let mut motor_pwm = pwm_slices.pwm6;
    motor_pwm.set_ph_correct();
    motor_pwm.enable();

    // Channel A -> motor 1 PWM --- Channel B -> Motor 2 PWM
    let mut motor_1_pwm_channel = motor_pwm.channel_a;
    let mut motor_2_pwm_channel = motor_pwm.channel_b;

    // Initialize the motor 1 directional outputs
    let mut motor_1_forwards = pins.gpio10.into_push_pull_output();
    let mut motor_1_backwards = pins.gpio11.into_push_pull_output();

    // Initialize the motor 2 directional outputs
    let mut motor_2_forwards = pins.gpio14.into_push_pull_output();
    let mut motor_2_backwards = pins.gpio15.into_push_pull_output();

    // Setup bluetooth UART on GPIO 4 and GPIO 5
    let uart_pins = (
        pins.gpio4.into_mode::<FunctionUart>(),
        pins.gpio5.into_mode::<FunctionUart>(),
    );

    // Clock init UART or it will freeze
    // TODO: Figure out how to broadcast bluetooth availability
    let uart = UartPeripheral::new(peripherals.UART1, uart_pins, &mut peripherals.RESETS)
        .enable(
            UartConfig::new(9600_u32.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        ).unwrap();

    let mut i2c = I2C::i2c0(
        peripherals.I2C0,
        pins.gpio8.into_mode(),
        pins.gpio9.into_mode(),
        400.kHz(),
        &mut peripherals.RESETS,
        120_000_000.Hz(),
    );

    loop {
        // info!("on!");
        // led_pin.set_high().unwrap();
        // delay.delay_ms(500);
        // info!("off!");
        // led_pin.set_low().unwrap();
        // delay.delay_ms(500);
    }
}

// End of file
