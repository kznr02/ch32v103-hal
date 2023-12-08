use ch32v1::ch32v103 as pac;
use core::convert::Infallible;
use riscv::interrupt::free;

/// choose which group of GPIO you want to use
pub enum Port {
    /// GPIOA group, will effect GPIOA_xxxR register
    GPIOA,
    /// GPIOB group, will effect GPIOB_xxxR register
    GPIOB,
    /// GPIOC group, will effect GPIOC_xxxR register
    GPIOC,
    /// GPIOD group, will effect GPIOD_xxxR register
    GPIOD,
}

/// Mode of a GPIO pin, it's GPIOx_CFGxR register value
#[derive(Debug, Clone, Copy)]
pub enum PinMode {
    Input = 0b00,
    Output,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputType {
    PushPull,
    OpenDrain,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputSpeed {
    /// GPIO 2Mhz speed
    LowSpeed = 1,
    /// GPIO 10Mhz speed
    MediumSpeed,
    /// GPIO 50Mhz speed
    HighSpeed,
}

#[derive(Debug, Clone, Copy)]
pub enum Pull {
    PullUp,
    PullDown,
}

#[derive(Debug, Clone, Copy)]
pub enum CfgLock {
    Unlock,
    Lock,
}

#[derive(Debug, Clone, Copy)]
pub enum PinState {
    Low,
    High,
}
/// GPIO Pin abstract structure
pub struct Pin {
    port: Port,
    pin: u8,
}

impl Pin {
    /// get GPIO Register
    const fn regs(&self) -> *const pac::gpioa::RegisterBlock {
        _regs(&self.port)
    }

    pub fn new(port: Port, pin: u8, mode: PinMode) -> Self {
        // assert!(pin <= 15, "Pin range: 0~15");
        free(|| {
            let rcc = unsafe { &(*pac::RCC::ptr()) };

            match port {
                Port::GPIOA => {
                    if rcc.apb2pcenr.read().iopaen().bit_is_clear() {
                        rcc.apb2pcenr.modify(|_, w| w.iopaen().set_bit())
                    }
                }
                Port::GPIOB => {
                    if rcc.apb2pcenr.read().iopben().bit_is_clear() {
                        rcc.apb2pcenr.modify(|_, w| w.iopben().set_bit())
                    }
                }
                Port::GPIOC => {
                    if rcc.apb2pcenr.read().iopcen().bit_is_clear() {
                        rcc.apb2pcenr.modify(|_, w| w.iopcen().set_bit())
                    }
                }
                Port::GPIOD => {
                    if rcc.apb2pcenr.read().iopden().bit_is_clear() {
                        rcc.apb2pcenr.modify(|_, w| w.iopden().set_bit())
                    }
                }
            }
        });

        let pin = Self {
            port: port,
            pin: pin,
        };

        pin.mode(mode);

        pin
    }

    pub fn mode(&self, mode: PinMode) {
        let reg = unsafe { &(*self.regs()) };
        free(|| {
            let offset = 4 * (self.pin & !(0x01 << 3));
            if self.pin >> 3 == 0 {
                reg.cfglr.modify(|r, w| {
                    let val = r.bits() & !(0x03 << offset) | ((mode as u32) << offset);
                    unsafe { w.bits(val) }
                })
            } else {
                reg.cfghr.modify(|r, w| {
                    let val = r.bits() & !(0x03 << offset) | ((mode as u32) << offset);
                    unsafe { w.bits(val) }
                })
            }
        })
    }

    pub fn output_type(&self, val: OutputType) {
        let reg = unsafe { &(*self.regs()) };
        free(|| {
            let offset = 4 * (self.pin & !(0x01 << 3));
            if self.pin >> 3 == 0 {
                reg.cfglr.modify(|r, w| {
                    let bits = r.bits() & !(0x0c << offset) | ((val as u32) << (offset + 2));
                    unsafe { w.bits(bits) }
                })
            }
            if self.pin >> 3 == 1 {
                let offset = 4 * (self.pin - 8);
                reg.cfghr.modify(|r, w| {
                    let bits = r.bits() & !(0x0c << offset) | ((val as u32) << (offset + 2));
                    unsafe { w.bits(bits) }
                })
            }
        })
    }

    pub fn output_speed(&self, speed: OutputSpeed) {
        let reg = unsafe { &(*(self.regs())) };

        free(|| {
            if self.pin >> 3 == 0 {
                let offset = 4 * (self.pin & !(0x01 << 3));
                reg.cfglr.modify(|r, w| {
                    let bits = r.bits() & !(0x03 << offset) | (speed as u32);
                    unsafe { w.bits(bits) }
                })
            }
            if self.pin >> 3 == 1 {
                let offset = 4 * (self.pin - 8);
                reg.cfghr.modify(|r, w| {
                    let bits = r.bits() & !(0x03 << offset) | (speed as u32);
                    unsafe { w.bits(bits) }
                })
            }
        })
    }

    pub fn pull(&self, value: Pull) {
        let reg = unsafe { &(*(self.regs())) };

        free(|| {
            let offset = 4 * (self.pin & !(0x01 << 3));

            let mut current_mode = 0x00;

            if self.pin >> 3 == 0 {
                current_mode = (reg.cfglr.read().bits() & (0x03 << offset)) >> offset;
            } else if self.pin >> 3 == 1 {
                current_mode = (reg.cfghr.read().bits() & (0x03 << offset)) >> offset;
            }

            if current_mode == PinMode::Input as u32 {
                reg.outdr.modify(|r, w| {
                    let bits = r.bits() & !(0x01 << self.pin) | ((value as u32) << self.pin);
                    unsafe { w.bits(bits) }
                })
            }
        })
    }

    pub fn cfg_lock(&self, value: CfgLock) {
        let reg = unsafe { &(*(self.regs())) };

        free(|| {
            if reg.lckr.read().lckk().is_unlocked() {
                reg.lckr.write(|w| w.lckk().locked())
            }
        })
    }

    pub fn get_state(&self) -> PinState {
        let reg = unsafe { &(*(self.regs())) };
        free(|| {
            let state = (reg.indr.read().bits() & !(0x01 << self.pin)) >> self.pin;

            if state == 1 {
                PinState::High
            } else {
                PinState::Low
            }
        })
    }

    pub fn set_state(&self, value: PinState) {
        let reg = unsafe { &(*(self.regs())) };

        free(|| match value {
            PinState::Low => reg.bcr.write(|w| unsafe { w.bits(0x01 << self.pin) }),
            PinState::High => reg.bshr.write(|w| unsafe { w.bits(0x01 << self.pin) }),
        })
    }
}

const fn _regs(port: &Port) -> *const pac::gpioa::RegisterBlock {
    match port {
        Port::GPIOA => pac::GPIOA::ptr(),
        Port::GPIOB => pac::GPIOB::ptr(),
        Port::GPIOC => pac::GPIOC::ptr(),
        Port::GPIOD => pac::GPIOD::ptr(),
    }
}
