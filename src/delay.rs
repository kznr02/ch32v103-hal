use riscv::asm::delay;
use crate::clocks::{self, Clocks};
pub struct Delay(usize);

impl Delay {
    pub fn new(cp: &Clocks) -> Self {
        Self(cp.sysclk() as usize)
    }

    pub fn delay_us(&self, us: usize) {
        unsafe { delay((us * self.0 / 1_500_000) as u32)};
    }

    pub fn delay_ms(&self, ms: usize) {
        unsafe { delay((ms  * self.0 / 1_500) as u32)};
    }

    pub fn delay_s(&self, s: usize) {
        unsafe { delay((s * self.0 / 3 * 2) as u32)};
    }
}
