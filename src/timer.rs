use ch32v1::ch32v103 as pac;
use riscv::interrupt::free;

pub trait TimerBaseOp<Tim> {
    type Result;

    fn new(tim: Tim, config: TimBaseConfig) -> Self;
    fn enable(&self) -> Self::Result;

    fn disable(&self) -> Self::Result;
}

pub enum TimerError {
    EnableFailed,
    DisableFailed,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Tim {
    Tim1,
    Tim2,
    Tim3,
    Tim4,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum CounterMode {
    Up = 0,
    Down = 1,
}

impl CounterMode {
    pub fn val(&self) -> bool {
        match self {
            Self::Up => true,
            Self::Down => false
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ClockDivision {
    Div1 = 0,
    Div2 = 1,
    Div3 = 2,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PSCReloadMode {
    Update,
    Immediate
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct TimBaseConfig {
    pub prescaler: u16,
    pub counter_mode: CounterMode,
    pub period: u16,
    pub clock_division: ClockDivision,
    pub repetition_counter: u16,
    pub psc_reload_mode: PSCReloadMode
}

pub enum ADVTimer {
    TIM1,
}

pub struct AdvancedTimer {
    pub tim: ADVTimer,
    pub config: TimBaseConfig,
}

impl TimerBaseOp<ADVTimer> for AdvancedTimer {
    type Result = nb::Result<(), TimerError>;

    fn new(tim: ADVTimer, config: TimBaseConfig) -> Self {
        let reg = match tim {
            ADVTimer::TIM1 => unsafe { &(*(pac::TIM1::ptr())) }
        };
        free(|| {
            unsafe {
                reg.ctlr1.modify(|_, w| {
                    // set timer count mode
                    w.dir().bit(config.counter_mode.val())
                    // set clock division factor
                     .ckd().bits(config.clock_division as u8)
                });
                //set timer auto reload value
                reg.atrlr.modify(|_, w| w.bits(config.period));
                // set timer prescaler
                reg.psc.modify(|_, w| w.bits(config.prescaler));
                // set timer reptition counter
                reg.rptcr.modify(|_, w| w.bits(config.repetition_counter));
                // set prescaler reload mode
                reg.swevgr.write(|w| w.bits(config.psc_reload_mode as u16))
            }
        });
        

        Self {
            tim: tim,
            config: config,
        }

    }

    #[inline]
    fn enable(&self) -> Self::Result {
        let reg = match self.tim {
            ADVTimer::TIM1 => unsafe { &(*pac::TIM1::ptr()) },
        };

        free(|| {
            reg.ctlr1.modify(|r, w| {
                if r.cen().is_disabled() {
                    w.cen().enabled()
                } else {
                    w
                }
            });

            if reg.ctlr1.read().cen().is_enabled() {
                Ok(())
            } else {
                Err(nb::Error::Other(TimerError::EnableFailed))
            }
        })
    }

    #[inline]
    fn disable(&self) -> Self::Result {
        let reg = match self.tim {
            ADVTimer::TIM1 => unsafe { &(*pac::TIM1::ptr()) },
        };

        free(|| {
            reg.ctlr1.modify(|r, w| {
                if r.cen().is_enabled() {
                    w.cen().disabled()
                } else {
                    w
                }
            });

            if reg.ctlr1.read().cen().is_disabled() {
                Ok(())
            } else {
                Err(nb::Error::Other(TimerError::DisableFailed))
            }
        })
    }
}
