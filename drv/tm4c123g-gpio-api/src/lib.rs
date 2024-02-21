// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]

use userlib::{sys_send, FromPrimitive};
use zerocopy::AsBytes;

#[derive(Copy, Clone, Debug, FromPrimitive, AsBytes)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum Pin {
    GPIOA_0 = 0,
    GPIOA_1 = 1,
    GPIOA_2 = 2,
    GPIOA_3 = 3,
    GPIOA_4 = 4,
    GPIOA_5 = 5,
    GPIOA_6 = 6,
    GPIOA_7 = 7,

    GPIOB_0 = 0 | (1 << 3),
    GPIOB_1 = 1 | (1 << 3),
    GPIOB_2 = 2 | (1 << 3),
    GPIOB_3 = 3 | (1 << 3),
    GPIOB_4 = 4 | (1 << 3),
    GPIOB_5 = 5 | (1 << 3),
    GPIOB_6 = 6 | (1 << 3),
    GPIOB_7 = 7 | (1 << 3),

    GPIOC_0 = 0 | (2 << 3),
    GPIOC_1 = 1 | (2 << 3),
    GPIOC_2 = 2 | (2 << 3),
    GPIOC_3 = 3 | (2 << 3),
    GPIOC_4 = 4 | (2 << 3),
    GPIOC_5 = 5 | (2 << 3),
    GPIOC_6 = 6 | (2 << 3),
    GPIOC_7 = 7 | (2 << 3),

    GPIOD_0 = 0 | (3 << 3),
    GPIOD_1 = 1 | (3 << 3),
    GPIOD_2 = 2 | (3 << 3),
    GPIOD_3 = 3 | (3 << 3),
    GPIOD_4 = 4 | (3 << 3),
    GPIOD_5 = 5 | (3 << 3),
    GPIOD_6 = 6 | (3 << 3),
    GPIOD_7 = 7 | (3 << 3),

    GPIOE_0 = 0 | (4 << 3),
    GPIOE_1 = 1 | (4 << 3),
    GPIOE_2 = 2 | (4 << 3),
    GPIOE_3 = 3 | (4 << 3),
    GPIOE_4 = 4 | (4 << 3),
    GPIOE_5 = 5 | (4 << 3),
    GPIOE_6 = 6 | (4 << 3),
    GPIOE_7 = 7 | (4 << 3),

    GPIOF_0 = 0 | (5 << 4),
    GPIOF_1 = 1 | (5 << 4),
    GPIOF_2 = 2 | (5 << 4),
    GPIOF_3 = 3 | (5 << 4),
    GPIOF_4 = 4 | (5 << 4),
    GPIOF_5 = 5 | (5 << 4),
    GPIOF_6 = 6 | (5 << 4),
    GPIOF_7 = 7 | (5 << 4),
}

impl Pin {
    pub fn unpack(self) -> (u8, u8) {
        let raw: u8 = self as u8;

        let port = raw >> 4;
        let pin = raw & 0x7;

        (port, pin)
    }
}

/// Possible modes for a GPIO pin.
#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum Mode {
    Input = 0b00,
    Output = 0b01,
    Alternate = 0b10,
    Analog = 0b11,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum OutputType {
    PushPull = 0,
    OpenDrain = 1,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum Speed {
    Low = 0b00,
    Medium = 0b01,
    High = 0b10,
    VeryHigh = 0b11,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum Pull {
    None = 0b00,
    Up = 0b01,
    Down = 0b10,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum Alternate {
    AF0 = 0,
    AF1 = 1,
    AF2 = 2,
    AF3 = 3,
    AF4 = 4,
    AF5 = 5,
    AF6 = 6,
    AF7 = 7,
    AF8 = 8,
    AF9 = 9,
    AF10 = 10,
    AF11 = 11,
    AF12 = 12,
    AF13 = 13,
    AF14 = 14,
    AF15 = 15,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive, AsBytes)]
#[repr(u8)]
pub enum Value {
    Zero = 0,
    One = 1,
}

impl Gpios {
    pub fn pack_attribute(
        mode: Mode,
        output_type: OutputType,
        speed: Speed,
        pull: Pull,
        af: Alternate,
    ) -> u16 {
        mode as u16
            | (output_type as u16) << 2
            | (speed as u16) << 3
            | (pull as u16) << 5
            | (af as u16) << 7
    }

    pub fn unpack_attribute(
        packed_attributes: u16,
    ) -> (Mode, OutputType, Speed, Pull, Alternate) {
        let mode = Mode::from_u16(packed_attributes & 0x3).unwrap_lite();
        let output_type =
            OutputType::from_u16((packed_attributes >> 2) & 0x1).unwrap_lite();
        let speed =
            Speed::from_u16((packed_attributes >> 3) & 0x3).unwrap_lite();
        let pull = Pull::from_u16((packed_attributes >> 5) & 0x3).unwrap_lite();
        let af =
            Alternate::from_u16((packed_attributes >> 7) & 0xf).unwrap_lite();

        (mode, output_type, speed, pull, af)
    }

    /// Configures a Pin.
    ///
    /// This is the raw operation, which can be useful if you're doing something
    /// unusual, but see `gpio_configure_output`, `gpio_configure_input`, and
    /// `gpio_configure_alternate` for the common cases.
    pub fn gpio_configure(
        &self,
        pin: Pin,
        mode: Mode,
        output_type: OutputType,
        speed: Speed,
        pull: Pull,
        af: Alternate,
    ) {
        let packed_attributes =
            Self::pack_attribute(mode, output_type, speed, pull, af);

        self.gpio_configure_raw(pin, packed_attributes);
    }

    pub fn gpio_configure_input(&self, pin: Pin, pull: Pull) {
        self.gpio_configure(
            pin,
            Mode::Input,
            OutputType::PushPull, // doesn't matter
            Speed::High,          // doesn't matter
            pull,
            Alternate::AF0, // doesn't matter
        );
    }

    pub fn gpio_configure_output(
        &self,
        pin: Pin,
        output_type: OutputType,
        speed: Speed,
        pull: Pull,
    ) {
        self.gpio_configure(
            pin,
            Mode::Output,
            output_type,
            speed,
            pull,
            Alternate::AF0, // doesn't matter
        );
    }

    pub fn gpio_configure_alternate(
        &self,
        pin: Pin,
        output_type: OutputType,
        speed: Speed,
        pull: Pull,
        af: Alternate,
    ) {
        self.gpio_configure(pin, Mode::Alternate, output_type, speed, pull, af);
    }

    /// Configures the pins in `PinSet` in the given alternate function, which
    /// should be an input.
    ///
    /// This calls `configure_alternate` passing arbitrary values for
    /// `OutputType` and `Speed`. This is appropriate for inputs, but not for
    /// outputs or bidirectional signals.
    pub fn gpio_configure_alternate_input(
        &self,
        pin: Pin,
        pull: Pull,
        af: Alternate,
    ) {
        self.gpio_configure_alternate(
            pin,
            OutputType::OpenDrain,
            Speed::High,
            pull,
            af,
        );
    }
}

include!(concat!(env!("OUT_DIR"), "/client_stub.rs"));
