use ch32v1::ch32v103::{FLASH, RCC};
use riscv::interrupt::free;

const MAX_CLK_FREQ: u32 = 72_000_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PllSrc {
    Hsi,
    Hse(u32),
    HseDiv2(u32),
}

impl PllSrc {
    pub fn bits(&self) -> bool {
        match self {
            Self::Hsi => false,
            _ => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputSrc {
    Hsi,
    Hse(u8),
    Pll(PllSrc),
    Unavailable,
}

impl InputSrc {
    pub fn bits(&self) -> u8 {
        match self {
            InputSrc::Hsi => 0,
            InputSrc::Hse(_) => 1,
            InputSrc::Pll(_) => 2,
            InputSrc::Unavailable => 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PllMul {
    Mul2 = 0b0000,
    Mul3 = 0b0001,
    Mul4 = 0b0010,
    Mul5 = 0b0011,
    Mul6 = 0b0100,
    Mul7 = 0b0101,
    Mul8 = 0b0110,
    Mul9 = 0b0111,
    Mul10 = 0b1000,
    Mul11 = 0b1001,
    Mul12 = 0b1010,
    Mul13 = 0b1011,
    Mul14 = 0b1100,
    Mul15 = 0b1101,
    Mul16 = 0b1110,
}

impl PllMul {
    pub fn val(&self) -> u32 {
        *self as u32 + 2
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AHBPreDiv {
    NoDiv = 0b0000,
    Div2 = 0b1000,
    Div4 = 0b1001,
    Div8 = 0b1010,
    Div16 = 0b1011,
    Div64 = 0b1100,
    Div128 = 0b1101,
    Div256 = 0b1110,
    Div512 = 0b1111,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum APB1PreDiv {
    NoDiv = 0b000,
    Div2 = 0b100,
    Div4 = 0b101,
    Div8 = 0b110,
    Div16 = 0b111,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum APB2PreDiv {
    NoDiv = 0b000,
    Div2 = 0b100,
    Div4 = 0b101,
    Div8 = 0b110,
    Div16 = 0b111,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ADCPreDiv {
    Div2 = 0b00,
    Div4 = 0b01,
    Div6 = 0b10,
    Div8 = 0b11,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum USBPreDiv {
    NoDiv = 0b01,
    Div1_5 = 0b00,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RccError {
    Speed,
    Hardware,
}

pub struct Clocks {
    pub input_src: InputSrc,
    pub sysclk_state: InputSrc,
    pub pllmul: PllMul,
    pub ahb_prediv: AHBPreDiv,
    pub apb1_prediv: APB1PreDiv,
    pub apb2_prediv: APB2PreDiv,
    pub adc_prediv: ADCPreDiv,
    pub css: bool,
    pub hse_bypass: bool,
}

impl Clocks {
    pub fn setup(&self) -> Result<(), RccError> {
        // if let Err(e) = self.validate_freq() {
        //     return Err(e);
        // }

        let rcc = unsafe { &(*(RCC::ptr())) };

        let flash = unsafe { &(*(FLASH::ptr())) };

        flash.actlr.modify(|_, w| {
            w.prftbe().bit(true);
            if self.sysclk() < 48_000_000 {
                w.latency().one()
            } else {
                w.latency().two()
            }
        });

        // let sysclk = self.sysclk();

        free(|| unsafe {
            match self.input_src {
                InputSrc::Hsi => {
                    rcc.ctlr.modify(|_, w| w.hsion().on());
                    while rcc.ctlr.read().hsirdy().is_not_ready() {}
                }
                InputSrc::Hse(_) => {
                    rcc.ctlr.modify(|_, w| w.hseon().on());
                    while rcc.ctlr.read().hserdy().is_not_ready() {}
                }
                InputSrc::Pll(src) => match src {
                    PllSrc::Hse(_) => {
                        rcc.ctlr.modify(|_, w| w.hseon().bit(true));
                        while rcc.ctlr.read().hserdy().is_not_ready() {}
                    }
                    PllSrc::HseDiv2(_) => {
                        rcc.ctlr.modify(|_, w| w.hseon().on());
                        while rcc.ctlr.read().hserdy().is_not_ready() {}
                    }
                    _ => {
                        rcc.ctlr.modify(|_, w| w.hsion().on());
                        while rcc.ctlr.read().hsirdy().is_not_ready() {}
                    }
                },
                InputSrc::Unavailable => {}
            }

            rcc.ctlr.modify(|_, w| w.hsebyp().bit(self.hse_bypass));

            if let InputSrc::Pll(src) = self.input_src {
                rcc.ctlr.modify(|_, w| w.pllon().bit(false));

                while rcc.ctlr.read().pllrdy().is_ready() {}

                rcc.cfgr0
                    .modify(|_, w| w.pllsrc().bit(src.bits()).pllmul().bits(self.pllmul as u8));

                rcc.ctlr.modify(|_, w| w.pllon().bit(true));

                while rcc.ctlr.read().pllrdy().is_not_ready() {}
            }

            rcc.cfgr0.modify(|_, w| {
                w.sw().bits(self.input_src.bits());
                w.hpre().bits(self.ahb_prediv as u8);
                w.ppre1().bits(self.apb1_prediv as u8);
                w.ppre2().bits(self.apb2_prediv as u8)
            });

            rcc.ctlr.modify(|_, w| w.csson().bit(self.css));
        });
        Ok(())
    }

    pub fn sysclk(&self) -> u32 {
        match self.input_src {
            InputSrc::Hsi => 8_000_000,
            InputSrc::Hse(f) => f as u32 * 1_000_000,
            InputSrc::Pll(src) => match src {
                PllSrc::Hse(v) => v * self.pllmul.val() * 1_000_000,
                PllSrc::HseDiv2(v) => v / 2 * self.pllmul.val() * 1_000_000,
                PllSrc::Hsi => 8_000_000 * self.pllmul.val(),
            },
            InputSrc::Unavailable => 0,
        }
    }

    pub fn hclk(&self) -> u32 {
        self.sysclk() / self.ahb_prediv as u32
    }

    pub fn pclk1(&self) -> u32 {
        self.hclk() / self.apb1_prediv as u32
    }

    pub fn pclk2(&self) -> u32 {
        self.hclk() / self.apb2_prediv as u32
    }

    pub fn adc_clk(&self) -> u32 {
        self.pclk2() / self.adc_prediv as u32
    }

    pub fn input_src(&self) -> InputSrc {
        let rcc = unsafe { &(*(RCC::ptr())) };
        free(|| {
            let bits = rcc.cfgr0.read().sws().bits();
            if bits == 0b00 {
                InputSrc::Hsi
            } else if bits == 0b01 || bits == 0b10 {
                self.input_src
            } else {
                InputSrc::Unavailable
            }
        })
    }

    pub fn validate_freq(&self) -> Result<(), RccError> {
        if self.sysclk() > MAX_CLK_FREQ {
            return Err(RccError::Speed);
        }

        if self.hclk() > MAX_CLK_FREQ {
            return Err(RccError::Speed);
        }

        if self.pclk1() > MAX_CLK_FREQ {
            return Err(RccError::Speed);
        }

        if self.pclk2() > MAX_CLK_FREQ {
            return Err(RccError::Speed);
        }

        if self.adc_clk() > 14_000_000 {
            return Err(RccError::Speed);
        }

        Ok(())
    }
}

impl Default for Clocks {
    fn default() -> Self {
        Self {
            input_src: InputSrc::Hsi,
            sysclk_state: InputSrc::Hsi,
            pllmul: PllMul::Mul2,
            ahb_prediv: AHBPreDiv::NoDiv,
            apb1_prediv: APB1PreDiv::NoDiv,
            apb2_prediv: APB2PreDiv::NoDiv,
            adc_prediv: ADCPreDiv::Div2,
            css: false,
            hse_bypass: false,
        }
    }
}
