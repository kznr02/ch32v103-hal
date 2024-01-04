use ch32v1::ch32v103::{self as pac, AFIO, EXTI};
use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin};
use riscv::interrupt::free;

/// choose which group of GPIO you want to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Port {
    /// GPIOA group, will effect GPIOA_xxxR register
    GPIOA = 0x00,
    /// GPIOB group, will effect GPIOB_xxxR register
    GPIOB = 0x01,
    /// GPIOC group, will effect GPIOC_xxxR register
    GPIOC = 0x02,
    /// GPIOD group, will effect GPIOD_xxxR register
    GPIOD = 0x03,
}

/// Mode of a GPIO pin, it's GPIOx_CFGxR register value

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinMode {
    Input = 0b00,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    PushPull,
    OpenDrain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputSpeed {
    /// GPIO 2Mhz speed
    LowSpeed = 1,
    /// GPIO 10Mhz speed
    MediumSpeed,
    /// GPIO 50Mhz speed
    HighSpeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pull {
    PullUp,
    PullDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfgLock {
    Unlock,
    Lock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntTrigger {
    Rising = 0b01,
    Falling = 0b10,
    RisingFalling = 0b11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
                reg.lckr.modify(|_, w| w.lckk().locked())
            }
            todo!()
        })
    }

    pub fn enable_int(&self, trigger: IntTrigger) {
        let afio = unsafe { &(*AFIO::ptr()) };
        let offset = (self.pin & 0x03) * 4;
        let exti = unsafe { &(*(EXTI::ptr())) };

        free(|| {
            // Set AFIO EXTICRx to enable gpio exti line
            if self.pin <= 3 {
                afio.exticr1
                    .modify(|_, w| unsafe { w.bits((self.port as u32) << offset) })
            } else if self.pin <= 7 {
                afio.exticr2
                    .modify(|_, w| unsafe { w.bits((self.port as u32) << offset) })
            } else if self.pin <= 11 {
                afio.exticr3
                    .modify(|_, w| unsafe { w.bits((self.port as u32) << offset) })
            } else if self.pin <= 15 {
                afio.exticr4
                    .modify(|_, w| unsafe { w.bits((self.port as u32) << offset) })
            }

            // enable exti for pinX
            exti.intenr
                .modify(|_, w| unsafe { w.bits(0x01 << self.pin) });
            // enable interrupt event for pinX
            exti.evenr
                .modify(|_, w| unsafe { w.bits(0x01 << self.pin) });
            // set rising trigger
            exti.rtenr
                .modify(|_, w| unsafe { w.bits((trigger as u32 & 0x01) << self.pin) });
            // set falling trigger
            exti.ftenr
                .modify(|_, w| unsafe { w.bits((trigger as u32 >> 1) << self.pin) });
        })
    }

    pub fn disable_int(&self) {
        let exti = unsafe { &(*(EXTI::ptr())) };

        free(|| {
            // enable exti for pinX
            exti.intenr.modify(|_, w| unsafe { w.bits(0 << self.pin) });
            // enable interrupt event for pinX
            exti.evenr.modify(|_, w| unsafe { w.bits(0 << self.pin) });
            // set rising trigger
            exti.rtenr.modify(|_, w| unsafe { w.bits(0 << self.pin) });
            // set falling trigger
            exti.ftenr.modify(|_, w| unsafe { w.bits(0 << self.pin) });
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

    pub fn is_high(&self) -> bool {
        let reg = unsafe { &(*(self.regs())) };
        free(|| {
            let state = (reg.indr.read().bits() & (0x01 << self.pin)) >> self.pin;
            if state == 0 {
                false
            } else {
                true
            }
        })
    }

    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    pub fn set_high(&self) {
        self.set_state(PinState::High);
    }

    pub fn set_low(&self) {
        self.set_state(PinState::Low);
    }

    pub fn toggle(&self) {
        if self.is_high() {
            self.set_low();
        } else {
            self.set_high();
        }
    }
}

impl InputPin for Pin {
    type Error = Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(Pin::is_high(&self))
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(Pin::is_low(&self))
    }
}

impl OutputPin for Pin {
    type Error = Infallible;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(Pin::set_high(&self))
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(Pin::set_low(&self))
    }

    fn set_state(&mut self, state: embedded_hal::digital::v2::PinState) -> Result<(), Self::Error> {
        match state {
            embedded_hal::digital::v2::PinState::High => Ok(Pin::set_state(&self, PinState::High)),
            embedded_hal::digital::v2::PinState::Low => Ok(Pin::set_state(&self, PinState::Low)),
        }
    }
}

impl ToggleableOutputPin for Pin {
    type Error = Infallible;

    fn toggle(&mut self) -> Result<(), Self::Error> {
        Ok(Pin::toggle(&self))
    }
}

impl StatefulOutputPin for Pin {
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(Pin::is_high(&self))
    }

    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(Pin::is_low(&self))
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
