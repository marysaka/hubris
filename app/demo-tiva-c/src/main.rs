// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]
use core::fmt::Write;
use cortex_m_rt::entry;
use cortex_m_semihosting::hio;

use boxxo as _;

#[entry]
fn main() -> ! {
    let mut stdout = hio::hstdout().unwrap();
    writeln!(stdout, "Hello world").unwrap();

    // NOTE: At boot the main clock isn't 80Mhz but this is rectified by the syscon driver.
    const CYCLES_PER_MS: u32 = 80_000;

    unsafe { kern::startup::start_kernel(CYCLES_PER_MS) }
}
