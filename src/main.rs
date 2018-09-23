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

use embedded_hal::digital::{InputPin, OutputPin};

// use switch_matrix::{Cols, Matrix, Rows, VirtPin};
use switch_matrix::VirtPin;

entry!(main);

macro_rules! matrix {
    ( $struct_name:ident {
        rows: ( $($row_type:ty),* $(,)* ),
        cols: ( $($col_type:ty),* $(,)* ),
    }
    ) => {
        struct $struct_name {
            rows: ($($row_type),*),
            cols: ($($col_type),*),
        }
        impl $struct_name {
            // fn new(rows: ($($row_type),*), cols: ($($col_type),*)) -> Self {
            //     Self {rows: rows, cols: cols,}
            // }

            fn decompose<'a>(&'a self) ->matrix!(@array2d_type  ($($row_type),*) ($($col_type),*) ) {
                let rows: [&InputPin; 2] = matrix!(@tuple  self.rows,  ($($row_type),*));
                let cols: [&RefCell<OutputPin>; 2] = matrix!(@tuple  self.cols,  ($($col_type),*));
                let mut out: matrix!(@array2d_type  ($($row_type),*) ($($col_type),*) ) = unsafe {::core::mem::uninitialized()};
                for r in 0..rows.len(){
                    for c in 0..cols.len(){
                        out[r][c]= VirtPin{ row: rows[r], col: cols[c] };
                    }
                }
                out
            }
        }
    };
    (@array2d_type ($($row:ty),*) ($($col:ty),*) ) => {
        [matrix!(@array1d_type ($($col),*)) ; matrix!(@count $($row)*)]
    };
    (@array1d_type ($($col:ty),*)) => {
        [matrix!(@element_type) ; matrix!(@count $($col)*)]
    };
    (@element_type ) => {
        VirtPin<'a>
    };
    (@count $($token_trees:tt)*) => {0usize $(+ matrix!(@replace $token_trees 1usize))*};
    (@replace $_t:tt $sub:expr) =>  {$sub};
    (@underscore $unused:tt) => {
        _
    };
    (@destruct $tuple:expr, ($($repeats:ty),*)) => {
        {
            let (
                $(matrix!(@underscore $repeats),)*
                    ref nth, ..) = $tuple;
            nth
        }
    };
    (@tuple_helper $tuple:expr, ($head:ty), ($($result:expr),*  $(,)*)) => {
        [
            matrix!(@destruct $tuple, ()),
            $($result),*
        ]
    };
    (@tuple_helper $tuple:expr, ($head:ty $(,$repeats:ty)* $(,)*),  ($($result:expr),*  $(,)*)) => {
        matrix!(
            @tuple_helper $tuple, ($($repeats),*),
            (
                matrix!(@destruct $tuple, ($($repeats),*)),
                $($result),*
            )
        )
    };
    (@tuple $tuple:expr, ($($repeats:ty),*)) => {
        matrix!(@tuple_helper $tuple, ($($repeats),*) , ())
    };
}

matrix!{
    MyMatrix {
        rows: (
            hal::gpio::gpiob::PB6<Input<PullUp>>,
            hal::gpio::gpiob::PB7<Input<PullUp>>,
        ),
        cols: (
            RefCell<hal::gpio::gpiob::PB4<Output<OpenDrain>>>,
            RefCell<hal::gpio::gpiob::PB5<Output<OpenDrain>>>,
        ),
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

    let mat = MyMatrix {
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
