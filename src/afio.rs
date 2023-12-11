use ch32v1::ch32v103::AFIO;
use riscv::interrupt::free;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SwcfgRemap {
    Disable = 0b000,
    Enable = 0b100,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OscRemap {
    Disable = 0,
    Enable = 1,
}

impl OscRemap {
    pub fn bit(&self) -> bool {
        match self {
            OscRemap::Disable => false,
            OscRemap::Enable => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CanRemap {
    Disable = 0b00,
    Enable = 0b10,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tim3ChRemap {
    Default = 0b00,
    Partial = 0b10,
    Full = 0b11,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tim2ChRemap {
    Default = 0b00,
    Partial1 = 0b01,
    Partial2 = 0b10,
    Full = 0b11,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tim1ChRemap {
    Default = 0b00,
    Partial = 0b01,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Usart3Remap {
    Default = 0b00,
    Partial = 0b01,
    Full = 0b11,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Usart1Remap {
    Default,
    Full,
}

impl Usart1Remap {
    pub fn bit(&self) -> bool {
        match self {
            Usart1Remap::Default => false,
            Usart1Remap::Full => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum I2c1Remap {
    Disable,
    Enable,
}

impl I2c1Remap {
    pub fn bit(&self) -> bool {
        match self {
            I2c1Remap::Disable => false,
            I2c1Remap::Enable => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spi1Remap {
    Disable = 0,
    Enable = 1,
}

impl Spi1Remap {
    pub fn bit(&self) -> bool {
        match self {
            Spi1Remap::Disable => false,
            Spi1Remap::Enable => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AFConfig {
    pub swcfg: SwcfgRemap,
    pub osc: OscRemap,
    pub can: CanRemap,
    pub tim3: Tim3ChRemap,
    pub tim2: Tim2ChRemap,
    pub tim1: Tim1ChRemap,
    pub usart3: Usart3Remap,
    pub usart1: Usart1Remap,
    pub i2c1: I2c1Remap,
    pub spi1: Spi1Remap,
}

impl Default for AFConfig {
    fn default() -> Self {
        Self {
            swcfg: SwcfgRemap::Disable,
            osc: OscRemap::Disable,
            can: CanRemap::Disable,
            tim3: Tim3ChRemap::Default,
            tim2: Tim2ChRemap::Default,
            tim1: Tim1ChRemap::Default,
            usart3: Usart3Remap::Default,
            usart1: Usart1Remap::Default,
            i2c1: I2c1Remap::Disable,
            spi1: Spi1Remap::Disable,
        }
    }
}

impl AFConfig {
    pub fn apply(&self) {
        let afio = unsafe { &(*(AFIO::ptr())) };
        free(|| unsafe {
            afio.pcfr.modify(|_, w| {
                w.swcfg()
                    .bits(self.swcfg as u8)
                    .pd01rm()
                    .bit(self.osc.bit())
                    .canrm()
                    .bits(self.can as u8)
                    .tim3rm()
                    .bits(self.tim3 as u8)
                    .tim2rm()
                    .bits(self.tim2 as u8)
                    .tim1rm()
                    .bits(self.tim1 as u8)
                    .usart3rm()
                    .bits(self.usart3 as u8)
                    .usart1rm()
                    .bit(self.usart1.bit())
                    .i2c1rm()
                    .bit(self.i2c1.bit())
                    .spi1rm()
                    .bit(self.spi1.bit())
            })
        })
    }
}
