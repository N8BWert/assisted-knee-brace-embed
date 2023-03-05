//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    prelude::_embedded_hal_blocking_i2c_Read,
    PwmPin,
};
use fugit::RateExtU32;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio::{FunctionUart, Pins},
    i2c::I2C,
    pac,
    prelude::*,
    pwm::{Channel, InputHighRunning, Slices},
    sio::Sio,
    timer::Timer,
    uart::{self, DataBits, StopBits, UartConfig, UartPeripheral},
    usb::UsbBus,
    watchdog::Watchdog,
};

use usb_device::{class_prelude::*, prelude::*};

use usbd_serial::SerialPort;

use core::fmt::Write;
use heapless::String;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

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

    let mut onboard_led = pins.led.into_push_pull_output();

    // Initialize the indiactor pin (tells whether the system is on or off)
    let mut indicator_pin = pins.gpio26.into_push_pull_output();

    // Setup the sticky button to tell whether the device should be on or off
    let on_off_button = pins.gpio28.into_pull_down_input();

    // // Setup the calibration button
    let calibration_button = pins.gpio27.into_pull_down_input();
    let mut calibration_button_down = false;

    // Setup pwm channel 6 (6A will be used by motor 1 and 6B will be used by motor 2)
    let mut pwm_6 = pwm_slices.pwm6;
    pwm_6.set_ph_correct();
    pwm_6.enable();

    // Channel A -> motor 1 PWM --- Channel B -> Motor 2 PWM
    let mut motor_1_pwm_channel = pwm_6.channel_a;
    let motor_1_pwm_pin = motor_1_pwm_channel.output_to(pins.gpio12);
    let mut motor_2_pwm_channel = pwm_6.channel_b;
    let motor_2_pwm_pin = motor_2_pwm_channel.output_to(pins.gpio13);

    // // Initialize the motor 1 directional outputs
    let mut motor_1_forwards = pins.gpio10.into_push_pull_output();
    let mut motor_1_backwards = pins.gpio11.into_push_pull_output();

    // // Initialize the motor 2 directional outputs
    let mut motor_2_forwards = pins.gpio14.into_push_pull_output();
    let mut motor_2_backwards = pins.gpio15.into_push_pull_output();

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(2)
        .build();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);

    // let mut i2c = I2C::i2c0(
    //     peripherals.I2C0,
    //     pins.gpio8.into_mode(),
    //     pins.gpio9.into_mode(),
    //     400.kHz(),
    //     &mut peripherals.RESETS,
    //     120_000_000.Hz(),
    // );

    info!("begin");
    indicator_pin.set_low().unwrap();
    onboard_led.set_low().unwrap();
    motor_1_forwards.set_low().unwrap();
    motor_1_backwards.set_low().unwrap();
    motor_2_forwards.set_low().unwrap();
    motor_2_backwards.set_low().unwrap();
    motor_1_pwm_channel.set_duty(0);
    motor_2_pwm_channel.set_duty(0);
    let mut led_on = false;
    loop {
        if led_on {
            led_on = false;
            indicator_pin.set_low().unwrap();
        } else {
            led_on = true;
            indicator_pin.set_high().unwrap();
        }

        let _ = serial.write(b"Hello, World!\r\n");
        delay.delay_ms(1000);

        // indicator_pin.set_high().unwrap();

        // for i in (20_000..=65_535).step_by(50) {
        //     if on_off_button.is_high().unwrap() {
        //         if forwards {
        //             motor_1_forwards.set_high().unwrap();
        //             motor_1_backwards.set_low().unwrap();
        //             motor_2_forwards.set_high().unwrap();
        //             motor_2_backwards.set_low().unwrap();
        //         } else {
        //             motor_1_forwards.set_low().unwrap();
        //             motor_1_backwards.set_high().unwrap();
        //             motor_2_forwards.set_low().unwrap();
        //             motor_2_backwards.set_high().unwrap();
        //         }
        //     } else {
        //         motor_1_forwards.set_low().unwrap();
        //         motor_1_backwards.set_low().unwrap();
        //         motor_2_forwards.set_low().unwrap();
        //         motor_2_backwards.set_low().unwrap();
        //     }

        //     if calibration_button.is_high().unwrap() && !calibration_button_down {
        //         forwards = false;
        //     } else if calibration_button.is_low().unwrap() && calibration_button_down {
        //         forwards = true;
        //     }

        //     motor_1_pwm_channel.set_duty(i);
        //     motor_2_pwm_channel.set_duty(i);
        //     delay.delay_ms(5);
        // }

        // indicator_pin.set_low().unwrap();

        // for i in (20_000..=65_535).step_by(50).rev() {
        //     if on_off_button.is_high().unwrap() {
        //         if forwards {
        //             motor_1_forwards.set_high().unwrap();
        //             motor_1_backwards.set_low().unwrap();
        //             motor_2_forwards.set_high().unwrap();
        //             motor_2_backwards.set_low().unwrap();
        //         } else {
        //             motor_1_forwards.set_low().unwrap();
        //             motor_1_backwards.set_high().unwrap();
        //             motor_2_forwards.set_low().unwrap();
        //             motor_2_backwards.set_high().unwrap();
        //         }
        //     } else {
        //         motor_1_forwards.set_low().unwrap();
        //         motor_1_backwards.set_low().unwrap();
        //         motor_2_forwards.set_low().unwrap();
        //         motor_2_backwards.set_low().unwrap();
        //     }

        //     if calibration_button.is_high().unwrap() && !calibration_button_down {
        //         forwards = false;
        //     } else if calibration_button.is_low().unwrap() && calibration_button_down {
        //         forwards = true;
        //     }

        //     motor_1_pwm_channel.set_duty(i);
        //     motor_2_pwm_channel.set_duty(i);
        //     delay.delay_ms(5);
        // }
    }
}

// End of file
