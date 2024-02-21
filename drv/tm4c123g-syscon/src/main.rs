// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]

use boxxo as device;
use cortex_m::asm::nop;
use drv_tm4c123g_syscon_api::*;
use idol_runtime::RequestError;
use task_jefe_api::{Jefe, ResetReason};
use userlib::*;

task_slot!(JEFE, jefe);

struct ServerImpl;

impl ServerImpl {
    fn control_clock(&self, peripheral: Peripheral, enable: bool) {
        let bit = peripheral.as_bit();
        let rcgc_offset = peripheral.as_rcgc_offset();

        let syscon_base_addr = device::SYSCTL::ptr() as *const u32 as u32;
        let rcgc_ptr = (syscon_base_addr + rcgc_offset) as *mut u32;

        unsafe {
            let reg_val = rcgc_ptr.read_volatile();

            if enable {
                rcgc_ptr.write_volatile(reg_val | 1 << bit);
            } else {
                rcgc_ptr.write_volatile(reg_val & !(1 << bit));
            }
        }

        // As per manual we need to wait 3 system clocks before accessing module registers (Section 5.2.6):
        nop();
        nop();
        nop();
    }

    fn control_reset(&self, peripheral: Peripheral, enable: bool) {
        let bit = peripheral.as_bit();
        let sr_offset = peripheral.as_sr_offset();
        let pr_offset = peripheral.as_pr_offset();

        let syscon_base_addr = device::SYSCTL::ptr() as *const u32 as u32;
        let sr_ptr = (syscon_base_addr + sr_offset) as *mut u32;
        let pr_ptr = (syscon_base_addr + pr_offset) as *mut u32;

        unsafe {
            let reg_val = sr_ptr.read_volatile();

            if enable {
                sr_ptr.write_volatile(reg_val | 1 << bit);

                // Wait for the peripheral to be ready
                while (pr_ptr.read_volatile() & 1 << bit) == 0 {
                    nop();
                }
            } else {
                sr_ptr.write_volatile(reg_val & !(1 << bit));
            }
        }
    }
}

impl idl::InOrderSysconImpl for ServerImpl {
    fn enable_clock(
        &mut self,
        _: &RecvMessage,
        peripheral: Peripheral,
    ) -> Result<(), RequestError<core::convert::Infallible>> {
        self.control_clock(peripheral, true);
        Ok(())
    }

    fn disable_clock(
        &mut self,
        _: &RecvMessage,
        peripheral: Peripheral,
    ) -> Result<(), RequestError<core::convert::Infallible>> {
        self.control_clock(peripheral, false);
        Ok(())
    }

    fn enter_reset(
        &mut self,
        _: &RecvMessage,
        peripheral: Peripheral,
    ) -> Result<(), RequestError<core::convert::Infallible>> {
        self.control_reset(peripheral, true);
        Ok(())
    }

    fn leave_reset(
        &mut self,
        _: &RecvMessage,
        peripheral: Peripheral,
    ) -> Result<(), RequestError<core::convert::Infallible>> {
        self.control_reset(peripheral, false);
        Ok(())
    }

    fn chip_reset(
        &mut self,
        _: &RecvMessage,
    ) -> Result<(), RequestError<core::convert::Infallible>> {
        todo!()
    }
}

#[export_name = "main"]
fn main() -> ! {
    let syscon = unsafe { &*device::SYSCTL::ptr() };

    // Setup main clock to 25Hmz and PPL to 80Mhz

    // Configure main clock to 25Mhz
    syscon.rcc.write(|w| {
        // Enable bypass
        w.sysctl_rcc_bypass().set_bit();

        // Oscillator source set to Main
        w.sysctl_rcc_oscsrc().sysctl_rcc_oscsrc_main();

        // Main Oscillator not disabled
        w.sysctl_rcc_moscdis().clear_bit();

        // Crystal Frequency set to 25Mhz
        w.sysctl_rcc_xtal().sysctl_rcc_xtal_25mhz();

        // System Clock Divisor disabled
        w.sysctl_rcc_usesysdiv().clear_bit();
        unsafe {
            w.sysctl_rcc_sysdiv().bits(0);
        }

        w
    });

    // Lock and enable PPL
    syscon.misc.write(|w| w.sysctl_misc_plllmis().set_bit());
    syscon.rcc.modify(|_, w| w.sysctl_rcc_pwrdn().clear_bit());

    // Wait for the PPL to be active
    while syscon.pllstat.read().sysctl_pllstat_lock().bit_is_clear() {
        cortex_m::asm::nop();
    }

    // Enable and setup PPL to 80MHz
    syscon.rcc2.write(|w| {
        w.sysctl_rcc2_usercc2().set_bit();
        w.sysctl_rcc2_div400().set_bit();
        w.sysctl_rcc2_sysdiv2lsb().clear_bit();
        unsafe { w.sysctl_rcc2_sysdiv2().bits(2) };
        w.sysctl_rcc2_bypass2().clear_bit();

        w
    });

    // TODO: configure peripheral clocks

    set_reset_reason(syscon);

    let mut incoming = [0; idl::INCOMING_SIZE];
    loop {
        idol_runtime::dispatch(&mut incoming, &mut ServerImpl);
    }
}

fn set_reset_reason(sysctl: &device::sysctl::RegisterBlock) {
    const POR: u32 = 1 << 1;
    const BOR: u32 = 1 << 2;
    const WD0: u32 = 1 << 3;
    const SW: u32 = 1 << 4;
    const WD1: u32 = 1 << 5;

    let resc = sysctl.resc.read().bits();

    let reason = match resc {
        POR => ResetReason::PowerOn,
        BOR => ResetReason::Brownout,
        WD0 => ResetReason::SystemWatchdog,
        SW => ResetReason::SystemCall,
        WD1 => ResetReason::SystemWatchdog,
        _ => ResetReason::Other(resc),
    };

    Jefe::from(JEFE.get_task_id()).set_reset_reason(reason);
}

mod idl {
    use super::Peripheral;

    include!(concat!(env!("OUT_DIR"), "/server_stub.rs"));
}
