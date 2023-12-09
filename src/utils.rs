macro_rules! rcc_en_reset {
    (apb1, $periph:expr, $rcc:expr) => {
        paste::paste! {
                $rcc.apb1pcenr.modify(|_, w| w.[<$periph en>]().set_bit());
                $rcc.apb1prstr.modify(|_, w| w.[<$periph rst>]().set_bit());
                $rcc.apb1prstr.modify(|_, w| w.[<$periph rst>]().clear_bit());
        }
    };
    (apb2, $periph:expr, $rcc:expr) => {
        paste::paste! {
                $rcc.apb2pcenr.modify(|_, w| w.[<$periph en>]().set_bit());
                $rcc.apb2prstr.modify(|_, w| w.[<$periph rst>]().set_bit());
                $rcc.apb2prstr.modify(|_, w| w.[<$periph rst>]().clear_bit());
        }
    };
    (ahb1, $periph:expr, $rcc:expr) => {
        paste::paste! { 
                $rcc.ahb1pcenr.modify(|_, w| w.[<$periph en>]().set_bit());
                $rcc.ahb1prstr.modify(|_, w| w.[<$periph rst>]().set_bit());
                $rcc.ahb1prstr.modify(|_, w| w.[<$periph rst>]().clear_bit());
        }
    };
}


pub(crate) use rcc_en_reset;
