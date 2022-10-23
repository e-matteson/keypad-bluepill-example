#![no_std]
#![no_main]

use core::convert::Infallible;
use core::fmt::Write;
use cortex_m_semihosting::hio;

use stm32f1xx_hal::gpio::gpioa::{PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7};
use stm32f1xx_hal::gpio::{Input, OpenDrain, Output, PullUp};
use stm32f1xx_hal::{pac, prelude::*};

use keypad::{embedded_hal::digital::v2::InputPin, keypad_new, keypad_struct};

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m_rt::entry;

// Define the struct that represents your keypad matrix. Give the types of the
// specific pins that will be used for the rows and columns of your matrix. Rows
// must be input pins, and columns must be output pins. Select the modes
// (PullUp/Floating/OpenDrain/PushPull) that are appropriate for your circuit.
keypad_struct! {
    struct MyKeypad<Error = Infallible> {
        rows: (
            PA0<Input<PullUp>>,
            PA1<Input<PullUp>>,
            PA2<Input<PullUp>>,
            PA3<Input<PullUp>>,
            PA4<Input<PullUp>>,
        ),
        columns: (
            PA5<Output<OpenDrain>>,
            PA6<Output<OpenDrain>>,
            PA7<Output<OpenDrain>>,
        ),
    }
}

#[entry]
fn main() -> ! {
    // Print using semihosting
    let mut stdout = hio::hstdout().unwrap();
    writeln!(stdout, "Hello, world!").unwrap();

    // Get access to peripherals
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();

    // Configure clocks
    let mut flash = dp.FLASH.constrain();
    let _clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Get access to port A pins
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // Create an instance of the keypad struct you defined above.
    let keypad = keypad_new!(MyKeypad {
        rows: (
            gpioa.pa0.into_pull_up_input(&mut gpioa.crl),
            gpioa.pa1.into_pull_up_input(&mut gpioa.crl),
            gpioa.pa2.into_pull_up_input(&mut gpioa.crl),
            gpioa.pa3.into_pull_up_input(&mut gpioa.crl),
            gpioa.pa4.into_pull_up_input(&mut gpioa.crl),
        ),
        columns: (
            gpioa.pa5.into_open_drain_output(&mut gpioa.crl),
            gpioa.pa6.into_open_drain_output(&mut gpioa.crl),
            gpioa.pa7.into_open_drain_output(&mut gpioa.crl),
        ),
    });

    // Create a 2d array of virtual `KeyboardInput` pins, each representing 1 key in the
    // matrix. They implement the `InputPin` trait and can (mostly) be used
    // just like any other embedded-hal input pins.
    let keys = keypad.decompose();

    let first_key = &keys[0][0];
    writeln!(stdout, "Is first key low? {:?}", first_key.is_low()).unwrap();

    // Repeatedly read every key and print a message if it's pressed.

    loop {
        for (row_index, row) in keys.iter().enumerate() {
            for (col_index, key) in row.iter().enumerate() {
                if key.is_low().unwrap() {
                    writeln!(stdout, "Pressed: ({}, {})", row_index, col_index).unwrap();
                }
            }
        }
    }
}
