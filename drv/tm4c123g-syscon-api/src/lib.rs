// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]

use userlib::*;
use zerocopy::AsBytes;

// The system control peripheral registers are always at the same fixed base.
// We can use that fact to encode the offset of the  register in the top part of Peripheral,
// and define the bit offset inside the lower part.
const SYSCTL_SR_BASE_OFFSET: u32 = 0x500;
const SYSCTL_RCGC_BASE_OFFSET: u32 = 0x600;
const SYSCTL_PR_BASE_OFFSET: u32 = 0xA00;
const SYSCTL_WD: u32 = 0x00;
const SYSCTL_TIMER: u32 = 0x04;
const SYSCTL_GPIO: u32 = 0x08;
const SYSCTL_DMA: u32 = 0x0C;
const SYSCTL_HIB: u32 = 0x14;
const SYSCTL_UART: u32 = 0x18;
const SYSCTL_SSI: u32 = 0x1C;
const SYSCTL_I2C: u32 = 0x20;
const SYSCTL_USB: u32 = 0x28;
const SYSCTL_CAN: u32 = 0x34;
const SYSCTL_ADC: u32 = 0x38;
const SYSCTL_ACMP: u32 = 0x3C;
const SYSCTL_PWM: u32 = 0x40;
const SYSCTL_QEI: u32 = 0x44;
const SYSCTL_EEPROM: u32 = 0x58;
const SYSCTL_WTIMER: u32 = 0x5C;

#[derive(Copy, Clone, Eq, PartialEq, Debug, FromPrimitive, AsBytes)]
#[repr(u32)]
pub enum Peripheral {
    Watchdog0 = SYSCTL_WD << 16 | 0,
    Watchdog1 = SYSCTL_WD << 16 | 1,

    Timer0 = SYSCTL_TIMER << 16 | 0,
    Timer1 = SYSCTL_TIMER << 16 | 1,
    Timer2 = SYSCTL_TIMER << 16 | 2,
    Timer3 = SYSCTL_TIMER << 16 | 3,
    Timer4 = SYSCTL_TIMER << 16 | 4,
    Timer5 = SYSCTL_TIMER << 16 | 5,

    GpioA = SYSCTL_GPIO << 16 | 0,
    GpioB = SYSCTL_GPIO << 16 | 1,
    GpioC = SYSCTL_GPIO << 16 | 2,
    GpioD = SYSCTL_GPIO << 16 | 3,
    GpioE = SYSCTL_GPIO << 16 | 4,
    GpioF = SYSCTL_GPIO << 16 | 5,

    Dma = SYSCTL_DMA << 16 | 0,
    Hib = SYSCTL_HIB << 16 | 0,

    Uart0 = SYSCTL_UART << 16 | 0,
    Uart1 = SYSCTL_UART << 16 | 1,
    Uart2 = SYSCTL_UART << 16 | 2,
    Uart3 = SYSCTL_UART << 16 | 3,
    Uart4 = SYSCTL_UART << 16 | 4,
    Uart5 = SYSCTL_UART << 16 | 5,
    Uart6 = SYSCTL_UART << 16 | 6,
    Uart7 = SYSCTL_UART << 16 | 7,

    Ssi0 = SYSCTL_SSI << 16 | 0,
    Ssi1 = SYSCTL_SSI << 16 | 1,
    Ssi2 = SYSCTL_SSI << 16 | 2,
    Ssi3 = SYSCTL_SSI << 16 | 3,

    I2C0 = SYSCTL_I2C << 16 | 0,
    I2C1 = SYSCTL_I2C << 16 | 1,
    I2C2 = SYSCTL_I2C << 16 | 2,
    I2C3 = SYSCTL_I2C << 16 | 3,

    Usb = SYSCTL_USB << 16 | 0,

    Can0 = SYSCTL_CAN << 16 | 0,
    Can1 = SYSCTL_CAN << 16 | 1,

    Adc0 = SYSCTL_ADC << 16 | 0,
    Adc1 = SYSCTL_ADC << 16 | 1,

    Acmp0 = SYSCTL_ACMP << 16 | 0,

    Pwm0 = SYSCTL_PWM << 16 | 0,
    Pwm1 = SYSCTL_PWM << 16 | 1,

    Qei0 = SYSCTL_QEI << 16 | 0,
    Qei1 = SYSCTL_QEI << 16 | 1,

    Eeprom = SYSCTL_EEPROM << 16 | 0,

    WideTimer0 = SYSCTL_WTIMER << 16 | 0,
    WideTimer1 = SYSCTL_WTIMER << 16 | 1,
    WideTimer2 = SYSCTL_WTIMER << 16 | 2,
    WideTimer3 = SYSCTL_WTIMER << 16 | 3,
    WideTimer4 = SYSCTL_WTIMER << 16 | 4,
    WideTimer5 = SYSCTL_WTIMER << 16 | 5,
}

impl Peripheral {
    pub fn as_bit(self) -> u32 {
        (self as u32) & 0xFFFF
    }

    pub fn as_offset(self) -> u32 {
        (self as u32) >> 16
    }

    pub fn as_sr_offset(self) -> u32 {
        self.as_offset() + SYSCTL_SR_BASE_OFFSET
    }

    pub fn as_rcgc_offset(self) -> u32 {
        self.as_offset() + SYSCTL_RCGC_BASE_OFFSET
    }

    pub fn as_pr_offset(self) -> u32 {
        self.as_offset() + SYSCTL_PR_BASE_OFFSET
    }
}

include!(concat!(env!("OUT_DIR"), "/client_stub.rs"));
