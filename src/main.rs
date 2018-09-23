#![no_main]
#![no_std]

extern crate switch_matrix;

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting;
extern crate embedded_hal;
extern crate nb;
extern crate panic_semihosting;
extern crate stm32f103xx_hal as hal;

// #[macro_use]
// extern crate generic_array;

use core::cell::RefCell;

use rt::ExceptionFrame;

// use generic_array::{ArrayLength, GenericArray};

use hal::gpio::{Input, OpenDrain, Output, PullUp};
use hal::prelude::*;
use hal::stm32f103xx;

// use embedded_hal::digital::{InputPin, OutputPin};

// use switch_matrix::{Cols, Matrix, Rows, VirtPin};
use switch_matrix::VirtPin;

entry!(main);

struct MyMatrix {
    rows: (
        hal::gpio::gpiob::PB6<Input<PullUp>>,
        hal::gpio::gpiob::PB7<Input<PullUp>>,
    ),
    cols: (
        RefCell<hal::gpio::gpiob::PB4<Output<OpenDrain>>>,
        RefCell<hal::gpio::gpiob::PB5<Output<OpenDrain>>>,
    ),
}

// matrix!{
//     MyMatrix{
//         rows: [hal::gpio::gpiob::PB6, hal::gpio::gpiob::PB7],
//         cols: [hal::gpio::gpiob::PB4, hal::gpio::gpiob::PB5],
//     }
// }

impl MyMatrix {
    // Return type depends on row and col nums.
    // Can't be part of a trait - unless that's generic too.
    // Return references instead? Or owned virtpins in an iterator?
    // Why does it matter that you get the array of VirtPins?
    fn decompose<'a>(&'a mut self) -> [[VirtPin<'a>; 2]; 2] {
        [
            [
                VirtPin {
                    row: &self.rows.0,
                    col: &self.cols.0,
                },
                VirtPin {
                    row: &self.rows.0,
                    col: &self.cols.1,
                },
            ],
            [
                VirtPin {
                    row: &self.rows.1,
                    col: &self.cols.0,
                },
                VirtPin {
                    row: &self.rows.1,
                    col: &self.cols.1,
                },
            ],
        ]
    }
}

fn main() -> ! {
    let dp = stm32f103xx::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();

    // Configure clocks
    // TODO is the flash stuff needed?
    let mut flash = dp.FLASH.constrain();
    let _clock_freqs = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let pb4 = gpiob.pb4.into_open_drain_output(&mut gpiob.crl);
    let pb5 = gpiob.pb5.into_open_drain_output(&mut gpiob.crl);
    let pb6 = gpiob.pb6.into_pull_up_input(&mut gpiob.crl);
    let pb7 = gpiob.pb7.into_pull_up_input(&mut gpiob.crl);

    let mut mat = MyMatrix {
        rows: (pb6, pb7),
        cols: (RefCell::new(pb4), RefCell::new(pb5)),
    };
    {
        let pins = mat.decompose();
        pins[0][0].is_low();
        pins[1][0].is_low();
        drop(pins);
    }
    drop(mat);

    loop {}
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
