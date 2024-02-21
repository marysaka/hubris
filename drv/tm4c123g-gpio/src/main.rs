// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A driver for the STM32xx RCC and GPIO blocks, combined for compactness.

#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

use boxxo;

use drv_tm4c123g_gpio_api::{Gpios, Mode, OutputType, Pin, Pull, Speed, Value};
use drv_tm4c123g_syscon_api::{Peripheral, Syscon};
use idol_runtime::RequestError;
use userlib::*;

task_slot!(SYSCON, syscon_driver);

use boxxo as device;

fn turn_on_gpio_clocks() {
    let syscon = Syscon::from(SYSCON.get_task_id());

    syscon.enable_clock(Peripheral::GpioA);
    syscon.leave_reset(Peripheral::GpioA);

    syscon.enable_clock(Peripheral::GpioB);
    syscon.leave_reset(Peripheral::GpioB);

    syscon.enable_clock(Peripheral::GpioC);
    syscon.leave_reset(Peripheral::GpioC);

    syscon.enable_clock(Peripheral::GpioD);
    syscon.leave_reset(Peripheral::GpioD);

    syscon.enable_clock(Peripheral::GpioE);
    syscon.leave_reset(Peripheral::GpioE);

    syscon.enable_clock(Peripheral::GpioF);
    syscon.leave_reset(Peripheral::GpioF);
}

#[export_name = "main"]
fn main() -> ! {
    turn_on_gpio_clocks();

    // Field messages.
    let mut buffer = [0u8; idl::INCOMING_SIZE];

    let mut server = ServerImpl;
    loop {
        idol_runtime::dispatch(&mut buffer, &mut server);
    }
}

pub trait AnyGpioPeriph {
    fn configure(&self, pin: u8, atts: u16);
    fn set(&self, pin: u8, value: Value);
    fn read(&self, pin: u8) -> Value;
    fn toggle(&self, pin: u8);
}

struct ServerImpl;

#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn get_gpio_regs(port: u8) -> &'static dyn AnyGpioPeriph {
    match port {
        0 => &*device::GPIOA::ptr(),
        1 => &*device::GPIOB::ptr(),
        2 => &*device::GPIOC::ptr(),
        3 => &*device::GPIOD::ptr(),
        4 => &*device::GPIOE::ptr(),
        5 => &*device::GPIOF::ptr(),
        _ => unreachable!(),
    }
}

impl AnyGpioPeriph for device::gpioa::RegisterBlock {
    fn configure(&self, pin: u8, atts: u16) {
        let (mode, output_type, speed, pull, af) =
            Gpios::unpack_attribute(atts);

        let set_mask = 1 << pin;
        let clear_mask = !set_mask;

        let analog_en = matches!(mode, Mode::Analog) as u32;
        let digital_en = !matches!(mode, Mode::Analog) as u32;
        let alternate_en = matches!(mode, Mode::Alternate) as u32;
        let output_en = matches!(mode, Mode::Output) as u32;

        self.amsel.modify(|r, w| unsafe {
            w.bits((r.bits() & clear_mask) | analog_en << pin)
        });

        self.den.modify(|r, w| unsafe {
            w.bits((r.bits() & clear_mask) | digital_en << pin)
        });

        self.afsel.modify(|r, w| unsafe {
            w.bits((r.bits() & clear_mask) | alternate_en << pin)
        });

        self.dir.modify(|r, w| unsafe {
            w.bits((r.bits() & clear_mask) | output_en << pin)
        });

        match mode {
            Mode::Output => match speed {
                Speed::Low => {
                    self.dr2r
                        .modify(|r, w| unsafe { w.bits(r.bits() | set_mask) });
                }
                Speed::Medium => {
                    self.dr4r
                        .modify(|r, w| unsafe { w.bits(r.bits() | set_mask) });
                }
                Speed::High => {
                    self.dr8r
                        .modify(|r, w| unsafe { w.bits(r.bits() | set_mask) });
                }
                Speed::VeryHigh => {
                    self.dr8r
                        .modify(|r, w| unsafe { w.bits(r.bits() | set_mask) });
                    self.slr
                        .modify(|r, w| unsafe { w.bits(r.bits() | set_mask) });
                }
            },
            Mode::Alternate => {
                self.pctl.modify(|r, w| unsafe {
                    let raw_value = r.bits() & !(0xF << (pin * 4));

                    w.bits(raw_value | ((af as u32) << (pin * 4)))
                });
            }
            _ => {}
        };

        let open_drain_en = matches!(output_type, OutputType::OpenDrain) as u32;
        self.odr.modify(|r, w| unsafe {
            w.bits((r.bits() & clear_mask) | open_drain_en << pin)
        });

        if matches!(output_type, OutputType::PushPull)
            && !matches!(mode, Mode::Input)
        {
            let pull_up_en = matches!(pull, Pull::Up) as u32;
            let pull_down_en = matches!(pull, Pull::Down) as u32;

            self.pur.modify(|r, w| unsafe {
                w.bits((r.bits() & clear_mask) | pull_up_en << pin)
            });

            self.pdr.modify(|r, w| unsafe {
                w.bits((r.bits() & clear_mask) | pull_down_en << pin)
            });
        }
    }

    fn set(&self, pin: u8, value: Value) {
        self.data.modify(|r, w| unsafe {
            let new_val = if matches!(value, Value::One) {
                r.bits() | 1 << pin
            } else {
                r.bits() & !(1 << pin)
            };

            w.bits(new_val)
        });
    }

    fn read(&self, pin: u8) -> Value {
        if (self.data.read().bits() & (1 << pin)) != 0 {
            Value::One
        } else {
            Value::Zero
        }
    }

    fn toggle(&self, pin: u8) {
        self.data
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << pin)) });
    }
}

impl idl::InOrderGpiosImpl for ServerImpl {
    fn gpio_configure_raw(
        &mut self,
        _: &RecvMessage,
        pin: Pin,
        packed_attributes: u16,
    ) -> Result<(), RequestError<core::convert::Infallible>> {
        let (port, pin) = pin.unpack();
        unsafe { get_gpio_regs(port) }.configure(pin, packed_attributes);

        Ok(())
    }

    fn set_val(
        &mut self,
        _: &RecvMessage,
        pin: Pin,
        value: Value,
    ) -> Result<(), RequestError<core::convert::Infallible>> {
        let (port, pin) = pin.unpack();
        unsafe { get_gpio_regs(port) }.set(pin, value);

        Ok(())
    }

    fn read_val(
        &mut self,
        _: &RecvMessage,
        pin: Pin,
    ) -> Result<Value, RequestError<core::convert::Infallible>> {
        let (port, pin) = pin.unpack();
        Ok(unsafe { get_gpio_regs(port) }.read(pin))
    }

    fn toggle(
        &mut self,
        _: &RecvMessage,
        pin: Pin,
    ) -> Result<(), RequestError<core::convert::Infallible>> {
        let (port, pin) = pin.unpack();
        unsafe { get_gpio_regs(port) }.toggle(pin);

        Ok(())
    }
}

mod idl {
    use super::{Pin, Value};

    include!(concat!(env!("OUT_DIR"), "/server_stub.rs"));
}
