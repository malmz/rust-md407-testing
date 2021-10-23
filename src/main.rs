#![no_std]
#![no_main]

use core::convert::Infallible;

use bxcan::{filter::Mask32, Frame, StandardId};
use nb::{block, Result};
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
use stm32f4xx_hal as hal;

use cortex_m::peripheral::SCB;
use hal::{
    can::Can,
    pac::{Peripherals, USART1},
    prelude::*,
    rcc::RccExt,
    serial::{config::Config, Serial, Tx},
};

use cortex_m_rt::entry;
use cortex_m_rt::pre_init;
use stm32f4xx_hal::interrupt;

#[pre_init]
unsafe fn startup() {
    (*SCB::PTR).ccr.modify(|r| r & !(1 << 3));
}

struct Address(u8);

trait CanExt {
    fn transmit_msg(&mut self, msg: Message, address: Address)
        -> Result<Option<Frame>, Infallible>;
}

impl<I: bxcan::Instance> CanExt for bxcan::Can<I> {
    fn transmit_msg(
        &mut self,
        msg: Message,
        address: Address,
    ) -> Result<Option<Frame>, Infallible> {
        let id = msg.msg_id() as u16;

        let std_id = StandardId::new(id << 7 | address.0 as u16).unwrap();
        let frame = Frame::new_data(std_id, []);
        self.transmit(&frame)
    }
}

#[derive(Clone, Copy)]
enum Message {
    Ping,
    Pong,
}

impl Message {
    fn msg_id(self) -> u8 {
        match self {
            Message::Ping => 1,
            Message::Pong => 2,
        }
    }
}

fn run() -> ! {
    let dp = Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(168.mhz()).pclk1(8.mhz()).freeze();
    let gpioa = dp.GPIOA.split();
    let tx_pin = gpioa.pa9;

    let mut tx: Tx<USART1> = Serial::tx(dp.USART1, tx_pin, Config::default(), clocks).unwrap();

    let gpiob = dp.GPIOB.split();
    let mut can1 = {
        let rx = gpiob.pb8.into_alternate::<9>();
        let tx = gpiob.pb9.into_alternate();

        let can = Can::new(dp.CAN1, (tx, rx));

        bxcan::Can::new(can)
    };

    /* can1.configure(|config| {
        config.set_bit_timing(0x001b0001);
    }); */

    can1.modify_config().set_bit_timing(0x001b0001);
    can1.modify_filters().enable_bank(0, Mask32::accept_all());

    block!(can1.enable()).unwrap();

    let test = [0, 1, 2, 3, 4, 5, 6, 7];
    let id = 0x0500;

    let (can1, can1_rx) = can1.split();

    loop {
        let frame = Frame::new_data(StandardId::new(id).unwrap(), test);
        block!(can1.transmit(&frame)).unwrap();
    }
}

#[interrupt]
fn CAN1_RX0() {
    c
}

#[entry]
fn main() -> ! {
    run();
}
